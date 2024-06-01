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

#[derive(Clone, Copy)]
pub struct ObjPoolRef {
    idx: usize,
}

enum ObjPoolObjValue {
    Null,
    Err(String),
    Float64(f64),
    List(Vec<ObjPoolRef>),
}

struct ObjPoolObj {
    ref_count: usize,
    value: ObjPoolObjValue,
}

struct ObjPool {
    vals: Vec<ObjPoolObj>,
}

impl ObjPool {
    fn new() -> Self {
        ObjPool { vals: Vec::new() }
    }

    fn new_null(&mut self) -> ObjPoolRef {
        let idx = self.vals.len();
        self.vals.push(ObjPoolObj {
            ref_count: 0,
            value: ObjPoolObjValue::Null,
        });
        ObjPoolRef { idx }
    }

    fn new_err(&mut self, msg: &str) -> ObjPoolRef {
        let idx = self.vals.len();
        self.vals.push(ObjPoolObj {
            ref_count: 0,
            value: ObjPoolObjValue::Err(msg.to_string()),
        });
        ObjPoolRef { idx }
    }

    fn new_f64(&mut self, val: f64) -> ObjPoolRef {
        let idx = self.vals.len();
        self.vals.push(ObjPoolObj {
            ref_count: 0,
            value: ObjPoolObjValue::Float64(val),
        });
        ObjPoolRef { idx }
    }

    fn new_list(&mut self) -> ObjPoolRef {
        let idx = self.vals.len();
        self.vals.push(ObjPoolObj {
            ref_count: 0,
            value: ObjPoolObjValue::List(Vec::new()),
        });
        ObjPoolRef { idx }
    }

    fn get_f64(&self, obj: ObjPoolRef) -> f64 {
        match self.vals[obj.idx].value {
            ObjPoolObjValue::Float64(val) => val,
            _ => panic!(),
        }
    }

    fn get_list(&self, obj: ObjPoolRef) -> &[ObjPoolRef] {
        match &self.vals[obj.idx].value {
            ObjPoolObjValue::List(list) => list.as_slice(),
            _ => panic!(),
        }
    }

    fn incr_ref(&mut self, obj: ObjPoolRef) {
        self.vals[obj.idx].ref_count += 1;
    }

    fn decr_ref(&mut self, obj: ObjPoolRef) {
        assert!(self.vals[obj.idx].ref_count > 0);
        self.vals[obj.idx].ref_count -= 1;
    }

    fn collect_garbage(&mut self) {
        while self.vals.len() > 0 && self.vals[self.vals.len() - 1].ref_count == 0 {
            let top = self.vals.pop().unwrap();
            match top.value {
                ObjPoolObjValue::Null => {}
                ObjPoolObjValue::Err(_) => {}
                ObjPoolObjValue::Float64(_) => {}
                ObjPoolObjValue::List(val) => {
                    for elem in val {
                        self.decr_ref(elem);
                    }
                }
            }
        }
    }

    fn list_push(&mut self, obj: ObjPoolRef) {
        match &mut self.vals[obj.idx].value {
            ObjPoolObjValue::List(val) => {
                val.push(obj);
                self.incr_ref(obj);
            }
            _ => panic!(),
        }
    }

    fn write_json_escaped_str(
        &self,
        writer: &mut impl std::io::Write,
        val: &str,
    ) -> std::io::Result<usize> {
        writer.write("\"".as_bytes())?;
        writer.write(val.as_bytes())?;
        writer.write("\"".as_bytes())?;

        Ok(0)
    }

    fn inner_write_str(
        &self,
        writer: &mut impl std::io::Write,
        val: ObjPoolRef,
        indent: u64,
        use_indent: bool,
    ) -> std::io::Result<usize> {
        fn write_indent(writer: &mut impl std::io::Write, indent: u64) -> std::io::Result<usize> {
            for _ in 0..indent {
                writer.write("  ".as_bytes())?;
            }
            Ok(0)
        }

        match &self.vals[val.idx].value {
            ObjPoolObjValue::Null => {
                writer.write("null".as_bytes())?;
            }
            ObjPoolObjValue::Float64(val) => {
                // TODO: Don't allocate on every float write
                writer.write(val.to_string().as_str().as_bytes())?;
            }
            ObjPoolObjValue::Err(val) => {
                writer.write("{\"ERROR\":".as_bytes())?;
                self.write_json_escaped_str(writer, val.as_str())?;
                writer.write("}".as_bytes())?;
            }
            ObjPoolObjValue::List(val) => {
                writer.write("[".as_bytes())?;
                for (idx, elem) in val.iter().enumerate() {
                    if idx > 0 {
                        if use_indent {
                            writer.write(", ".as_bytes())?;
                        } else {
                            writer.write(",".as_bytes())?;
                        }
                    }
                    if use_indent {
                        writer.write("\n".as_bytes())?;
                        write_indent(writer, indent + 1)?;
                    }
                    self.inner_write_str(writer, *elem, indent + 1, use_indent)?;
                }
                if val.len() > 0 && use_indent {
                    writer.write("\n".as_bytes())?;
                    write_indent(writer, indent)?;
                }
                writer.write("]".as_bytes())?;
            }
        }
        Ok(0)
    }
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
        let ast = parser.parse_expr().unwrap().unwrap();
        let val = self.obj_pool.new_null();
        self.eval(ast, val, &parser)
    }

    fn eval(&mut self, node: AstNodePtr, obj: ObjPoolRef, parser: &Parser) -> ObjPoolRef {
        match parser.pool.vals[node] {
            AstNode::Null => self.obj_pool.new_null(),
            AstNode::Pipe(left, right) => {
                let left_val = self.eval(left, obj, parser);
                self.eval(right, left_val, parser)
            }
            AstNode::Dot => obj,
            AstNode::Add(left, right) => {
                let left_val = self.eval(left, obj, parser);
                let left_val = match self.obj_pool.vals[left_val.idx].value {
                    ObjPoolObjValue::Float64(val) => val,
                    _ => {
                        return self
                            .obj_pool
                            .new_err("Left side of addition has to be a float");
                    }
                };
                let right_val = self.eval(right, obj, parser);
                let right_val = match self.obj_pool.vals[right_val.idx].value {
                    ObjPoolObjValue::Float64(val) => val,
                    _ => {
                        return self
                            .obj_pool
                            .new_err("Right side of addition has to be a float");
                    }
                };
                self.obj_pool.new_f64(left_val + right_val)
            }
            AstNode::Integer(val) => self.obj_pool.new_f64(val as f64),
            _ => panic!(),
        }
    }

    pub fn write_val(
        &self,
        val: ObjPoolRef,
        writer: &mut impl std::io::Write,
        use_indent: bool,
    ) -> std::io::Result<()> {
        match self.obj_pool.inner_write_str(writer, val, 0, use_indent) {
            Err(err) => Err(err),
            Ok(_) => Ok(()),
        }
    }
}
