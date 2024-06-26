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

    fn parse_identifier(&mut self, exclude_keywords: bool) -> Option<AstNodePtr> {
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

        if exclude_keywords
            && (match iden_str {
                "and" => true,
                "or" => true,
                _ => false,
            })
        {
            return None;
        }

        self.idx += idx;
        Some(self.pool.new_node(AstNode::Identifier(iden_str)))
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

    fn parse_at_least_one_ws(&mut self) -> bool {
        match self.peek(0) {
            None => {
                return false;
            }
            Some(ch) => {
                if !Parser::is_whitespace(ch) {
                    return false;
                }
            }
        }
        self.parse_ws();
        true
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

            let is_spread = if self.parse_str_literal("*") {
                self.parse_ws();
                true
            } else {
                false
            };

            let mut elem = match self.parse_expr() {
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
            if is_spread {
                elem = self.pool.new_node(AstNode::Spread(elem));
            }
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

            if self.parse_str_literal("*") {
                self.parse_ws();
                let expr = match self.parse_expr() {
                    None => {
                        return Some(Err(self.get_err(ParseErrorType::NoExpressionAfterMapSpread)))
                    }
                    Some(expr) => match expr {
                        Err(err) => {
                            return Some(Err(err));
                        }
                        Ok(expr) => expr,
                    },
                };
                let new_part = self.pool.new_node(AstNode::Spread(expr));
                match parts {
                    None => {
                        parts = Some(new_part);
                    }
                    Some(prev_val) => {
                        parts = Some(self.pool.new_list_node(prev_val, new_part));
                    }
                }
            } else {
                let key = if self.parse_str_literal("[") {
                    self.parse_ws();
                    let key = match self.parse_expr() {
                        None => return Some(Err(self.get_err(ParseErrorType::NoExprInMapKey))),
                        Some(key) => match key {
                            Err(err) => return Some(Err(err)),
                            Ok(key) => key,
                        },
                    };
                    self.parse_ws();
                    if !self.parse_str_literal("]") {
                        return Some(Err(self.get_err(ParseErrorType::NoClosingBracketForMapKey)));
                    }
                    key
                } else if let Some(f_string) = self.parse_format_string() {
                    match f_string {
                        Err(err) => return Some(Err(err)),
                        Ok(f_string) => f_string,
                    }
                } else {
                    match self.parse_identifier(false) {
                        None => {
                            break;
                        }
                        Some(val) => val,
                    }
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
        self.pool.new_node(AstNode::SubString(
            std::str::from_utf8(
                &self.text.as_bytes()[self.idx + start_offset..self.idx + end_offset],
            )
            .unwrap(),
        ))
    }

    fn parse_format_string(&mut self) -> Option<Result<AstNodePtr, ParseError>> {
        if let Some(result) = self.inner_parse_format_string('\'' as u8) {
            return Some(result);
        }

        if let Some(result) = self.inner_parse_format_string('"' as u8) {
            return Some(result);
        }

        None
    }

    fn inner_parse_format_string(
        &mut self,
        quote_char: u8,
    ) -> Option<Result<AstNodePtr, ParseError>> {
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
            self.parse_ws();
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
            expr = self.pool.new_node(AstNode::AccessChain(expr, accessor));
        }

        Some(Ok(expr))
    }

    fn parse_base_expr(&mut self) -> Option<Result<AstNodePtr, ParseError>> {
        match self.parse_let_expr() {
            None => {}
            Some(expr) => match expr {
                Err(err) => return Some(Err(err)),
                Ok(expr) => return Some(Ok(expr)),
            },
        }

        if self.parse_str_literal(".") {
            let mut expr = self.pool.new_dot();
            self.parse_ws();
            if let Some(iden) = self.parse_identifier(true) {
                expr = self.pool.new_node(AstNode::AccessChain(expr, iden));
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

        if let Some(result) = self.parse_format_string() {
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
        if let Some(expr) = self.parse_identifier(true) {
            self.parse_ws();
            if self.parse_str_literal("(") {
                // This is a function call
                self.parse_ws();
                let mut args_node: Option<AstNodePtr> = None;
                loop {
                    let keyword = if self.parse_str_literal(":") {
                        self.parse_ws();
                        match self.parse_identifier(false) {
                            None => {
                                return Some(Err(
                                    self.get_err(ParseErrorType::NoIdentifierAfterKeywordArgument)
                                ));
                            }
                            Some(keyword) => {
                                if !self.parse_at_least_one_ws() {
                                    return Some(Err(self.get_err(
                                        ParseErrorType::NoWhitespaceAfterKeywordArgumentKeyword,
                                    )));
                                }
                                Some(keyword)
                            }
                        }
                    } else {
                        None
                    };

                    match self.parse_expr() {
                        None => {
                            break;
                        }
                        Some(expr) => match expr {
                            Err(err) => {
                                return Some(Err(err));
                            }
                            Ok(expr) => {
                                let expr = match keyword {
                                    None => expr,
                                    Some(keyword) => {
                                        self.pool.new_node(AstNode::KeywordArgument(keyword, expr))
                                    }
                                };
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

    fn parse_access_expr(&mut self) -> Option<Result<AstNodePtr, ParseError>> {
        if self.parse_str_literal("/") {
            self.parse_ws();
            match self.parse_expr() {
                None => Some(Err(self.get_err(ParseErrorType::NoExprReverseIndex))),
                Some(expr) => match expr {
                    Err(err) => Some(Err(err)),
                    Ok(expr) => Some(Ok(self.pool.new_node(AstNode::ReverseIdx(expr)))),
                },
            }
        } else {
            self.parse_expr()
        }
    }

    fn parse_access(&mut self) -> Option<Result<AstNodePtr, ParseError>> {
        if self.parse_str_literal(".") {
            self.parse_ws();
            let identifier = match self.parse_identifier(true) {
                None => {
                    return Some(Err(self.get_err(ParseErrorType::NoIdentifierAfterDotAccess)));
                }
                Some(val) => val,
            };

            Some(Ok(identifier))
        } else if self.parse_str_literal("[") {
            self.parse_ws();

            let start_expr = match self.parse_access_expr() {
                None => None,
                Some(expr) => match expr {
                    Err(err) => {
                        return Some(Err(err));
                    }
                    Ok(expr) => Some(expr),
                },
            };

            self.parse_ws();

            let access_expr = if self.parse_str_literal(":") {
                self.parse_ws();
                let end_expr = match self.parse_access_expr() {
                    None => None,
                    Some(expr) => match expr {
                        Err(err) => {
                            return Some(Err(err));
                        }
                        Ok(expr) => Some(expr),
                    },
                };
                self.parse_ws();
                self.pool
                    .new_node(AstNode::SliceAccess(start_expr, end_expr))
            } else {
                match start_expr {
                    None => {
                        return Some(Err(
                            self.get_err(ParseErrorType::NoExpressionForBracketAccess)
                        ));
                    }
                    Some(expr) => expr,
                }
            };

            if !self.parse_str_literal("]") {
                return Some(Err(
                    self.get_err(ParseErrorType::NoClosingBracketForBracketAccess)
                ));
            }

            Some(Ok(access_expr))
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
        self.inner_parse_expr(true)
    }

    fn inner_parse_expr(&mut self, allow_pipe: bool) -> Option<Result<AstNodePtr, ParseError>> {
        #[derive(Clone)]
        enum Op {
            Unary(UnaryOp),
            Binary(BinaryOp),
        }

        #[derive(Clone)]
        enum UnaryOp {
            Not,
            Negative,
        }

        #[derive(Clone)]
        enum BinaryOp {
            Pipe,
            Coalesce,
            Or,
            And,
            Equals,
            NotEquals,
            LessThan,
            LessThanOrEqual,
            GreaterThan,
            GreaterThanOrEqual,
            Add,
            Subtract,
            Multiply,
            Divide,
        }

        #[derive(PartialEq, PartialOrd, Clone, Copy)]
        enum OpOrder {
            End,
            Pipe,
            Coalesce,
            Or,
            And,
            Not,
            Equality,
            Add,
            Multiply,
            Negative,
        }

        enum Node {
            Expr(AstNodePtr),
            Op(Op, OpOrder),
        }

        let mut stack = Vec::<Node>::new();

        fn reduce_for_op_order(stack: &mut Vec<Node>, parser: &mut Parser, order: OpOrder) {
            // normal reduction
            // [a | b + c] |
            while stack.len() > 1 {
                if let (Node::Op(temp_op, temp_order), Node::Expr(right)) =
                    (&stack[stack.len() - 2], &stack[stack.len() - 1])
                {
                    let temp_order = *temp_order;
                    let right = *right;
                    if temp_order >= order {
                        match temp_op {
                            Op::Unary(temp_op) => {
                                let new_expr = match temp_op {
                                    UnaryOp::Not => parser.pool.new_node(AstNode::Not(right)),
                                    UnaryOp::Negative => {
                                        parser.pool.new_node(AstNode::Negative(right))
                                    }
                                };
                                stack.pop();
                                stack.pop();
                                stack.push(Node::Expr(new_expr));
                            }
                            Op::Binary(temp_op) => {
                                if let Node::Expr(left) = &stack[stack.len() - 3] {
                                    let left = *left;
                                    let new_expr = match temp_op {
                                        BinaryOp::Pipe => parser.pool.new_pipe(left, right),
                                        BinaryOp::Coalesce => {
                                            parser.pool.new_node(AstNode::Coalesce(left, right))
                                        }
                                        BinaryOp::Or => parser.pool.new_or(left, right),
                                        BinaryOp::And => parser.pool.new_and(left, right),
                                        BinaryOp::Equals => {
                                            parser.pool.new_node(AstNode::Equals(left, right))
                                        }
                                        BinaryOp::NotEquals => {
                                            parser.pool.new_node(AstNode::NotEquals(left, right))
                                        }
                                        BinaryOp::LessThan => {
                                            parser.pool.new_node(AstNode::LessThan(left, right))
                                        }
                                        BinaryOp::LessThanOrEqual => parser
                                            .pool
                                            .new_node(AstNode::LessThanOrEqual(left, right)),
                                        BinaryOp::GreaterThan => {
                                            parser.pool.new_node(AstNode::GreaterThan(left, right))
                                        }
                                        BinaryOp::GreaterThanOrEqual => parser
                                            .pool
                                            .new_node(AstNode::GreaterThanOrEqual(left, right)),
                                        BinaryOp::Add => parser.pool.new_add(left, right),
                                        BinaryOp::Subtract => parser.pool.new_subtract(left, right),
                                        BinaryOp::Multiply => {
                                            parser.pool.new_node(AstNode::Multiply(left, right))
                                        }
                                        BinaryOp::Divide => {
                                            parser.pool.new_node(AstNode::Divide(left, right))
                                        }
                                    };
                                    stack.pop();
                                    stack.pop();
                                    stack.pop();
                                    stack.push(Node::Expr(new_expr));
                                } else {
                                    panic!();
                                }
                            }
                        };
                    } else {
                        break;
                    }
                } else {
                    panic!()
                }
            }
        }

        fn parse_base_expr(
            stack: &mut Vec<Node>,
            parser: &mut Parser,
        ) -> Option<Result<(), ParseError>> {
            let mut has_unary_op = false;

            loop {
                if parser.parse_str_literal("not") {
                    parser.parse_ws();
                    stack.push(Node::Op(Op::Unary(UnaryOp::Not), OpOrder::Not));
                    has_unary_op = true;
                } else if parser.parse_str_literal("-") {
                    parser.parse_ws();
                    stack.push(Node::Op(Op::Unary(UnaryOp::Negative), OpOrder::Negative));
                    has_unary_op = true;
                } else {
                    break;
                }
            }

            let expr = match parser.parse_base_expr_with_accesses() {
                None => {
                    if has_unary_op {
                        return Some(Err(parser.get_err(ParseErrorType::NoExprAfterUnaryOperator)));
                    }
                    return None;
                }
                Some(expr) => match expr {
                    Err(err) => {
                        return Some(Err(err));
                    }
                    Ok(expr) => expr,
                },
            };
            stack.push(Node::Expr(expr));
            Some(Ok(()))
        }

        match parse_base_expr(&mut stack, self) {
            None => {
                return None;
            }
            Some(expr) => match expr {
                Err(err) => {
                    return Some(Err(err));
                }
                Ok(_) => {}
            },
        };

        loop {
            self.parse_ws();
            if let Some((next_op, next_order)) = if allow_pipe && self.parse_str_literal("|") {
                Some((BinaryOp::Pipe, OpOrder::Pipe))
            } else if self.parse_str_literal("??") {
                Some((BinaryOp::Coalesce, OpOrder::Coalesce))
            } else if self.parse_str_literal("or") {
                Some((BinaryOp::Or, OpOrder::Or))
            } else if self.parse_str_literal("and") {
                Some((BinaryOp::And, OpOrder::And))
            } else if self.parse_str_literal("==") {
                Some((BinaryOp::Equals, OpOrder::Equality))
            } else if self.parse_str_literal("!=") {
                Some((BinaryOp::NotEquals, OpOrder::Equality))
            } else if self.parse_str_literal("<=") {
                Some((BinaryOp::LessThanOrEqual, OpOrder::Equality))
            } else if self.parse_str_literal("<") {
                Some((BinaryOp::LessThan, OpOrder::Equality))
            } else if self.parse_str_literal(">=") {
                Some((BinaryOp::GreaterThanOrEqual, OpOrder::Equality))
            } else if self.parse_str_literal(">") {
                Some((BinaryOp::GreaterThan, OpOrder::Equality))
            } else if self.parse_str_literal("+") {
                Some((BinaryOp::Add, OpOrder::Add))
            } else if self.parse_str_literal("-") {
                Some((BinaryOp::Subtract, OpOrder::Add))
            } else if self.parse_str_literal("*") {
                Some((BinaryOp::Multiply, OpOrder::Multiply))
            } else if self.parse_str_literal("/") {
                Some((BinaryOp::Divide, OpOrder::Multiply))
            } else {
                break;
            } {
                self.parse_ws();

                reduce_for_op_order(&mut stack, self, next_order);
                stack.push(Node::Op(Op::Binary(next_op), next_order));

                match parse_base_expr(&mut stack, self) {
                    None => {
                        return Some(Err(self.get_err(ParseErrorType::NoExprAfterOperator)));
                    }
                    Some(expr) => match expr {
                        Err(err) => {
                            return Some(Err(err));
                        }
                        Ok(_) => {}
                    },
                };
            }
        }

        reduce_for_op_order(&mut stack, self, OpOrder::End);

        assert!(stack.len() == 1);

        if let Node::Expr(expr) = stack.pop().unwrap() {
            Some(Ok(expr))
        } else {
            panic!();
        }
    }

    fn parse_let_expr(&mut self) -> Option<Result<AstNodePtr, ParseError>> {
        if !self.parse_str_literal("let") {
            return None;
        }
        self.parse_ws();

        let identifier = match self.parse_identifier(true) {
            None => return Some(Err(self.get_err(ParseErrorType::NoIdentifierInLetStmt))),
            Some(res) => res,
        };
        self.parse_ws();

        if !self.parse_str_literal("=") {
            return Some(Err(self.get_err(ParseErrorType::NoEqualsInLetStmt)));
        }
        self.parse_ws();

        let expr = match self.inner_parse_expr(false) {
            None => return Some(Err(self.get_err(ParseErrorType::NoExprInLetStmt))),
            Some(expr) => match expr {
                Err(err) => return Some(Err(err)),
                Ok(expr) => expr,
            },
        };

        Some(Ok(self
            .pool
            .new_node(AstNode::LetStmt { identifier, expr })))
    }
}

#[derive(Debug)]
pub struct ParseError {
    idx: usize,
    typ: ParseErrorType,
}

impl ParseError {
    pub fn to_string(&self) -> String {
        format!("Parser error '{:?}' at index {}", self.typ, self.idx)
    }
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
    NoExprReverseIndex,
    NoExpressionAfterMapSpread,
    NoExprInMapKey,
    NoClosingBracketForMapKey,
    NoIdentifierAfterKeywordArgument,
    NoWhitespaceAfterKeywordArgumentKeyword,
    NoExprAfterUnaryOperator,
    NoIdentifierInLetStmt,
    NoEqualsInLetStmt,
    NoExprInLetStmt,
}
