use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use super::ast_node_pool::{AstNode, AstNodePtr};
// use super::obj_pool::{ObjPool, ObjPoolObjValue, Val, OrderedMap};
use super::parser::Parser;
use super::val::{OrderedMap, Val, ValType};

pub struct Evaluator {}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {}
    }

    pub fn parse_and_eval(&mut self, text: &str) -> Val {
        let mut parser = Parser::new(text);
        match parser.external_parse_expr() {
            None => Val::new_null(),
            Some(ast) => match ast {
                Err(err) => {
                    println!("Parse Error: {:?}", err);
                    Val::new_err("Parse Error")
                }
                Ok(ast) => {
                    // for (idx, val) in parser.pool.vals.iter().enumerate() {
                    //     println!("{}: {:?}", idx, val);
                    // }
                    let val = Val::new_null();
                    self.eval(ast, &val, &parser)
                }
            },
        }
    }

    fn eval_bool(&mut self, node: AstNodePtr, obj: &Val, parser: &Parser) -> Option<bool> {
        let val = self.eval(node, obj, parser);
        match val.get_val() {
            ValType::Bool(val) => Some(*val),
            _ => None,
        }
    }

    fn eval_bool_expr(
        &mut self,
        obj: &Val,
        parser: &Parser,
        left: AstNodePtr,
        right: AstNodePtr,
        left_err: &str,
        right_err: &str,
        callback: impl Fn(bool, bool) -> bool,
    ) -> Val {
        let left_val = match self.eval_bool(left, obj, parser) {
            None => {
                return Val::new_err(left_err);
            }
            Some(val) => val,
        };
        let right_val = match self.eval_bool(right, obj, parser) {
            None => {
                return Val::new_err(right_err);
            }
            Some(val) => val,
        };

        Val::new_bool(callback(left_val, right_val))
    }

    fn eval_f64_expr(
        &mut self,
        obj: &Val,
        parser: &Parser,
        left: AstNodePtr,
        right: AstNodePtr,
        left_err: &str,
        right_err: &str,
        callback: impl Fn(f64, f64) -> f64,
    ) -> Val {
        let left_val = self.eval(left, obj, parser);
        let left_val = match left_val.get_val() {
            ValType::Float64(val) => *val,
            _ => {
                return Val::new_err(left_err);
            }
        };
        let right_val = self.eval(right, obj, parser);
        let right_val = match right_val.get_val() {
            ValType::Float64(val) => *val,
            _ => {
                return Val::new_err(right_err);
            }
        };
        Val::new_f64(callback(left_val, right_val))
    }

    fn eval(&mut self, node: AstNodePtr, obj: &Val, parser: &Parser) -> Val {
        match parser.get_node(node) {
            AstNode::Null => Val::new_null(),
            AstNode::Pipe(left, right) => {
                let left_val = self.eval(*left, obj, parser);
                self.eval(*right, &left_val, parser)
            }
            AstNode::Dot => obj.clone(),
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
                Val::new_bool(left_val == right_val)
            }
            AstNode::NotEquals(left, right) => {
                let left_val = self.eval(*left, obj, parser);
                let right_val = self.eval(*right, obj, parser);
                Val::new_bool(left_val != right_val)
            }
            AstNode::LessThan(left, right) => {
                let left_val = self.eval(*left, obj, parser);
                let right_val = self.eval(*right, obj, parser);
                Val::new_bool(left_val < right_val)
            }
            AstNode::LessThanOrEqual(left, right) => {
                let left_val = self.eval(*left, obj, parser);
                let right_val = self.eval(*right, obj, parser);
                Val::new_bool(left_val <= right_val)
            }
            AstNode::GreaterThan(left, right) => {
                let left_val = self.eval(*left, obj, parser);
                let right_val = self.eval(*right, obj, parser);
                Val::new_bool(left_val > right_val)
            }
            AstNode::GreaterThanOrEqual(left, right) => {
                let left_val = self.eval(*left, obj, parser);
                let right_val = self.eval(*right, obj, parser);
                Val::new_bool(left_val >= right_val)
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
            AstNode::Multiply(left, right) => self.eval_f64_expr(
                obj,
                parser,
                *left,
                *right,
                "Left side of multiplication has to be a float",
                "Right side of multiplication has to be a float",
                |left, right| left * right,
            ),
            AstNode::Divide(left, right) => {
                let left_val = self.eval(*left, obj, parser);
                let left_val = match left_val.get_val() {
                    ValType::Float64(val) => *val,
                    _ => {
                        return Val::new_err("Left side of division has to be a float");
                    }
                };
                let right_val = self.eval(*right, obj, parser);
                let right_val = match right_val.get_val() {
                    ValType::Float64(val) => *val,
                    _ => {
                        return Val::new_err("Right side of division has to be a float");
                    }
                };
                if right_val == 0.0 {
                    Val::new_err("divide by zero")
                } else {
                    Val::new_f64(left_val / right_val)
                }
            }
            AstNode::Integer(val) => Val::new_f64(*val as f64),
            AstNode::MapLiteral(contents) => {
                let mut map = OrderedMap::new();
                fn helper(
                    this: &mut Evaluator,
                    obj: &Val,
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
                                AstNode::SubString(key_name) => Val::new_str(key_name),
                                _ => panic!(),
                            };
                            let val_obj = this.eval(*val, obj, parser);
                            map.insert(&key_obj, &val_obj);
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

                Val::new_map(map)
            }
            AstNode::ListLiteral(contents) => {
                let mut list = Vec::<Val>::new();

                fn helper(
                    this: &mut Evaluator,
                    obj: &Val,
                    parser: &Parser,
                    list: &mut Vec<Val>,
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

                Val::new_list(list)
            }
            AstNode::FormatString(contents) => {
                let mut buffer = Vec::<u8>::new();
                fn helper(
                    this: &mut Evaluator,
                    obj: &Val,
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
                            match elem_val.get_val() {
                                ValType::String(sub_text) => {
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

                Val::new_str(std::str::from_utf8(buffer.as_slice()).unwrap())
            }
            AstNode::Access(expr) => match obj.get_val() {
                ValType::Map(map) => {
                    let key_val = match parser.get_node(*expr) {
                        AstNode::SubString(key) => Val::new_str(key),
                        _ => self.eval(*expr, obj, parser),
                    };

                    match map.get(&key_val) {
                        None => Val::new_null(),
                        Some(val) => val,
                    }
                }
                ValType::List(_) => {
                    let access_val = self.eval(*expr, obj, parser);
                    let access_idx = match access_val.get_val() {
                        ValType::Float64(val) => {
                            if *val >= 0.0 && *val == val.floor() {
                                *val as usize
                            } else {
                                return Val::new_err("List access must be positive integer");
                            }
                        }
                        _ => {
                            return Val::new_err("List access must be a positive integer");
                        }
                    };
                    let list = match obj.get_val() {
                        ValType::List(list) => list,
                        _ => panic!(),
                    };
                    if access_idx < list.len() {
                        list[access_idx].clone()
                    } else {
                        Val::new_err("List access out of bounds")
                    }
                }
                ValType::Null => obj.clone(),
                _ => panic!(),
            },
            AstNode::Bool(val) => Val::new_bool(*val),
            AstNode::FcnCall { name, args } => {
                let name = match parser.get_node(*name) {
                    AstNode::SubString(name) => *name,
                    _ => panic!(),
                };

                let mut args_vec = Vec::<AstNodePtr>::new();
                fn helper(
                    obj: &Val,
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
        val: Val,
        writer: &mut impl std::io::Write,
        use_indent: bool,
    ) -> std::io::Result<()> {
        match val.write_to_str(writer, 0, use_indent) {
            Err(err) => Err(err),
            Ok(_) => Ok(()),
        }
    }

    pub fn eval_fcn(
        &mut self,
        parser: &Parser,
        obj: &Val,
        name: &str,
        args: Vec<AstNodePtr>,
    ) -> Val {
        match name {
            "len" => {
                if args.len() != 0 {
                    return Val::new_err("len() must be called with 0 arguments.");
                }
                match obj.get_val() {
                    ValType::List(val) => Val::new_f64(val.len() as f64),
                    ValType::Map(val) => Val::new_f64(val.len() as f64),
                    _ => Val::new_err("len() can only be called on a list or map"),
                }
            }
            "map" => {
                if args.len() != 1 {
                    return Val::new_err("map() must be called with 1 argument.");
                }
                match obj.get_val() {
                    ValType::List(val) => {
                        let mut result = Vec::<Val>::with_capacity(val.len());
                        for elem in val {
                            result.push(self.eval(args[0], &elem, parser));
                        }
                        Val::new_list(result)
                    }
                    _ => Val::new_err("map() must be called on a list"),
                }
            }
            "group" => {
                if args.len() != 1 {
                    return Val::new_err("group() must be called with one argument");
                }

                match obj.get_val() {
                    ValType::List(val) => {
                        let mut groups = Vec::<(Val, Vec<Val>)>::new();
                        let mut key_to_group_idx = HashMap::<Val, usize>::new();

                        for elem in val {
                            let key_val = self.eval(args[0], elem, parser);
                            match key_to_group_idx.entry(key_val.clone()) {
                                Entry::Occupied(entry) => {
                                    groups[*entry.get()].1.push(elem.clone());
                                }
                                Entry::Vacant(entry) => {
                                    entry.insert(groups.len());
                                    groups.push((key_val, vec![elem.clone()]));
                                }
                            }
                        }

                        let mut result = Vec::<Val>::with_capacity(groups.len());
                        for (key, vals) in groups {
                            result.push(Val::new_map(OrderedMap::from_kv_pair_slice(&[
                                (Val::new_str("key"), key),
                                (Val::new_str("vals"), Val::new_list(vals)),
                            ])))
                        }
                        Val::new_list(result)
                    }
                    _ => Val::new_err("group() must be called on a list"),
                }
            }
            "sort" => {
                if args.len() != 1 {
                    return Val::new_err("sort() must be called with one argument");
                }
                match obj.get_val() {
                    ValType::List(val) => {
                        let mut values = val.clone();
                        values.sort_by_cached_key(|elem| self.eval(args[0], elem, parser));

                        Val::new_list(values)
                    }
                    _ => Val::new_err("sort() must be called on a list"),
                }
            }
            "filter" => {
                if args.len() != 1 {
                    return Val::new_err("filter() must be called with one argument");
                }
                match obj.get_val() {
                    ValType::List(val) => {
                        let mut result = Vec::<Val>::new();
                        for elem in val {
                            let filter_val = self.eval(args[0], &elem, parser);
                            match filter_val.get_val() {
                                ValType::Bool(bool_val) => {
                                    if *bool_val {
                                        result.push(elem.clone());
                                    }
                                }
                                _ => {}
                            }
                        }

                        Val::new_list(result)
                    }
                    _ => Val::new_err("filter() must be called on a list"),
                }
            }
            _ => Val::new_err(format!("Unknown function \"{}\"", name).as_str()),
        }
    }
}
