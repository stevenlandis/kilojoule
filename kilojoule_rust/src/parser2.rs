struct Parser<'a> {
    text: &'a str,
    pool: AstNodePool<'a>,
    idx: usize,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Parser {
            text,
            pool: AstNodePool::new(),
            idx: 0,
        }
    }

    fn peek(&self, n: usize) -> Option<u8> {
        self.text.as_bytes().get(self.idx + n).copied()
    }

    fn is_alpha(val: u8) -> bool {
        ('A' as u8) <= val && val <= ('Z' as u8) || ('a' as u8) <= val && val <= ('z' as u8)
    }

    fn is_numeric(val: u8) -> bool {
        ('0' as u8) <= val && val <= ('9' as u8)
    }

    fn is_alpha_numeric(val: u8) -> bool {
        Parser::is_alpha(val) || Parser::is_numeric(val)
    }

    fn is_whitespace(val: u8) -> bool {
        val == (' ' as u8) || val == ('\n' as u8) || val == ('\t' as u8) || val == ('\r' as u8)
    }

    fn parse_identifier(&mut self) -> Option<AstNodePtr> {
        match self.peek(0) {
            None => {
                return None;
            }
            Some(c0) => {
                if Parser::is_alpha(c0) {
                    c0
                } else {
                    return None;
                }
            }
        };

        let mut idx: usize = 1;
        'outer: loop {
            match self.peek(idx) {
                None => {
                    break 'outer;
                }
                Some(cn) => {
                    if !Parser::is_alpha_numeric(cn) {
                        break 'outer;
                    }
                }
            };
            idx += 1;
        }

        let iden_str =
            std::str::from_utf8(&self.text.as_bytes()[self.idx..self.idx + idx]).unwrap();
        self.idx += idx;
        Some(self.pool.new_identifier(iden_str))
    }

    fn parse_ws(&mut self) {
        'outer: loop {
            match self.peek(0) {
                None => {
                    break 'outer;
                }
                Some(ch) => {
                    if !Parser::is_whitespace(ch) {
                        break 'outer;
                    }
                }
            }
            self.idx += 1;
        }
    }

    fn parse_integer(&mut self) -> Option<AstNodePtr> {
        let mut idx = 0 as usize;
        let mut val = 0 as u64;
        'outer: loop {
            match self.peek(idx) {
                None => {
                    break 'outer;
                }
                Some(ch) => {
                    if Parser::is_numeric(ch) {
                        val *= 10;
                        val += (ch - ('0' as u8)) as u64;
                    } else {
                        break 'outer;
                    }
                }
            }
            idx += 1;
        }

        if idx == 0 {
            return None;
        }
        self.idx += idx;

        Some(self.pool.new_integer(val))
    }

    fn get_err(&self, typ: ParseErrorType) -> ParseError {
        ParseError { idx: self.idx, typ }
    }

    fn parse_base_expr(&mut self) -> Option<Result<AstNodePtr, ParseError>> {
        if self.parse_str_literal(".") {
            let mut expr = self.pool.new_dot();
            self.parse_ws();
            if let Some(iden) = self.parse_identifier() {
                expr = self.pool.new_access(expr, iden)
            }
            return Some(Ok(expr));
        }

        if self.parse_str_literal("(") {
            let expr = match self.parse_expr() {
                None => return Some(Err(self.get_err(ParseErrorType::NoParenContents))),
                Some(expr) => match expr {
                    Err(err) => {
                        return Some(Err(err));
                    }
                    Ok(expr) => expr,
                },
            };
            self.parse_ws();
            if !self.parse_str_literal(")") {
                return Some(Err(self.get_err(ParseErrorType::NoClosingParen)));
            }
            return Some(Ok(expr));
        }

        if let Some(expr) = self.parse_integer() {
            return Some(Ok(expr));
        }
        if let Some(expr) = self.parse_identifier() {
            self.parse_ws();
            if self.parse_str_literal("(") {
                // This is a function call
                self.parse_ws();
                let mut args_node: Option<AstNodePtr> = None;
                loop {
                    match self.parse_expr() {
                        None => {
                            break;
                        }
                        Some(expr) => match expr {
                            Err(err) => {
                                return Some(Err(err));
                            }
                            Ok(expr) => {
                                args_node = Some(match args_node {
                                    None => expr,
                                    Some(args_node) => self.pool.new_list_node(args_node, expr),
                                })
                            }
                        },
                    };

                    self.parse_ws();
                    if !self.parse_str_literal(",") {
                        break;
                    }
                    self.parse_ws();
                }

                if self.parse_str_literal(")") {
                    return Some(Ok(self.pool.new_fcn_call(args_node)));
                } else {
                    return Some(Err(self.get_err(ParseErrorType::NoClosingParenFcnCall)));
                }
            }

            return Some(Ok(expr));
        }

        return None;
    }

    fn parse_str_literal(&mut self, text: &str) -> bool {
        let text_bytes = text.as_bytes();
        let mut idx = 0 as usize;
        while idx < text_bytes.len() {
            match self.peek(idx) {
                None => {
                    return false;
                }
                Some(ch) => {
                    if ch != text_bytes[idx] {
                        return false;
                    }
                }
            };
            idx += 1;
        }
        self.idx += idx;

        return true;
    }

    fn parse_expr(&mut self) -> Option<Result<AstNodePtr, ParseError>> {
        let mut expr = match self.parse_base_expr() {
            None => {
                return None;
            }
            Some(expr) => match expr {
                Err(err) => {
                    return Some(Err(err));
                }
                Ok(expr) => expr,
            },
        };

        #[derive(Clone)]
        enum Op {
            Base,
            Add,
            Pipe,
        }

        #[derive(PartialEq, PartialOrd, Clone, Copy)]
        enum OpOrder {
            Base,
            Add,
            Pipe,
            End,
        }

        struct OpStack {
            stack: Vec<(Op, AstNodePtr, OpOrder)>,
        }

        impl OpStack {
            fn reduce_for_op_order(&mut self, pool: &mut AstNodePool, order: OpOrder) {
                while self.stack.len() >= 2 && self.stack[self.stack.len() - 1].2 <= order {
                    let t0 = self.stack.pop().unwrap();
                    let t1 = self.stack.pop().unwrap();
                    let left = t1.1;
                    let right = t0.1;
                    self.stack.push((
                        t1.0,
                        match t0.0 {
                            Op::Add => pool.new_add(left, right),
                            Op::Pipe => pool.new_pipe(left, right),
                            _ => panic!(),
                        },
                        t1.2,
                    ));
                }
            }
        }

        let mut stack = OpStack {
            stack: vec![(Op::Base, expr, OpOrder::Base)],
        };

        loop {
            self.parse_ws();
            if let Some((next_op, next_order)) = if self.parse_str_literal("|") {
                Some((Op::Pipe, OpOrder::Pipe))
            } else if self.parse_str_literal("+") {
                Some((Op::Add, OpOrder::Add))
            } else {
                break;
            } {
                self.parse_ws();
                let next_expr = match self.parse_base_expr() {
                    None => {
                        return Some(Err(self.get_err(ParseErrorType::NoExprAfterOperator)));
                    }
                    Some(expr) => match expr {
                        Err(err) => {
                            return Some(Err(err));
                        }
                        Ok(expr) => expr,
                    },
                };

                stack.reduce_for_op_order(&mut self.pool, next_order);
                stack.stack.push((next_op, next_expr, next_order));
            }
        }

        stack.reduce_for_op_order(&mut self.pool, OpOrder::End);

        Some(Ok(stack.stack.pop().unwrap().1))
    }
}

#[derive(Debug)]
struct ParseError {
    idx: usize,
    typ: ParseErrorType,
}

#[derive(Debug)]
enum ParseErrorType {
    NoClosingParen,
    NoClosingParenFcnCall,
    NoParenContents,
    NoExprAfterOperator,
}

#[derive(Debug)]
enum AstNode<'a> {
    Null,
    Identifier(&'a str),
    Integer(u64),
    Pipe(AstNodePtr, AstNodePtr),
    Dot,
    Access(AstNodePtr, AstNodePtr),
    Add(AstNodePtr, AstNodePtr),
    FcnCall(Option<AstNodePtr>),
    ListNode(AstNodePtr, AstNodePtr),
}

type AstNodePtr = usize;

struct AstNodePool<'a> {
    vals: Vec<AstNode<'a>>,
}

impl<'a> AstNodePool<'a> {
    fn new() -> Self {
        AstNodePool { vals: Vec::new() }
    }

    fn new_null(&mut self) -> AstNodePtr {
        let ptr = self.vals.len() as AstNodePtr;
        self.vals.push(AstNode::Null);
        ptr
    }

    fn new_identifier(&mut self, text: &'a str) -> AstNodePtr {
        let ptr = self.vals.len() as AstNodePtr;
        self.vals.push(AstNode::Identifier(text));
        ptr
    }

    fn new_integer(&mut self, val: u64) -> AstNodePtr {
        let ptr = self.vals.len() as AstNodePtr;
        self.vals.push(AstNode::Integer(val));
        ptr
    }

    fn new_pipe(&mut self, left: AstNodePtr, right: AstNodePtr) -> AstNodePtr {
        let ptr = self.vals.len();
        self.vals.push(AstNode::Pipe(left, right));
        ptr
    }

    fn new_dot(&mut self) -> AstNodePtr {
        let ptr = self.vals.len();
        self.vals.push(AstNode::Dot);
        ptr
    }

    fn new_access(&mut self, expr: AstNodePtr, accessor: AstNodePtr) -> AstNodePtr {
        let ptr = self.vals.len();
        self.vals.push(AstNode::Access(expr, accessor));
        ptr
    }

    fn new_add(&mut self, left: AstNodePtr, right: AstNodePtr) -> AstNodePtr {
        let ptr = self.vals.len();
        self.vals.push(AstNode::Add(left, right));
        ptr
    }

    fn new_list_node(&mut self, left: AstNodePtr, right: AstNodePtr) -> AstNodePtr {
        let ptr = self.vals.len();
        self.vals.push(AstNode::ListNode(left, right));
        ptr
    }

    fn new_fcn_call(&mut self, args: Option<AstNodePtr>) -> AstNodePtr {
        let ptr = self.vals.len();
        self.vals.push(AstNode::FcnCall(args));
        ptr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_parser() {
        let mut parser = Parser::new("stuff + 1 + 2 | . | (9 | 10)");
        let result = parser.parse_expr();
        println!("Got result {:?}", result);
        for (idx, elem) in parser.pool.vals.iter().enumerate() {
            println!("{}: {:?}", idx, elem);
        }

        assert!(false);
    }
}
