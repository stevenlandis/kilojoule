use super::ast_node_pool::{AstNode, AstNodePool, AstNodePtr};

pub struct Parser<'a> {
    text: &'a str,
    pub pool: AstNodePool<'a>,
    idx: usize,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str) -> Self {
        Parser {
            text,
            pool: AstNodePool::new(),
            idx: 0,
        }
    }

    pub fn get_node(&self, node: AstNodePtr) -> &AstNode {
        self.pool.get(node)
    }

    fn peek(&self, n: usize) -> Option<u8> {
        self.text.as_bytes().get(self.idx + n).copied()
    }

    fn is_alpha(val: u8) -> bool {
        ('A' as u8) <= val && val <= ('Z' as u8) || ('a' as u8) <= val && val <= ('z' as u8)
    }

    fn is_alpha_underscore(val: u8) -> bool {
        Parser::is_alpha(val) || val == ('_' as u8)
    }

    fn is_numeric(val: u8) -> bool {
        ('0' as u8) <= val && val <= ('9' as u8)
    }

    fn is_alpha_underscore_numeric(val: u8) -> bool {
        Parser::is_alpha_underscore(val) || Parser::is_numeric(val)
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
                if Parser::is_alpha_underscore(c0) {
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
                    if !Parser::is_alpha_underscore_numeric(cn) {
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

    fn parse_list_literal(&mut self) -> Option<Result<AstNodePtr, ParseError>> {
        if !self.parse_str_literal("[") {
            return None;
        }

        let mut parts: Option<AstNodePtr> = None;

        loop {
            self.parse_ws();
            let elem = match self.parse_expr() {
                None => {
                    break;
                }
                Some(val) => match val {
                    Err(err) => {
                        return Some(Err(err));
                    }
                    Ok(val) => val,
                },
            };
            match parts {
                None => {
                    parts = Some(elem);
                }
                Some(prev_val) => {
                    parts = Some(self.pool.new_list_node(prev_val, elem));
                }
            }

            self.parse_ws();

            if !self.parse_str_literal(",") {
                break;
            }
        }

        if !self.parse_str_literal("]") {
            return Some(Err(self.get_err(ParseErrorType::NoListLiteralEndingBracket)));
        }

        Some(Ok(self.pool.new_list_literal(parts)))
    }

    fn parse_map_literal(&mut self) -> Option<Result<AstNodePtr, ParseError>> {
        if !self.parse_str_literal("{") {
            return None;
        }

        let mut parts: Option<AstNodePtr> = None;

        loop {
            self.parse_ws();
            let key = match self.parse_identifier() {
                None => {
                    break;
                }
                Some(val) => val,
            };
            self.parse_ws();

            if !self.parse_str_literal(":") {
                return Some(Err(self.get_err(ParseErrorType::NoColonInMapLiteral)));
            }
            self.parse_ws();

            let val = match self.parse_expr() {
                None => {
                    return Some(Err(self.get_err(ParseErrorType::NoMapLiteralValue)));
                }
                Some(val) => match val {
                    Err(err) => {
                        return Some(Err(err));
                    }
                    Ok(val) => val,
                },
            };

            let kv_pair = self.pool.new_map_kv_pair(key, val);
            match parts {
                None => {
                    parts = Some(kv_pair);
                }
                Some(prev_val) => {
                    parts = Some(self.pool.new_list_node(prev_val, kv_pair));
                }
            }

            self.parse_ws();

            if !self.parse_str_literal(",") {
                break;
            }
        }

        if !self.parse_str_literal("}") {
            return Some(Err(self.get_err(ParseErrorType::NoMapLiteralEndingBrace)));
        }

        Some(Ok(self.pool.new_map_literal(parts)))
    }

    fn get_substring(&mut self, start_offset: usize, end_offset: usize) -> AstNodePtr {
        self.pool.new_identifier(
            std::str::from_utf8(
                &self.text.as_bytes()[self.idx + start_offset..self.idx + end_offset],
            )
            .unwrap(),
        )
    }

    fn parse_format_string(&mut self, quote_char: u8) -> Option<Result<AstNodePtr, ParseError>> {
        if self.peek(0) != Some(quote_char) {
            return None;
        }
        self.idx += 1;

        let mut parts: Option<AstNodePtr> = None;

        let mut idx = 0 as usize;
        loop {
            match self.peek(idx) {
                None => {
                    return Some(Err(self.get_err(ParseErrorType::NoClosingQuoteOnString)));
                }
                Some(ch) => {
                    if ch == quote_char || ch == ('{' as u8) {
                        let part = self.get_substring(0, idx);
                        match parts {
                            None => {
                                parts = Some(part);
                            }
                            Some(prev) => {
                                parts = Some(self.pool.new_list_node(prev, part));
                            }
                        }

                        if ch == quote_char {
                            break;
                        } else {
                            self.idx += idx + 1;
                            self.parse_ws();
                            let part = match self.parse_expr() {
                                None => {
                                    return Some(Err(
                                        self.get_err(ParseErrorType::NoExprInFormatString)
                                    ));
                                }
                                Some(expr) => match expr {
                                    Err(err) => {
                                        return Some(Err(err));
                                    }
                                    Ok(expr) => expr,
                                },
                            };

                            match parts {
                                None => {
                                    parts = Some(part);
                                }
                                Some(prev) => {
                                    parts = Some(self.pool.new_list_node(prev, part));
                                }
                            }

                            self.parse_ws();
                            if !self.parse_str_literal("}") {
                                return Some(Err(
                                    self.get_err(ParseErrorType::NoClosingBraceInFormatString)
                                ));
                            }
                            idx = 0;
                        }
                    } else if ch == ('\\' as u8) {
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
            }
        }

        self.idx += idx + 1;

        Some(Ok(self.pool.new_format_string(parts)))
    }

    fn parse_base_expr_with_accesses(&mut self) -> Option<Result<AstNodePtr, ParseError>> {
        let mut expr = match self.parse_base_expr() {
            None => return None,
            Some(expr) => match expr {
                Err(err) => {
                    return Some(Err(err));
                }
                Ok(expr) => expr,
            },
        };

        loop {
            let accessor = match self.parse_access() {
                None => {
                    break;
                }
                Some(expr) => match expr {
                    Err(err) => {
                        return Some(Err(err));
                    }
                    Ok(expr) => expr,
                },
            };
            expr = self.pool.new_pipe(expr, accessor);
        }

        Some(Ok(expr))
    }

    fn parse_base_expr(&mut self) -> Option<Result<AstNodePtr, ParseError>> {
        if self.parse_str_literal(".") {
            let mut expr = self.pool.new_dot();
            self.parse_ws();
            if let Some(iden) = self.parse_identifier() {
                let access = self.pool.new_access(iden);
                expr = self.pool.new_pipe(expr, access);
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

        if let Some(result) = self.parse_map_literal() {
            return Some(result);
        }

        if let Some(result) = self.parse_list_literal() {
            return Some(result);
        }

        if let Some(result) = self.parse_format_string('\'' as u8) {
            return Some(result);
        }

        if let Some(result) = self.parse_format_string('"' as u8) {
            return Some(result);
        }

        if let Some(expr) = self.parse_integer() {
            return Some(Ok(expr));
        }
        if self.parse_str_literal("null") {
            return Some(Ok(self.pool.new_null()));
        }
        if self.parse_str_literal("true") {
            return Some(Ok(self.pool.new_bool(true)));
        }
        if self.parse_str_literal("false") {
            return Some(Ok(self.pool.new_bool(false)));
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
                    return Some(Ok(self.pool.new_fcn_call(expr, args_node)));
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

    fn parse_access(&mut self) -> Option<Result<AstNodePtr, ParseError>> {
        if self.parse_str_literal(".") {
            self.parse_ws();
            let identifier = match self.parse_identifier() {
                None => {
                    return Some(Err(self.get_err(ParseErrorType::NoIdentifierAfterDotAccess)));
                }
                Some(val) => val,
            };

            Some(Ok(self.pool.new_access(identifier)))
        } else if self.parse_str_literal("[") {
            self.parse_ws();
            let expr = match self.parse_expr() {
                None => {
                    return Some(Err(
                        self.get_err(ParseErrorType::NoExpressionForBracketAccess)
                    ));
                }
                Some(expr) => match expr {
                    Err(err) => {
                        return Some(Err(err));
                    }
                    Ok(expr) => expr,
                },
            };
            if !self.parse_str_literal("]") {
                return Some(Err(
                    self.get_err(ParseErrorType::NoClosingBracketForBracketAccess)
                ));
            }

            Some(Ok(self.pool.new_access(expr)))
        } else {
            None
        }
    }

    pub fn external_parse_expr(&mut self) -> Option<Result<AstNodePtr, ParseError>> {
        self.parse_ws();
        let expr = match self.parse_expr() {
            None => self.pool.new_null(),
            Some(expr) => match expr {
                Err(err) => {
                    return Some(Err(err));
                }
                Ok(expr) => expr,
            },
        };
        self.parse_ws();
        if self.idx < self.text.as_bytes().len() {
            return Some(Err(self.get_err(ParseErrorType::IncompleteParse)));
        }

        Some(Ok(expr))
    }

    fn parse_expr(&mut self) -> Option<Result<AstNodePtr, ParseError>> {
        let expr = match self.parse_base_expr_with_accesses() {
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
            Or,
            And,
            Add,
            Subtract,
            Pipe,
        }

        #[derive(PartialEq, PartialOrd, Clone, Copy)]
        enum OpOrder {
            End,
            Pipe,
            Or,
            And,
            Add,
            Base,
        }

        struct OpStack {
            stack: Vec<(Op, AstNodePtr, OpOrder)>,
        }

        impl OpStack {
            fn reduce_for_op_order(&mut self, pool: &mut AstNodePool, order: OpOrder) {
                while self.stack.len() >= 2 && self.stack[self.stack.len() - 1].2 >= order {
                    let t0 = self.stack.pop().unwrap();
                    let t1 = self.stack.pop().unwrap();
                    let left = t1.1;
                    let right = t0.1;
                    self.stack.push((
                        t1.0,
                        match t0.0 {
                            Op::Pipe => pool.new_pipe(left, right),
                            Op::Or => pool.new_or(left, right),
                            Op::And => pool.new_and(left, right),
                            Op::Add => pool.new_add(left, right),
                            Op::Subtract => pool.new_subtract(left, right),
                            Op::Base => panic!(),
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
            } else if self.parse_str_literal("or") {
                Some((Op::Or, OpOrder::Or))
            } else if self.parse_str_literal("and") {
                Some((Op::And, OpOrder::And))
            } else if self.parse_str_literal("+") {
                Some((Op::Add, OpOrder::Add))
            } else if self.parse_str_literal("-") {
                Some((Op::Subtract, OpOrder::Add))
            } else {
                break;
            } {
                self.parse_ws();
                let next_expr = match self.parse_base_expr_with_accesses() {
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
pub struct ParseError {
    idx: usize,
    typ: ParseErrorType,
}

#[derive(Debug)]
enum ParseErrorType {
    NoClosingParen,
    NoClosingParenFcnCall,
    NoParenContents,
    NoExprAfterOperator,
    NoColonInMapLiteral,
    NoMapLiteralValue,
    NoMapLiteralEndingBrace,
    NoListLiteralEndingBracket,
    NoIdentifierAfterDotAccess,
    NoClosingBracketForBracketAccess,
    NoExpressionForBracketAccess,
    NoClosingQuoteOnString,
    NoExprInFormatString,
    NoClosingBraceInFormatString,
    IncompleteParse,
}
