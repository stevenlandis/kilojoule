use super::ast_node_pool::{AstNode, AstNodePool, AstNodePtr};
use super::obj_pool::{ObjPool, ObjPoolObjValue, ObjPoolRef, OrderedMap};
use std::collections::HashMap;

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

        if let Some(expr) = self.parse_integer() {
            return Some(Ok(expr));
        }
        if self.parse_str_literal("null") {
            return Some(Ok(self.pool.new_null()));
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
        let expr = match self.parse_base_expr() {
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
    NoColonInMapLiteral,
    NoMapLiteralValue,
    NoMapLiteralEndingBrace,
}

pub struct Evaluator {
    obj_pool: ObjPool,
    var_stack: Vec<HashMap<String, ObjPoolRef>>,
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {
            obj_pool: ObjPool::new(),
            var_stack: Vec::new(),
        }
    }

    pub fn parse_and_eval(&mut self, text: &str) -> ObjPoolRef {
        let mut parser = Parser::new(text);
        match parser.parse_expr() {
            None => self.obj_pool.new_null(),
            Some(ast) => match ast {
                Err(_) => self.obj_pool.new_err("Parse Error"),
                Ok(ast) => {
                    // for (idx, val) in parser.pool.vals.iter().enumerate() {
                    //     println!("{}: {:?}", idx, val);
                    // }
                    let val = self.obj_pool.new_null();
                    self.eval(ast, val, &parser)
                }
            },
        }
    }

    fn eval(&mut self, node: AstNodePtr, obj: ObjPoolRef, parser: &Parser) -> ObjPoolRef {
        match parser.pool.get(node) {
            AstNode::Null => self.obj_pool.new_null(),
            AstNode::Pipe(left, right) => {
                let left_val = self.eval(*left, obj, parser);
                self.eval(*right, left_val, parser)
            }
            AstNode::Dot => obj,
            AstNode::Add(left, right) => {
                let left_val = self.eval(*left, obj, parser);
                let left_val = match self.obj_pool.get(left_val) {
                    ObjPoolObjValue::Float64(val) => *val,
                    _ => {
                        return self
                            .obj_pool
                            .new_err("Left side of addition has to be a float");
                    }
                };
                let right_val = self.eval(*right, obj, parser);
                let right_val = match self.obj_pool.get(right_val) {
                    ObjPoolObjValue::Float64(val) => val,
                    _ => {
                        return self
                            .obj_pool
                            .new_err("Right side of addition has to be a float");
                    }
                };
                self.obj_pool.new_f64(left_val + right_val)
            }
            AstNode::Integer(val) => self.obj_pool.new_f64(*val as f64),
            AstNode::MapLiteral(contents) => {
                let mut map = OrderedMap::new();
                fn helper(
                    this: &mut Evaluator,
                    obj: ObjPoolRef,
                    parser: &Parser,
                    map: &mut OrderedMap,
                    node: AstNodePtr,
                ) {
                    match parser.pool.get(node) {
                        AstNode::ListNode(left, right) => {
                            helper(this, obj, parser, map, *left);
                            helper(this, obj, parser, map, *right);
                        }
                        AstNode::MapKeyValPair { key, val } => {
                            let key_obj = match parser.pool.get(*key) {
                                AstNode::Identifier(key_name) => this.obj_pool.new_str(key_name),
                                _ => panic!(),
                            };
                            let val_obj = this.eval(*val, obj, parser);
                            map.insert(&this.obj_pool, key_obj, val_obj);
                        }
                        _ => panic!(),
                    }
                }

                match contents {
                    None => {}
                    Some(contents) => {
                        helper(self, obj, parser, &mut map, *contents);
                    }
                };

                self.obj_pool.new_map(map)
            }
            AstNode::Access(expr) => match &self.obj_pool.get(obj) {
                ObjPoolObjValue::Map(_) => {
                    let key_val = match parser.pool.get(*expr) {
                        AstNode::Identifier(key) => self.obj_pool.new_str(key),
                        _ => panic!(),
                    };
                    let map = match &self.obj_pool.get(obj) {
                        ObjPoolObjValue::Map(map) => map,
                        _ => panic!(),
                    };
                    match map.get(&self.obj_pool, key_val) {
                        None => self.obj_pool.new_null(),
                        Some(val) => val,
                    }
                }
                _ => panic!(),
            },
            _ => panic!("Unimplemented {:?}", parser.pool.get(node)),
        }
    }

    pub fn write_val(
        &self,
        val: ObjPoolRef,
        writer: &mut impl std::io::Write,
        use_indent: bool,
    ) -> std::io::Result<()> {
        match self.obj_pool.write_to_str(writer, val, 0, use_indent) {
            Err(err) => Err(err),
            Ok(_) => Ok(()),
        }
    }
}
