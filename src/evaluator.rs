use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};

use super::ast_node_pool::{AstNode, AstNodePtr};
use super::parser::Parser;
use super::val::{OrderedMap, Val, ValType};
use std::process::{Command, Stdio};

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
            AstNode::Coalesce(left, right) => {
                let left = self.eval(*left, obj, parser);
                match &left.get_val() {
                    ValType::Null => self.eval(*right, obj, parser),
                    _ => left,
                }
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
            AstNode::Not(expr) => {
                let left_val = match self.eval_bool(*expr, obj, parser) {
                    None => {
                        return Val::new_err("\"not\" operator has to be called on a boolean");
                    }
                    Some(val) => val,
                };

                Val::new_bool(!left_val)
            }
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
            AstNode::Negative(expr) => {
                let val = self.eval(*expr, obj, parser);
                let val = match val.get_val() {
                    ValType::Float64(val) => *val,
                    _ => {
                        return Val::new_err("Negative expression must be a number");
                    }
                };
                Val::new_f64(-val)
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
                ) -> Result<(), Val> {
                    match parser.get_node(node) {
                        AstNode::ListNode(left, right) => {
                            helper(this, obj, parser, map, *left)?;
                            helper(this, obj, parser, map, *right)?;
                            Ok(())
                        }
                        AstNode::MapKeyValPair { key, val } => {
                            let key_obj = match parser.get_node(*key) {
                                AstNode::SubString(key_name) => Val::new_str(key_name),
                                _ => this.eval(*key, obj, parser),
                            };
                            let val_obj = this.eval(*val, obj, parser);
                            map.insert(&key_obj, &val_obj);
                            Ok(())
                        }
                        AstNode::Spread(spread) => {
                            let spread_val = this.eval(*spread, obj, parser);
                            match spread_val.get_val() {
                                ValType::Map(spread_val) => {
                                    for (key, val) in spread_val.get_kv_pair_slice() {
                                        map.insert(key, val);
                                    }
                                }
                                _ => {
                                    return Err(Val::new_err("can only spread a map"));
                                }
                            }
                            Ok(())
                        }
                        _ => panic!(),
                    }
                }

                match contents {
                    None => {}
                    Some(contents) => {
                        match helper(self, obj, parser, &mut map, *contents) {
                            Ok(_) => {}
                            Err(err) => return err,
                        };
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
                ) -> Result<(), Val> {
                    match parser.get_node(node) {
                        AstNode::ListNode(left, right) => {
                            helper(this, obj, parser, list, *left)?;
                            helper(this, obj, parser, list, *right)?;
                            Ok(())
                        }
                        AstNode::Spread(spread) => {
                            let spread_list = this.eval(*spread, obj, parser);
                            match spread_list.get_val() {
                                ValType::List(spread_list) => {
                                    for elem in spread_list {
                                        list.push(elem.clone());
                                    }

                                    Ok(())
                                }
                                _ => {
                                    return Err(Val::new_err("Can only spread lists"));
                                }
                            }
                        }
                        _ => {
                            let elem_val = this.eval(node, obj, parser);
                            list.push(elem_val);
                            Ok(())
                        }
                    }
                }

                match contents {
                    None => {}
                    Some(contents) => {
                        match helper(self, obj, parser, &mut list, *contents) {
                            Ok(_) => {}
                            Err(err) => {
                                return err;
                            }
                        };
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
                    if let AstNode::ReverseIdx(_) = parser.get_node(*expr) {
                        return Val::new_err("Maps cannot be accessed with a reverse index");
                    }

                    let key_val = match parser.get_node(*expr) {
                        AstNode::SubString(key) => Val::new_str(key),
                        _ => self.eval(*expr, obj, parser),
                    };

                    match map.get(&key_val) {
                        None => Val::new_null(),
                        Some(val) => val,
                    }
                }
                ValType::List(list) => match parser.get_node(*expr) {
                    AstNode::SliceAccess(start, end) => {
                        let start_idx = match start {
                            None => 0,
                            Some(start_expr) => {
                                match self.eval_list_access(obj, *start_expr, parser) {
                                    Err(err) => {
                                        return err;
                                    }
                                    Ok((start_idx, is_rev)) => {
                                        if is_rev {
                                            list.len().saturating_sub(start_idx)
                                        } else {
                                            start_idx.min(list.len())
                                        }
                                    }
                                }
                            }
                        };
                        let end_idx = match end {
                            None => list.len(),
                            Some(end_expr) => match self.eval_list_access(obj, *end_expr, parser) {
                                Err(err) => {
                                    return err;
                                }
                                Ok((end_idx, is_rev)) => {
                                    if is_rev {
                                        list.len().saturating_sub(end_idx)
                                    } else {
                                        end_idx.min(list.len())
                                    }
                                }
                            },
                        };
                        let end_idx = end_idx.max(start_idx);
                        Val::new_list(list[start_idx..end_idx].to_vec())
                    }
                    _ => match self.eval_list_access(obj, *expr, parser) {
                        Err(err) => err,
                        Ok((idx, is_rev)) => {
                            if idx < list.len() {
                                let idx = if is_rev { list.len() - idx - 1 } else { idx };
                                list[idx].clone()
                            } else {
                                Val::new_err("List access out of bounds")
                            }
                        }
                    },
                },
                ValType::Null => obj.clone(),
                _ => Val::new_err("Invalid access"),
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

                self.eval_fcn(parser, obj, name, &args_vec)
            }
            _ => panic!("Unimplemented {:?}", parser.get_node(node)),
        }
    }

    fn eval_list_access(
        &mut self,
        obj: &Val,
        expr: AstNodePtr,
        parser: &Parser,
    ) -> Result<(usize, bool), Val> {
        let (is_rev, idx_expr) = match parser.get_node(expr) {
            AstNode::ReverseIdx(rev_expr) => (true, *rev_expr),
            _ => (false, expr),
        };
        let idx = self.eval(idx_expr, obj, parser);
        match idx.get_val() {
            ValType::Float64(num) => {
                let num = *num;
                if num == num.floor() && num >= 0.0 {
                    let num = num as usize;
                    if is_rev {
                        Ok((num, is_rev))
                    } else {
                        Ok((num, is_rev))
                    }
                } else {
                    Err(Val::new_err("Can only access a list with an integer."))
                }
            }
            _ => Err(Val::new_err("Can only access a list with an integer.")),
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
        args: &Vec<AstNodePtr>,
    ) -> Val {
        match name {
            "len" => {
                if args.len() != 0 {
                    return Val::new_err("len() must be called with 0 arguments.");
                }
                match obj.get_val() {
                    ValType::List(val) => Val::new_f64(val.len() as f64),
                    ValType::Map(val) => Val::new_f64(val.len() as f64),
                    ValType::Bytes(val) => Val::new_f64(val.len() as f64),
                    ValType::String(val) => Val::new_f64(val.as_bytes().len() as f64),
                    _ => {
                        Val::new_err("len() can only be called on a list or map or bytes or string")
                    }
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
            "unique" => match obj.get_val() {
                ValType::List(val) => {
                    let mut result = Vec::<Val>::new();
                    let mut reached_vals = HashSet::<Val>::new();
                    for elem in val {
                        if reached_vals.insert(elem.clone()) {
                            result.push(elem.clone());
                        }
                    }
                    Val::new_list(result)
                }
                _ => Val::new_err("unique() must be called on a list"),
            },
            "sort" => {
                if args.len() > 1 {
                    return Val::new_err("sort() must be called with zero or one arguments");
                }
                match obj.get_val() {
                    ValType::List(val) => {
                        let mut values = val.clone();
                        let sort_expr = if args.len() == 1 { Some(args[0]) } else { None };
                        match sort_expr {
                            None => {
                                values.sort();
                            }
                            Some(sort_expr) => {
                                values
                                    .sort_by_cached_key(|elem| self.eval(sort_expr, elem, parser));
                            }
                        }

                        Val::new_list(values)
                    }
                    _ => Val::new_err("sort() must be called on a list"),
                }
            }
            "reverse" => match obj.get_val() {
                ValType::List(val) => {
                    let mut val = val.clone();
                    val.reverse();
                    Val::new_list(val)
                }
                _ => Val::new_err("reverse() must be called on a list"),
            },
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
            "sum" => match &obj.get_val() {
                ValType::List(list) => {
                    let mut total: f64 = 0.0;
                    for elem in list {
                        match elem.get_val() {
                            ValType::Float64(val) => {
                                total += val;
                            }
                            _ => {
                                return Val::new_err(
                                    "sum() can only be called on a list of numbers",
                                )
                            }
                        }
                    }
                    Val::new_f64(total)
                }
                _ => Val::new_err("sum() has to be called on a list"),
            },
            "lines" => match obj.get_val() {
                ValType::String(val) => {
                    let mut lines = val
                        .split("\n")
                        .map(|line| Val::new_str(line))
                        .collect::<Vec<_>>();
                    if lines.len() > 0
                        && match lines[lines.len() - 1].get_val() {
                            ValType::String(text) => text == "",
                            _ => panic!(),
                        }
                    {
                        lines.pop();
                    }
                    Val::new_list(lines)
                }
                ValType::Bytes(_) => {
                    let text = self.eval_fcn(parser, obj, "str", args);
                    self.eval_fcn(parser, &text, name, args)
                }
                _ => Val::new_err("lines() must be called on a string or bytes"),
            },
            "joinlines" => match obj.get_val() {
                ValType::List(list) => {
                    let mut result = String::new();
                    for elem in list {
                        match elem.get_val() {
                            ValType::String(elem_text) => {
                                result.push_str(elem_text.as_str());
                                result.push('\n');
                            }
                            _ => {
                                return Val::new_err(
                                    "joinlines() must be called on a list of strings",
                                )
                            }
                        }
                    }
                    Val::new_str(result.as_str())
                }
                _ => Val::new_err("joinlines() must be called on a list"),
            },
            "split" => match obj.get_val() {
                ValType::String(text) => {
                    if args.len() == 0 {
                        Val::new_list(
                            text.split_whitespace()
                                .map(|elem| Val::new_str(elem))
                                .collect::<Vec<_>>(),
                        )
                    } else {
                        match self.eval(args[0], obj, parser).get_val() {
                            ValType::String(split_pattern) => Val::new_list(
                                text.split(split_pattern)
                                    .map(|elem| Val::new_str(elem))
                                    .collect::<Vec<_>>(),
                            ),
                            _ => Val::new_err("split() pattern must be a string"),
                        }
                    }
                }
                ValType::Bytes(_) => {
                    let text = self.eval_fcn(parser, obj, "str", args);
                    self.eval_fcn(parser, &text, name, args)
                }
                _ => Val::new_err("split() must be called on a string"),
            },
            "join" => match &obj.get_val() {
                ValType::List(elems) => {
                    let joiner = self.eval(args[0], obj, parser);
                    let joiner = match joiner.get_val() {
                        ValType::String(joiner) => joiner.as_str(),
                        _ => return Val::new_err("join() pattern must be a string"),
                    };

                    let mut strings_to_join = Vec::<&str>::with_capacity(elems.len());
                    for elem in elems {
                        match elem.get_val() {
                            ValType::String(elem_str) => {
                                strings_to_join.push(elem_str.as_str());
                            }
                            _ => {
                                return Val::new_err(
                                    "All elements passed to join() must be strings",
                                );
                            }
                        }
                    }

                    let result = strings_to_join.join(joiner);
                    Val::new_str(result.as_str())
                }
                _ => Val::new_err("join() must be called on a list"),
            },
            "in" => {
                let mut buffer = Vec::<u8>::new();
                std::io::stdin().read_to_end(&mut buffer).unwrap();
                return Val::new_bytes(buffer);
            }
            "str" => match obj.get_val() {
                ValType::Bytes(bytes) => match std::str::from_utf8(bytes) {
                    Ok(str) => Val::new_str(str),
                    Err(_) => Val::new_err("Unable to decode bytes as utf8 string."),
                },
                _ => Val::new_err("str() must be called on bytes"),
            },
            "bytes" => match obj.get_val() {
                ValType::String(text) => {
                    Val::new_bytes(text.as_bytes().iter().cloned().collect::<Vec<_>>())
                }
                _ => Val::new_err("bytes() must be called on str"),
            },
            "read" => match obj.get_val() {
                ValType::String(file_path) => {
                    let mut buffer = Vec::<u8>::new();
                    match std::fs::File::open(file_path) {
                        Err(_) => Val::new_err("Unable to open file"),
                        Ok(mut fp) => match fp.read_to_end(&mut buffer) {
                            Err(_) => Val::new_err("Unable to read file"),
                            Ok(_) => Val::new_bytes(buffer),
                        },
                    }
                }
                _ => Val::new_err("read() must be called on a string"),
            },
            "env" => {
                let kv_pairs = std::env::vars()
                    .map(|(key, val)| (Val::new_str(key.as_str()), Val::new_str(val.as_str())))
                    .collect::<Vec<_>>();
                Val::new_map(OrderedMap::from_kv_pair_slice(kv_pairs.as_slice()))
            }
            "fromjson" => match obj.get_val() {
                ValType::String(val) => Val::from_json_str(val.as_str()),
                ValType::Bytes(_) => {
                    let text = self.eval_fcn(parser, obj, "str", args);
                    self.eval_fcn(parser, &text, name, args)
                }
                _ => Val::new_err("fromjson() must be called on a string"),
            },
            "keys" => match &obj.get_val() {
                ValType::Map(map) => {
                    let keys = map.keys();
                    Val::new_list(keys)
                }
                _ => Val::new_err("keys() must be called on a map"),
            },
            "values" => match &obj.get_val() {
                ValType::Map(map) => {
                    let values = map.values();
                    Val::new_list(values)
                }
                _ => Val::new_err("values() must be called on a map"),
            },
            "items" => match &obj.get_val() {
                ValType::Map(map) => {
                    let items = map.items();
                    Val::new_list(items)
                }
                _ => Val::new_err("entries() must be called on a map"),
            },
            "fromitems" => match &obj.get_val() {
                ValType::List(val) => {
                    let key_name = Val::new_str("key");
                    let val_name = Val::new_str("val");
                    let mut kv_pairs = Vec::<(Val, Val)>::with_capacity(val.len());
                    for elem in val {
                        let kv_pair = match elem.get_val() {
                            ValType::Map(elem) => {
                                let key = match elem.get(&key_name) {
                                    None => Val::new_null(),
                                    Some(key) => key,
                                };
                                let val = match elem.get(&val_name) {
                                    None => Val::new_null(),
                                    Some(val) => val,
                                };
                                (key, val)
                            }
                            _ => return Val::new_err("entries() must be called on a list of maps"),
                        };
                        kv_pairs.push(kv_pair);
                    }
                    Val::new_map(OrderedMap::from_kv_pair_slice(kv_pairs.as_slice()))
                }
                _ => Val::new_err("entries() must be called on a map"),
            },
            "recursivemap" => {
                if args.len() != 2 {
                    return Val::new_err("recursivemap() must be called with 2 arguments.");
                }

                let sub_node_getter = args[0];
                let mapper = args[1];

                fn helper(
                    this: &mut Evaluator,
                    sub_node_getter: AstNodePtr,
                    mapper: AstNodePtr,
                    parser: &Parser,
                    obj: &Val,
                ) -> Val {
                    let sub_nodes = this.eval(sub_node_getter, obj, parser);
                    match sub_nodes.get_val() {
                        ValType::List(sub_nodes) => {
                            let mut mapped_sub_nodes = Vec::<Val>::with_capacity(sub_nodes.len());
                            for sub_node in sub_nodes {
                                mapped_sub_nodes.push(helper(
                                    this,
                                    sub_node_getter,
                                    mapper,
                                    parser,
                                    sub_node,
                                ));
                            }
                            let new_node = Val::new_map(OrderedMap::from_kv_pair_slice(&[
                                (Val::new_str("node"), obj.clone()),
                                (Val::new_str("vals"), Val::new_list(mapped_sub_nodes)),
                            ]));

                            this.eval(mapper, &new_node, parser)
                        }
                        ValType::Err(_) => sub_nodes,
                        _ => Val::new_err("mapper function in recursivemap() must return a list"),
                    }
                }

                helper(self, sub_node_getter, mapper, parser, obj)
            }
            "recursiveflatten" => {
                if args.len() != 1 {
                    return Val::new_err("recursiveflatten must be called with 1 argument.");
                }

                fn helper(
                    this: &mut Evaluator,
                    results: &mut Vec<Val>,
                    sub_node_getter: AstNodePtr,
                    parser: &Parser,
                    obj: &Val,
                ) -> Result<(), Val> {
                    results.push(obj.clone());
                    let sub_nodes = this.eval(sub_node_getter, obj, parser);
                    match sub_nodes.get_val() {
                        ValType::List(sub_nodes) => {
                            for sub_node in sub_nodes {
                                helper(this, results, sub_node_getter, parser, sub_node)?;
                            }
                            Ok(())
                        }
                        ValType::Err(_) => Err(sub_nodes),
                        _ => Err(Val::new_err(
                            "mapper function in recursiveflatten() must return a list",
                        )),
                    }
                }

                let sub_node_getter = args[0];
                let mut results = Vec::<Val>::new();
                match helper(self, &mut results, sub_node_getter, parser, obj) {
                    Ok(_) => {}
                    Err(err) => {
                        return err;
                    }
                }

                Val::new_list(results)
            }
            "call" => {
                let input = match obj.get_val() {
                    ValType::Bytes(input) => Some(input.as_slice()),
                    ValType::String(str) => Some(str.as_str().as_bytes()),
                    _ => None,
                };
                if args.len() == 0 {
                    return Val::new_err("call() has to be called with at least one argument");
                }

                let mut arg_strs = Vec::<String>::new();
                let mut cwd: Option<String> = None;
                for arg in args {
                    match parser.get_node(*arg) {
                        AstNode::KeywordArgument(keyword, val) => {
                            let keyword = match parser.get_node(*keyword) {
                                AstNode::SubString(keyword) => *keyword,
                                _ => panic!(),
                            };
                            match keyword {
                                "cwd" => {
                                    let cwd_val = self.eval(*val, obj, parser);
                                    let cwd_val = match cwd_val.get_val() {
                                        ValType::String(cwd_val) => cwd_val,
                                        _ => {
                                            return Val::new_err(
                                                "In call(), the :env keyword must be a string",
                                            );
                                        }
                                    };
                                    cwd = Some(cwd_val.clone());
                                }
                                _ => return Val::new_err("Unknown keyword passed to call()"),
                            }
                        }
                        _ => {
                            let arg_val = self.eval(*arg, obj, parser);
                            let arg_str = match arg_val.get_val() {
                                ValType::String(str) => str,
                                _ => {
                                    return Val::new_err("call() arguments must be strings");
                                }
                            };
                            arg_strs.push(arg_str.clone());
                        }
                    }
                }

                let mut cmd = &mut Command::new(arg_strs[0].as_str());

                cmd = cmd.args(&arg_strs[1..]);

                if let Some(cwd) = cwd {
                    cmd = cmd.current_dir(cwd);
                }

                cmd = cmd.stderr(Stdio::inherit());
                cmd = cmd.stdout(Stdio::piped());

                match input {
                    Some(_) => {
                        cmd = cmd.stdin(Stdio::piped());
                    }
                    None => {
                        cmd = cmd.stdin(Stdio::null());
                    }
                };

                let mut proc = match cmd.spawn() {
                    Err(_) => return Val::new_err("Unable to start command"),
                    Ok(proc) => proc,
                };

                if let Some(input) = input {
                    match proc.stdin.take().unwrap().write(input) {
                        Ok(_) => {}
                        Err(_) => {
                            return Val::new_err("Unable to write input to command");
                        }
                    }
                }

                let mut output_buf = Vec::<u8>::new();
                match proc.stdout.take().unwrap().read_to_end(&mut output_buf) {
                    Ok(_) => {}
                    Err(_) => {
                        return Val::new_err("Unable to read output from command");
                    }
                }

                Val::new_bytes(output_buf)
            }
            "range" => match obj.get_val() {
                ValType::Float64(val) => {
                    let val = *val;
                    if val == val.floor() {
                        let val = val as i64;
                        if val > 0 {
                            let val = val as usize;
                            let mut result = Vec::<Val>::with_capacity(val);
                            for idx in 0..val {
                                result.push(Val::new_f64(idx as f64));
                            }
                            Val::new_list(result)
                        } else {
                            Val::new_list(Vec::new())
                        }
                    } else {
                        Val::new_err("range() must be called with an integer")
                    }
                }
                _ => Val::new_err("range() must be called with a number"),
            },
            "zip" => {
                let arg_vals = args
                    .iter()
                    .map(|arg| self.eval(*arg, obj, parser))
                    .collect::<Vec<_>>();
                let mut arg_lists = Vec::<&Vec<Val>>::with_capacity(args.len());
                for arg in &arg_vals {
                    match arg.get_val() {
                        ValType::List(vals) => {
                            arg_lists.push(vals);
                        }
                        _ => return Val::new_err("each argument in zip() must be a list"),
                    }
                }

                let min_len = arg_lists.iter().map(|list| list.len()).min().unwrap_or(0);

                let mut results = Vec::<Val>::with_capacity(min_len);
                for idx in 0..min_len {
                    let temp_list = arg_lists
                        .iter()
                        .map(|list| list[idx].clone())
                        .collect::<Vec<_>>();
                    results.push(Val::new_list(temp_list));
                }

                Val::new_list(results)
            }
            "repeat" => {
                if args.len() != 1 {
                    return Val::new_err("repeat() has to be called with one argument");
                }

                let arg_val = self.eval(args[0], obj, parser);
                match arg_val.get_val() {
                    ValType::Float64(val) => {
                        let val = *val;
                        if val == val.floor() && val >= 0.0 {
                            let val = val as usize;
                            let result = (0..val).map(|_| obj.clone()).collect::<Vec<_>>();
                            Val::new_list(result)
                        } else {
                            Val::new_err("repeat() must be called with a positive integer")
                        }
                    }
                    _ => Val::new_err("repeat() must be called with a number"),
                }
            }
            "iserr" => match obj.get_val() {
                ValType::Err(_) => Val::new_bool(true),
                _ => Val::new_bool(false),
            },
            _ => Val::new_err(format!("Unknown function \"{}\"", name).as_str()),
        }
    }
}
