use std::cmp::Ordering;

use super::ast_node_pool::{AstNode, AstNodePtr};
use super::obj_pool::{ObjPool, ObjPoolObjValue, ObjPoolRef, OrderedMap};
use super::parser::Parser;

pub struct Evaluator {
    obj_pool: ObjPool,
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {
            obj_pool: ObjPool::new(),
        }
    }

    pub fn parse_and_eval(&mut self, text: &str) -> ObjPoolRef {
        let mut parser = Parser::new(text);
        match parser.external_parse_expr() {
            None => self.obj_pool.new_null(),
            Some(ast) => match ast {
                Err(err) => {
                    println!("Parse Error: {:?}", err);
                    self.obj_pool.new_err("Parse Error")
                }
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

    fn eval_bool(&mut self, node: AstNodePtr, obj: ObjPoolRef, parser: &Parser) -> Option<bool> {
        let val = self.eval(node, obj, parser);
        match self.obj_pool.get(val) {
            ObjPoolObjValue::Bool(val) => Some(*val),
            _ => None,
        }
    }

    fn eval_bool_expr(
        &mut self,
        obj: ObjPoolRef,
        parser: &Parser,
        left: AstNodePtr,
        right: AstNodePtr,
        left_err: &str,
        right_err: &str,
        callback: impl Fn(bool, bool) -> bool,
    ) -> ObjPoolRef {
        let left_val = match self.eval_bool(left, obj, parser) {
            None => {
                return self.obj_pool.new_err(left_err);
            }
            Some(val) => val,
        };
        let right_val = match self.eval_bool(right, obj, parser) {
            None => {
                return self.obj_pool.new_err(right_err);
            }
            Some(val) => val,
        };

        self.obj_pool.new_bool(callback(left_val, right_val))
    }

    fn eval_f64_expr(
        &mut self,
        obj: ObjPoolRef,
        parser: &Parser,
        left: AstNodePtr,
        right: AstNodePtr,
        left_err: &str,
        right_err: &str,
        callback: impl Fn(f64, f64) -> f64,
    ) -> ObjPoolRef {
        let left_val = self.eval(left, obj, parser);
        let left_val = match self.obj_pool.get(left_val) {
            ObjPoolObjValue::Float64(val) => *val,
            _ => {
                return self.obj_pool.new_err(left_err);
            }
        };
        let right_val = self.eval(right, obj, parser);
        let right_val = match self.obj_pool.get(right_val) {
            ObjPoolObjValue::Float64(val) => *val,
            _ => {
                return self.obj_pool.new_err(right_err);
            }
        };
        self.obj_pool.new_f64(callback(left_val, right_val))
    }

    fn eval(&mut self, node: AstNodePtr, obj: ObjPoolRef, parser: &Parser) -> ObjPoolRef {
        match parser.get_node(node) {
            AstNode::Null => self.obj_pool.new_null(),
            AstNode::Pipe(left, right) => {
                let left_val = self.eval(*left, obj, parser);
                self.eval(*right, left_val, parser)
            }
            AstNode::Dot => obj,
            AstNode::Or(left, right) => self.eval_bool_expr(
                obj,
                parser,
                *left,
                *right,
                "Left side of OR has to be a boolean",
                "Right side of OR has to be a boolean",
                |left, right| left || right,
            ),
            AstNode::And(left, right) => self.eval_bool_expr(
                obj,
                parser,
                *left,
                *right,
                "Left side of OR has to be a boolean",
                "Right side of OR has to be a boolean",
                |left, right| left && right,
            ),
            AstNode::Equals(left, right) => {
                let left_val = self.eval(*left, obj, parser);
                let right_val = self.eval(*right, obj, parser);
                self.obj_pool
                    .new_bool(self.obj_pool.val_equals(left_val, right_val))
            }
            AstNode::NotEquals(left, right) => {
                let left_val = self.eval(*left, obj, parser);
                let right_val = self.eval(*right, obj, parser);
                self.obj_pool
                    .new_bool(!self.obj_pool.val_equals(left_val, right_val))
            }
            AstNode::LessThan(left, right) => {
                let left_val = self.eval(*left, obj, parser);
                let right_val = self.eval(*right, obj, parser);
                self.obj_pool
                    .new_bool(self.obj_pool.cmp_values(left_val, right_val) == Ordering::Less)
            }
            AstNode::LessThanOrEqual(left, right) => {
                let left_val = self.eval(*left, obj, parser);
                let right_val = self.eval(*right, obj, parser);
                self.obj_pool
                    .new_bool(self.obj_pool.cmp_values(left_val, right_val) != Ordering::Greater)
            }
            AstNode::GreaterThan(left, right) => {
                let left_val = self.eval(*left, obj, parser);
                let right_val = self.eval(*right, obj, parser);
                self.obj_pool
                    .new_bool(self.obj_pool.cmp_values(left_val, right_val) == Ordering::Greater)
            }
            AstNode::GreaterThanOrEqual(left, right) => {
                let left_val = self.eval(*left, obj, parser);
                let right_val = self.eval(*right, obj, parser);
                self.obj_pool
                    .new_bool(self.obj_pool.cmp_values(left_val, right_val) != Ordering::Less)
            }
            AstNode::Add(left, right) => self.eval_f64_expr(
                obj,
                parser,
                *left,
                *right,
                "Left side of addition has to be a float",
                "Right side of addition has to be a float",
                |left, right| left + right,
            ),
            AstNode::Subtract(left, right) => self.eval_f64_expr(
                obj,
                parser,
                *left,
                *right,
                "Left side of subtraction has to be a float",
                "Right side of subtraction has to be a float",
                |left, right| left - right,
            ),
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
                    match parser.get_node(node) {
                        AstNode::ListNode(left, right) => {
                            helper(this, obj, parser, map, *left);
                            helper(this, obj, parser, map, *right);
                        }
                        AstNode::MapKeyValPair { key, val } => {
                            let key_obj = match parser.get_node(*key) {
                                AstNode::SubString(key_name) => this.obj_pool.new_str(key_name),
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
            AstNode::ListLiteral(contents) => {
                let mut list = Vec::<ObjPoolRef>::new();

                fn helper(
                    this: &mut Evaluator,
                    obj: ObjPoolRef,
                    parser: &Parser,
                    list: &mut Vec<ObjPoolRef>,
                    node: AstNodePtr,
                ) {
                    match parser.get_node(node) {
                        AstNode::ListNode(left, right) => {
                            helper(this, obj, parser, list, *left);
                            helper(this, obj, parser, list, *right);
                        }
                        _ => {
                            let elem_val = this.eval(node, obj, parser);
                            list.push(elem_val);
                        }
                    }
                }

                match contents {
                    None => {}
                    Some(contents) => {
                        helper(self, obj, parser, &mut list, *contents);
                    }
                };

                self.obj_pool.new_list(list)
            }
            AstNode::FormatString(contents) => {
                let mut buffer = Vec::<u8>::new();
                fn helper(
                    this: &mut Evaluator,
                    obj: ObjPoolRef,
                    parser: &Parser,
                    buffer: &mut Vec<u8>,
                    node: AstNodePtr,
                ) {
                    match parser.get_node(node) {
                        AstNode::ListNode(left, right) => {
                            helper(this, obj, parser, buffer, *left);
                            helper(this, obj, parser, buffer, *right);
                        }
                        AstNode::SubString(text) => {
                            let text_bytes = text.as_bytes();
                            let mut idx = 0 as usize;
                            while idx < text_bytes.len() {
                                let ch = text_bytes[idx];
                                if ch as char == '\\' {
                                    match text_bytes[idx + 1] as char {
                                        'n' => {
                                            buffer.push('\n' as u8);
                                        }
                                        'r' => {
                                            buffer.push('\r' as u8);
                                        }
                                        't' => {
                                            buffer.push('\t' as u8);
                                        }
                                        '\\' => {
                                            buffer.push('\\' as u8);
                                        }
                                        '"' => {
                                            buffer.push('"' as u8);
                                        }
                                        '\'' => {
                                            buffer.push('\'' as u8);
                                        }
                                        '{' => {
                                            buffer.push('{' as u8);
                                        }
                                        '}' => {
                                            buffer.push('}' as u8);
                                        }
                                        _ => panic!(),
                                    }
                                    idx += 1;
                                } else {
                                    buffer.push(ch);
                                }
                                idx += 1;
                            }
                        }
                        _ => {
                            let elem_val = this.eval(node, obj, parser);
                            match this.obj_pool.get(elem_val) {
                                ObjPoolObjValue::String(sub_text) => {
                                    buffer.extend(sub_text.as_bytes());
                                }
                                _ => {
                                    this.write_val(elem_val, buffer, false).unwrap();
                                }
                            }
                        }
                    }
                }

                match contents {
                    None => {}
                    Some(contents) => {
                        helper(self, obj, parser, &mut buffer, *contents);
                    }
                };

                self.obj_pool
                    .new_str(std::str::from_utf8(buffer.as_slice()).unwrap())
            }
            AstNode::Access(expr) => match self.obj_pool.get(obj) {
                ObjPoolObjValue::Map(_) => {
                    let key_val = match parser.get_node(*expr) {
                        AstNode::SubString(key) => self.obj_pool.new_str(key),
                        _ => self.eval(*expr, obj, parser),
                    };
                    let map = match self.obj_pool.get(obj) {
                        ObjPoolObjValue::Map(map) => map,
                        _ => panic!(),
                    };
                    match map.get(&self.obj_pool, key_val) {
                        None => self.obj_pool.new_null(),
                        Some(val) => val,
                    }
                }
                ObjPoolObjValue::List(_) => {
                    let access_val = self.eval(*expr, obj, parser);
                    let access_idx = match self.obj_pool.get(access_val) {
                        ObjPoolObjValue::Float64(val) => {
                            if *val >= 0.0 && *val == val.floor() {
                                *val as usize
                            } else {
                                return self
                                    .obj_pool
                                    .new_err("List access must be positive integer");
                            }
                        }
                        _ => {
                            return self
                                .obj_pool
                                .new_err("List access must be a positive integer");
                        }
                    };
                    let list = match self.obj_pool.get(obj) {
                        ObjPoolObjValue::List(list) => list,
                        _ => panic!(),
                    };
                    if access_idx < list.len() {
                        list[access_idx]
                    } else {
                        self.obj_pool.new_err("List access out of bounds")
                    }
                }
                ObjPoolObjValue::Null => obj,
                _ => panic!(),
            },
            AstNode::Bool(val) => self.obj_pool.new_bool(*val),
            AstNode::FcnCall { name, args } => {
                let name = match parser.get_node(*name) {
                    AstNode::SubString(name) => *name,
                    _ => panic!(),
                };

                let mut args_vec = Vec::<AstNodePtr>::new();
                fn helper(
                    obj: ObjPoolRef,
                    parser: &Parser,
                    args_vec: &mut Vec<AstNodePtr>,
                    node: AstNodePtr,
                ) {
                    match parser.get_node(node) {
                        AstNode::ListNode(left, right) => {
                            helper(obj, parser, args_vec, *left);
                            helper(obj, parser, args_vec, *right);
                        }
                        _ => {
                            args_vec.push(node);
                        }
                    }
                }

                match args {
                    None => {}
                    Some(args) => {
                        helper(obj, parser, &mut args_vec, *args);
                    }
                };

                self.eval_fcn(parser, obj, name, args_vec)
            }
            _ => panic!("Unimplemented {:?}", parser.get_node(node)),
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

    pub fn eval_fcn(
        &mut self,
        parser: &Parser,
        obj: ObjPoolRef,
        name: &str,
        args: Vec<AstNodePtr>,
    ) -> ObjPoolRef {
        match name {
            "len" => {
                if args.len() != 0 {
                    return self
                        .obj_pool
                        .new_err("len() must be called with 0 arguments.");
                }
                match self.obj_pool.get(obj) {
                    ObjPoolObjValue::List(val) => self.obj_pool.new_f64(val.len() as f64),
                    ObjPoolObjValue::Map(val) => self.obj_pool.new_f64(val.len() as f64),
                    _ => self
                        .obj_pool
                        .new_err("len() can only be called on a list or map"),
                }
            }
            "map" => {
                if args.len() != 1 {
                    return self
                        .obj_pool
                        .new_err("map() must be called with 1 argument.");
                }
                match self.obj_pool.get(obj) {
                    ObjPoolObjValue::List(val) => {
                        // TODO: figure out how to not clone here
                        let val = val.clone();
                        let mut result = Vec::<ObjPoolRef>::with_capacity(val.len());
                        for elem in val {
                            result.push(self.eval(args[0], elem, parser));
                        }
                        self.obj_pool.new_list(result)
                    }
                    _ => self.obj_pool.new_err("map() must be called on a list"),
                }
            }
            "group" => {
                if args.len() != 1 {
                    return self
                        .obj_pool
                        .new_err("group() must be called with one argument");
                }
                match self.obj_pool.get(obj) {
                    ObjPoolObjValue::List(val) => {
                        let mut groups = Vec::<(ObjPoolRef, Vec<ObjPoolRef>)>::new();
                        let val = val.clone();
                        // let mut result = Vec::<ObjPoolRef>::with_capacity(val.len());
                        for elem in val {
                            let key_val = self.eval(args[0], elem, parser);
                            let mut found = false;
                            for (loop_key_val, loop_vals) in &mut groups {
                                if self.obj_pool.val_equals(key_val, *loop_key_val) {
                                    loop_vals.push(elem);
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                groups.push((key_val, vec![elem]));
                            }
                        }

                        let mut result = Vec::<ObjPoolRef>::with_capacity(groups.len());
                        for (key, rows) in groups {
                            let key_label = self.obj_pool.new_str("key");
                            let rows_label = self.obj_pool.new_str("rows");
                            let rows_obj = self.obj_pool.new_list(rows);
                            result.push(self.obj_pool.new_map_from_iter(
                                [(key_label, key), (rows_label, rows_obj)].iter(),
                            ));
                        }
                        self.obj_pool.new_list(result)
                    }
                    _ => self.obj_pool.new_err("group() must be called on a list"),
                }
            }
            "sort" => {
                if args.len() != 1 {
                    return self
                        .obj_pool
                        .new_err("sort() must be called with one argument");
                }
                match self.obj_pool.get(obj) {
                    ObjPoolObjValue::List(val) => {
                        let val = val.clone();
                        let mut values = Vec::<(ObjPoolRef, ObjPoolRef)>::with_capacity(val.len());
                        for elem in val {
                            values.push((elem, self.eval(args[0], elem, parser)));
                        }
                        values.sort_by(|(_, left), (_, right)| {
                            self.obj_pool.cmp_values(*left, *right)
                        });

                        let result = values.iter().map(|(val, _)| *val).collect::<Vec<_>>();

                        self.obj_pool.new_list(result)
                    }
                    _ => self.obj_pool.new_err("sort() must be called on a list"),
                }
            }
            "filter" => {
                if args.len() != 1 {
                    return self
                        .obj_pool
                        .new_err("filter() must be called with one argument");
                }
                match self.obj_pool.get(obj) {
                    ObjPoolObjValue::List(val) => {
                        let val = val.clone();
                        let mut result = Vec::<ObjPoolRef>::new();
                        for elem in val {
                            let filter_val = self.eval(args[0], elem, parser);
                            match self.obj_pool.get(filter_val) {
                                ObjPoolObjValue::Bool(bool_val) => {
                                    if *bool_val {
                                        result.push(elem);
                                    }
                                }
                                _ => {}
                            }
                        }

                        self.obj_pool.new_list(result)
                    }
                    _ => self.obj_pool.new_err("filter() must be called on a list"),
                }
            }
            _ => self
                .obj_pool
                .new_err(format!("Unknown function \"{}\"", name).as_str()),
        }
    }
}
