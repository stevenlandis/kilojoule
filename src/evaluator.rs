use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};

use super::ast_node_pool::{AstNode, AstNodePtr};
use super::parser::Parser;
use super::val::{OrderedMap, Val, ValType};
use std::process::{Command, Stdio};

pub struct EvalCtx {
    variables: HashMap<String, Val>,
    pub val: Val,
}

impl EvalCtx {
    pub fn new() -> Self {
        EvalCtx {
            variables: HashMap::new(),
            val: Val::new_null(),
        }
    }

    pub fn parse_and_eval(&self, text: &str) -> EvalCtx {
        let mut parser = Parser::new(text);
        match parser.external_parse_expr() {
            None => self.with_val(Val::new_null()),
            Some(ast) => match ast {
                Err(err) => self.with_val(Val::new_err(err.to_string().as_str())),
                Ok(ast) => {
                    // for (idx, val) in parser.pool.vals.iter().enumerate() {
                    //     println!("{}: {:?}", idx, val);
                    // }
                    self.eval(ast, &parser)
                }
            },
        }
    }

    fn eval_bool(&self, node: AstNodePtr, parser: &Parser) -> Option<bool> {
        let val = self.eval(node, parser).val;
        match val.get_val() {
            ValType::Bool(val) => Some(*val),
            _ => None,
        }
    }

    fn eval_bool_expr(
        &self,
        parser: &Parser,
        left: AstNodePtr,
        right: AstNodePtr,
        left_err: &str,
        right_err: &str,
        callback: impl Fn(bool, bool) -> bool,
    ) -> Val {
        let left_val = match self.eval_bool(left, parser) {
            None => {
                return Val::new_err(left_err);
            }
            Some(val) => val,
        };
        let right_val = match self.eval_bool(right, parser) {
            None => {
                return Val::new_err(right_err);
            }
            Some(val) => val,
        };

        Val::new_bool(callback(left_val, right_val))
    }

    fn eval_f64_expr(
        &self,
        parser: &Parser,
        left: AstNodePtr,
        right: AstNodePtr,
        left_err: &str,
        right_err: &str,
        callback: impl Fn(f64, f64) -> f64,
    ) -> Val {
        let left_val = self.eval(left, parser).val;
        let left_val = match left_val.get_val() {
            ValType::Float64(val) => *val,
            _ => {
                return Val::new_err(left_err);
            }
        };
        let right_val = self.eval(right, parser).val;
        let right_val = match right_val.get_val() {
            ValType::Float64(val) => *val,
            _ => {
                return Val::new_err(right_err);
            }
        };
        Val::new_f64(callback(left_val, right_val))
    }

    fn with_val(&self, val: Val) -> EvalCtx {
        EvalCtx {
            variables: self.variables.clone(),
            val,
        }
    }

    fn eval(&self, node: AstNodePtr, parser: &Parser) -> EvalCtx {
        match parser.get_node(node) {
            AstNode::Null => self.with_val(Val::new_null()),
            AstNode::Pipe(left, right) => {
                let left_val = self.eval(*left, parser);
                left_val.eval(*right, parser)
            }
            AstNode::Coalesce(left, right) => {
                let left = self.eval(*left, parser);
                match &left.val.get_val() {
                    ValType::Null => self.eval(*right, parser),
                    _ => left,
                }
            }
            AstNode::Dot => self.with_val(self.val.clone()),
            AstNode::Or(left, right) => self.with_val(self.eval_bool_expr(
                parser,
                *left,
                *right,
                "Left side of OR has to be a boolean",
                "Right side of OR has to be a boolean",
                |left, right| left || right,
            )),
            AstNode::And(left, right) => self.with_val(self.eval_bool_expr(
                parser,
                *left,
                *right,
                "Left side of OR has to be a boolean",
                "Right side of OR has to be a boolean",
                |left, right| left && right,
            )),
            AstNode::Not(expr) => {
                let left_val = match self.eval_bool(*expr, parser) {
                    None => {
                        return self.with_val(Val::new_err(
                            "\"not\" operator has to be called on a boolean",
                        ));
                    }
                    Some(val) => val,
                };

                self.with_val(Val::new_bool(!left_val))
            }
            AstNode::Equals(left, right) => {
                let left_val = self.eval(*left, parser);
                let right_val = self.eval(*right, parser);
                self.with_val(Val::new_bool(left_val.val == right_val.val))
            }
            AstNode::NotEquals(left, right) => {
                let left_val = self.eval(*left, parser);
                let right_val = self.eval(*right, parser);
                self.with_val(Val::new_bool(left_val.val != right_val.val))
            }
            AstNode::LessThan(left, right) => {
                let left_val = self.eval(*left, parser);
                let right_val = self.eval(*right, parser);
                self.with_val(Val::new_bool(left_val.val < right_val.val))
            }
            AstNode::LessThanOrEqual(left, right) => {
                let left_val = self.eval(*left, parser);
                let right_val = self.eval(*right, parser);
                self.with_val(Val::new_bool(left_val.val <= right_val.val))
            }
            AstNode::GreaterThan(left, right) => {
                let left_val = self.eval(*left, parser);
                let right_val = self.eval(*right, parser);
                self.with_val(Val::new_bool(left_val.val > right_val.val))
            }
            AstNode::GreaterThanOrEqual(left, right) => {
                let left_val = self.eval(*left, parser);
                let right_val = self.eval(*right, parser);
                self.with_val(Val::new_bool(left_val.val >= right_val.val))
            }
            AstNode::Add(left, right) => self.with_val(self.eval_f64_expr(
                parser,
                *left,
                *right,
                "Left side of addition has to be a float",
                "Right side of addition has to be a float",
                |left, right| left + right,
            )),
            AstNode::Subtract(left, right) => self.with_val(self.eval_f64_expr(
                parser,
                *left,
                *right,
                "Left side of subtraction has to be a float",
                "Right side of subtraction has to be a float",
                |left, right| left - right,
            )),
            AstNode::Multiply(left, right) => self.with_val(self.eval_f64_expr(
                parser,
                *left,
                *right,
                "Left side of multiplication has to be a float",
                "Right side of multiplication has to be a float",
                |left, right| left * right,
            )),
            AstNode::Divide(left, right) => {
                let left_val = self.eval(*left, parser).val;
                let left_val = match left_val.get_val() {
                    ValType::Float64(val) => *val,
                    _ => {
                        return self
                            .with_val(Val::new_err("Left side of division has to be a float"));
                    }
                };
                let right_val = self.eval(*right, parser).val;
                let right_val = match right_val.get_val() {
                    ValType::Float64(val) => *val,
                    _ => {
                        return self
                            .with_val(Val::new_err("Right side of division has to be a float"));
                    }
                };
                if right_val == 0.0 {
                    self.with_val(Val::new_err("divide by zero"))
                } else {
                    self.with_val(Val::new_f64(left_val / right_val))
                }
            }
            AstNode::Negative(expr) => {
                let val = self.eval(*expr, parser).val;
                let val = match val.get_val() {
                    ValType::Float64(val) => *val,
                    _ => {
                        return self.with_val(Val::new_err("Negative expression must be a number"));
                    }
                };
                self.with_val(Val::new_f64(-val))
            }
            AstNode::Integer(val) => self.with_val(Val::new_f64(*val as f64)),
            AstNode::MapLiteral(contents) => {
                let mut map = OrderedMap::new();
                fn helper(
                    this: &EvalCtx,
                    parser: &Parser,
                    map: &mut OrderedMap,
                    node: AstNodePtr,
                ) -> Result<(), Val> {
                    match parser.get_node(node) {
                        AstNode::ListNode(left, right) => {
                            helper(this, parser, map, *left)?;
                            helper(this, parser, map, *right)?;
                            Ok(())
                        }
                        AstNode::MapKeyValPair { key, val } => {
                            let key_obj = match parser.get_node(*key) {
                                AstNode::Identifier(key_name) => Val::new_str(key_name),
                                _ => this.eval(*key, parser).val,
                            };
                            let val_obj = this.eval(*val, parser).val;
                            map.insert(&key_obj, &val_obj);
                            Ok(())
                        }
                        AstNode::Spread(spread) => {
                            let spread_val = this.eval(*spread, parser).val;
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
                        match helper(self, parser, &mut map, *contents) {
                            Ok(_) => {}
                            Err(err) => return self.with_val(err),
                        };
                    }
                };

                self.with_val(Val::new_map(map))
            }
            AstNode::ListLiteral(contents) => {
                let mut list = Vec::<Val>::new();

                fn helper(
                    this: &EvalCtx,
                    parser: &Parser,
                    list: &mut Vec<Val>,
                    node: AstNodePtr,
                ) -> Result<(), Val> {
                    match parser.get_node(node) {
                        AstNode::ListNode(left, right) => {
                            helper(this, parser, list, *left)?;
                            helper(this, parser, list, *right)?;
                            Ok(())
                        }
                        AstNode::Spread(spread) => {
                            let spread_list = this.eval(*spread, parser).val;
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
                            let elem_val = this.eval(node, parser).val;
                            list.push(elem_val);
                            Ok(())
                        }
                    }
                }

                match contents {
                    None => {}
                    Some(contents) => {
                        match helper(self, parser, &mut list, *contents) {
                            Ok(_) => {}
                            Err(err) => {
                                return self.with_val(err);
                            }
                        };
                    }
                };

                self.with_val(Val::new_list(list))
            }
            AstNode::FormatString(contents) => {
                let mut buffer = Vec::<u8>::new();
                fn helper(this: &EvalCtx, parser: &Parser, buffer: &mut Vec<u8>, node: AstNodePtr) {
                    match parser.get_node(node) {
                        AstNode::ListNode(left, right) => {
                            helper(this, parser, buffer, *left);
                            helper(this, parser, buffer, *right);
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
                            let elem_val = this.eval(node, parser).val;
                            match elem_val.get_val() {
                                ValType::String(sub_text) => {
                                    buffer.extend(sub_text.as_bytes());
                                }
                                _ => {
                                    EvalCtx::write_val(&elem_val, buffer, false).unwrap();
                                }
                            }
                        }
                    }
                }

                match contents {
                    None => {}
                    Some(contents) => {
                        helper(self, parser, &mut buffer, *contents);
                    }
                };

                self.with_val(Val::new_str(
                    std::str::from_utf8(buffer.as_slice()).unwrap(),
                ))
            }
            AstNode::AccessChain(expr, accessor) => {
                let val_to_access = self.eval(*expr, parser).val;
                if let ValType::Err(_) = val_to_access.get_val() {
                    return self.with_val(val_to_access);
                }

                match val_to_access.get_val() {
                    ValType::Map(map) => {
                        if let AstNode::ReverseIdx(_) = parser.get_node(*accessor) {
                            return self.with_val(Val::new_err(
                                "Maps cannot be accessed with a reverse index",
                            ));
                        }

                        let key_val = match parser.get_node(*accessor) {
                            AstNode::Identifier(key) => Val::new_str(key),
                            _ => self.eval(*accessor, parser).val,
                        };

                        match map.get(&key_val) {
                            None => self.with_val(Val::new_null()),
                            Some(val) => self.with_val(val),
                        }
                    }
                    ValType::List(list) => match parser.get_node(*accessor) {
                        AstNode::SliceAccess(start, end) => {
                            let start_idx = match start {
                                None => 0,
                                Some(start_expr) => {
                                    match self.eval_list_access(*start_expr, parser) {
                                        Err(err) => {
                                            return self.with_val(err);
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
                                Some(end_expr) => match self.eval_list_access(*end_expr, parser) {
                                    Err(err) => {
                                        return self.with_val(err);
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
                            self.with_val(Val::new_list(list[start_idx..end_idx].to_vec()))
                        }
                        _ => match self.eval_list_access(*accessor, parser) {
                            Err(err) => self.with_val(err),
                            Ok((idx, is_rev)) => {
                                if idx < list.len() {
                                    let idx = if is_rev { list.len() - idx - 1 } else { idx };
                                    self.with_val(list[idx].clone())
                                } else {
                                    self.with_val(Val::new_err("List access out of bounds"))
                                }
                            }
                        },
                    },
                    ValType::Null => self.with_val(self.val.clone()),
                    _ => self.with_val(Val::new_err("Invalid access")),
                }
            }
            AstNode::Bool(val) => self.with_val(Val::new_bool(*val)),
            AstNode::FcnCall { name, args } => {
                let name = match parser.get_node(*name) {
                    AstNode::Identifier(name) => *name,
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
                        helper(&self.val, parser, &mut args_vec, *args);
                    }
                };

                self.with_val(self.eval_fcn(parser, name, &args_vec))
            }
            AstNode::LetStmt { identifier, expr } => {
                let identifier = match parser.get_node(*identifier) {
                    AstNode::Identifier(identifier) => *identifier,
                    _ => panic!(),
                };

                let val = self.eval(*expr, parser).val;
                let mut variables = self.variables.clone();

                variables.insert(identifier.to_string(), val);

                EvalCtx {
                    variables,
                    val: self.val.clone(),
                }
            }
            AstNode::Identifier(identifier) => match self.variables.get(*identifier) {
                None => self.with_val(Val::new_err("undefined variable access")),
                Some(val) => self.with_val(val.clone()),
            },
            _ => panic!("Unimplemented {:?}", parser.get_node(node)),
        }
    }

    fn eval_list_access(&self, expr: AstNodePtr, parser: &Parser) -> Result<(usize, bool), Val> {
        let (is_rev, idx_expr) = match parser.get_node(expr) {
            AstNode::ReverseIdx(rev_expr) => (true, *rev_expr),
            _ => (false, expr),
        };
        let idx = self.eval(idx_expr, parser).val;
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
        val: &Val,
        writer: &mut impl std::io::Write,
        use_indent: bool,
    ) -> std::io::Result<()> {
        match val.write_to_str(writer, 0, use_indent) {
            Err(err) => Err(err),
            Ok(_) => Ok(()),
        }
    }

    pub fn eval_fcn(&self, parser: &Parser, name: &str, args: &Vec<AstNodePtr>) -> Val {
        match name {
            "len" => {
                if args.len() != 0 {
                    return Val::new_err("len() must be called with 0 arguments.");
                }
                match self.val.get_val() {
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
                match self.val.get_val() {
                    ValType::List(val) => {
                        let mut result = Vec::<Val>::with_capacity(val.len());
                        for elem in val {
                            result.push(self.with_val(elem.clone()).eval(args[0], parser).val);
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

                match self.val.get_val() {
                    ValType::List(val) => {
                        let mut groups = Vec::<(Val, Vec<Val>)>::new();
                        let mut key_to_group_idx = HashMap::<Val, usize>::new();

                        for elem in val {
                            let ctx = EvalCtx {
                                variables: self.variables.clone(),
                                val: elem.clone(),
                            };
                            let key_val = ctx.eval(args[0], parser).val;
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
            "unique" => match self.val.get_val() {
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
                match self.val.get_val() {
                    ValType::List(val) => {
                        let mut values = val.clone();
                        let sort_expr = if args.len() == 1 { Some(args[0]) } else { None };
                        match sort_expr {
                            None => {
                                values.sort();
                            }
                            Some(sort_expr) => {
                                values.sort_by_cached_key(|elem| {
                                    let ctx = EvalCtx {
                                        variables: self.variables.clone(),
                                        val: elem.clone(),
                                    };
                                    ctx.eval(sort_expr, parser).val
                                });
                            }
                        }

                        Val::new_list(values)
                    }
                    _ => Val::new_err("sort() must be called on a list"),
                }
            }
            "reverse" => match self.val.get_val() {
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
                match self.val.get_val() {
                    ValType::List(val) => {
                        let mut result = Vec::<Val>::new();
                        for elem in val {
                            let ctx = self.with_val(elem.clone());
                            let filter_val = ctx.eval(args[0], parser).val;
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
            "sum" => match self.val.get_val() {
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
            "lines" => match self.val.get_val() {
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
                    let text = self.eval_fcn(parser, "str", args);
                    self.with_val(text).eval_fcn(parser, name, args)
                }
                _ => Val::new_err("lines() must be called on a string or bytes"),
            },
            "joinlines" => match self.val.get_val() {
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
            "split" => match self.val.get_val() {
                ValType::String(text) => {
                    if args.len() == 0 {
                        Val::new_list(
                            text.split_whitespace()
                                .map(|elem| Val::new_str(elem))
                                .collect::<Vec<_>>(),
                        )
                    } else {
                        match self.eval(args[0], parser).val.get_val() {
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
                    let text = self.eval_fcn(parser, "str", args);
                    self.with_val(text).eval_fcn(parser, name, args)
                }
                _ => Val::new_err("split() must be called on a string"),
            },
            "join" => match self.val.get_val() {
                ValType::List(elems) => {
                    let joiner = self.eval(args[0], parser).val;
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
            "str" => match self.val.get_val() {
                ValType::Bytes(bytes) => match std::str::from_utf8(bytes) {
                    Ok(str) => Val::new_str(str),
                    Err(_) => Val::new_err("Unable to decode bytes as utf8 string."),
                },
                _ => Val::new_err("str() must be called on bytes"),
            },
            "bytes" => match self.val.get_val() {
                ValType::String(text) => {
                    Val::new_bytes(text.as_bytes().iter().cloned().collect::<Vec<_>>())
                }
                _ => Val::new_err("bytes() must be called on str"),
            },
            "read" => match self.val.get_val() {
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
            "fromjson" => match self.val.get_val() {
                ValType::String(val) => Val::from_json_str(val.as_str()),
                ValType::Bytes(_) => {
                    let text = self.eval_fcn(parser, "str", args);
                    self.with_val(text).eval_fcn(parser, name, args)
                }
                _ => Val::new_err("fromjson() must be called on a string"),
            },
            "keys" => match self.val.get_val() {
                ValType::Map(map) => {
                    let keys = map.keys();
                    Val::new_list(keys)
                }
                _ => Val::new_err("keys() must be called on a map"),
            },
            "values" => match self.val.get_val() {
                ValType::Map(map) => {
                    let values = map.values();
                    Val::new_list(values)
                }
                _ => Val::new_err("values() must be called on a map"),
            },
            "items" => match self.val.get_val() {
                ValType::Map(map) => {
                    let items = map.items();
                    Val::new_list(items)
                }
                _ => Val::new_err("entries() must be called on a map"),
            },
            "fromitems" => match self.val.get_val() {
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
                            ValType::List(elems) => {
                                if elems.len() != 2 {
                                    return Val::new_err("fromitems() lists must have 2 elements");
                                }
                                (elems[0].clone(), elems[1].clone())
                            }
                            _ => {
                                return Val::new_err("fromitems() must be called on a list of maps")
                            }
                        };
                        kv_pairs.push(kv_pair);
                    }
                    Val::new_map(OrderedMap::from_kv_pair_slice(kv_pairs.as_slice()))
                }
                _ => Val::new_err("fromitems() must be called on a map"),
            },
            "recursivemap" => {
                if args.len() != 2 {
                    return Val::new_err("recursivemap() must be called with 2 arguments.");
                }

                let sub_node_getter = args[0];
                let mapper = args[1];

                fn helper(
                    this: &EvalCtx,
                    sub_node_getter: AstNodePtr,
                    mapper: AstNodePtr,
                    parser: &Parser,
                ) -> Val {
                    let sub_nodes = this.eval(sub_node_getter, parser).val;
                    match sub_nodes.get_val() {
                        ValType::List(sub_nodes) => {
                            let mut mapped_sub_nodes = Vec::<Val>::with_capacity(sub_nodes.len());
                            for sub_node in sub_nodes {
                                mapped_sub_nodes.push(helper(
                                    &this.with_val(sub_node.clone()),
                                    sub_node_getter,
                                    mapper,
                                    parser,
                                ));
                            }
                            let new_node = Val::new_map(OrderedMap::from_kv_pair_slice(&[
                                (Val::new_str("node"), this.val.clone()),
                                (Val::new_str("vals"), Val::new_list(mapped_sub_nodes)),
                            ]));

                            this.with_val(new_node).eval(mapper, parser).val
                        }
                        ValType::Err(_) => sub_nodes,
                        _ => Val::new_err("mapper function in recursivemap() must return a list"),
                    }
                }

                helper(self, sub_node_getter, mapper, parser)
            }
            "recursiveflatten" => {
                if args.len() != 1 {
                    return Val::new_err("recursiveflatten must be called with 1 argument.");
                }

                fn helper(
                    this: &EvalCtx,
                    results: &mut Vec<Val>,
                    sub_node_getter: AstNodePtr,
                    parser: &Parser,
                ) -> Result<(), Val> {
                    results.push(this.val.clone());
                    let sub_nodes = this.eval(sub_node_getter, parser).val;
                    match sub_nodes.get_val() {
                        ValType::List(sub_nodes) => {
                            for sub_node in sub_nodes {
                                helper(
                                    &this.with_val(sub_node.clone()),
                                    results,
                                    sub_node_getter,
                                    parser,
                                )?;
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
                match helper(self, &mut results, sub_node_getter, parser) {
                    Ok(_) => {}
                    Err(err) => {
                        return err;
                    }
                }

                Val::new_list(results)
            }
            "call" => {
                let input = match self.val.get_val() {
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
                                AstNode::Identifier(keyword) => *keyword,
                                _ => panic!(),
                            };
                            match keyword {
                                "cwd" => {
                                    let cwd_val = self.eval(*val, parser).val;
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
                            let arg_val = self.eval(*arg, parser).val;
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
            "range" => match self.val.get_val() {
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
                if args.len() != 0 {
                    return Val::new_err("zip() must be called with zero arguments");
                }
                let arg_vals = match self.val.get_val() {
                    ValType::List(vals) => vals,
                    _ => return Val::new_err("zip() must be called on a list"),
                };
                let mut arg_lists = Vec::<&Vec<Val>>::with_capacity(args.len());
                for arg in arg_vals {
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

                let arg_val = self.eval(args[0], parser).val;
                match arg_val.get_val() {
                    ValType::Float64(val) => {
                        let val = *val;
                        if val == val.floor() && val >= 0.0 {
                            let val = val as usize;
                            let result = (0..val).map(|_| self.val.clone()).collect::<Vec<_>>();
                            Val::new_list(result)
                        } else {
                            Val::new_err("repeat() must be called with a positive integer")
                        }
                    }
                    _ => Val::new_err("repeat() must be called with a number"),
                }
            }
            "iserr" => match self.val.get_val() {
                ValType::Err(_) => Val::new_bool(true),
                _ => Val::new_bool(false),
            },
            "isnumber" => match self.val.get_val() {
                ValType::Float64(_) => Val::new_bool(true),
                _ => Val::new_bool(false),
            },
            "isbool" => match self.val.get_val() {
                ValType::Bool(_) => Val::new_bool(true),
                _ => Val::new_bool(false),
            },
            "isstring" => match self.val.get_val() {
                ValType::String(_) => Val::new_bool(true),
                _ => Val::new_bool(false),
            },
            "islist" => match self.val.get_val() {
                ValType::List(_) => Val::new_bool(true),
                _ => Val::new_bool(false),
            },
            "ismap" => match self.val.get_val() {
                ValType::Map(_) => Val::new_bool(true),
                _ => Val::new_bool(false),
            },
            "isbytes" => match self.val.get_val() {
                ValType::Bytes(_) => Val::new_bool(true),
                _ => Val::new_bool(false),
            },
            "texttable" => match self.val.get_val() {
                ValType::String(_) => {
                    let lines = self.eval_fcn(parser, "lines", &Vec::new());

                    let split_lines = match lines.get_val() {
                        ValType::Err(_) => {
                            return lines;
                        }
                        ValType::List(val) => {
                            if val.len() == 0 {
                                return Val::new_err("texttable() has to be called on a string with at least one line.");
                            }
                            val.iter()
                                .map(|elem| {
                                    self.with_val(elem.clone()).eval_fcn(
                                        parser,
                                        "split",
                                        &Vec::new(),
                                    )
                                })
                                .collect::<Vec<_>>()
                        }
                        _ => panic!(),
                    };
                    let split_lines = split_lines
                        .iter()
                        .map(|elem| match elem.get_val() {
                            ValType::List(sub_list) => sub_list,
                            _ => panic!(),
                        })
                        .collect::<Vec<_>>();

                    let col_names = &split_lines[0];
                    let n_keys = col_names.len();
                    let null = Val::new_null();
                    let mut rows = Vec::<Val>::new();
                    for line in &split_lines[1..] {
                        let mut map = OrderedMap::new();
                        for col_idx in 0..n_keys - 1 {
                            map.insert(
                                &col_names[col_idx],
                                if col_idx < line.len() {
                                    &line[col_idx]
                                } else {
                                    &null
                                },
                            )
                        }
                        if n_keys - 1 < line.len() {
                            let list_to_join = line[n_keys - 1..]
                                .iter()
                                .map(|elem| match elem.get_val() {
                                    ValType::String(val) => val.as_str(),
                                    _ => panic!(),
                                })
                                .collect::<Vec<_>>();
                            let joined_line = list_to_join.join(" ");
                            map.insert(&col_names[n_keys - 1], &Val::new_str(joined_line.as_str()));
                        } else {
                            map.insert(&col_names[n_keys - 1], &null);
                        }
                        rows.push(Val::new_map(map));
                    }

                    Val::new_list(rows)
                }
                ValType::Bytes(_) => {
                    let text = self.eval_fcn(parser, "str", args);
                    self.with_val(text).eval_fcn(parser, name, args)
                }
                _ => Val::new_err("texttable() must be called on a string"),
            },
            "flatten" => match self.val.get_val() {
                ValType::List(lists) => {
                    let mut result = Vec::<Val>::new();
                    for list in lists {
                        match list.get_val() {
                            ValType::List(elems) => {
                                for elem in elems {
                                    result.push(elem.clone());
                                }
                            }
                            _ => {
                                return Val::new_err("flatten() must be called on a list of lists")
                            }
                        }
                    }

                    Val::new_list(result)
                }
                _ => Val::new_err("flatten() must be called on a list"),
            },
            "fromcsv" => {
                let mut reader = match self.val.get_val() {
                    ValType::Bytes(bytes) => csv::Reader::from_reader(bytes.as_slice()),
                    ValType::String(val) => csv::Reader::from_reader(val.as_bytes()),
                    _ => return Val::new_err("fromcsv() must be called on string or bytes"),
                };

                let mut lines = Vec::<Val>::new();

                if reader.has_headers() {
                    match reader.headers() {
                        Err(_) => return Val::new_err("fromcsv() is unable to read csv"),
                        Ok(record) => {
                            let mut line = Vec::<Val>::new();
                            for elem in record.iter() {
                                line.push(Val::new_str(elem));
                            }
                            lines.push(Val::new_list(line));
                        }
                    }
                }

                for record in reader.records() {
                    let mut line = Vec::<Val>::new();
                    match record {
                        Err(_) => return Val::new_err("fromcsv() is unable to read csv"),
                        Ok(record) => {
                            for elem in record.iter() {
                                line.push(Val::new_str(elem));
                            }
                        }
                    }
                    lines.push(Val::new_list(line))
                }

                Val::new_list(lines)
            }
            "tocsv" => {
                let mut buffer = Vec::<u8>::new();
                let mut writer = csv::Writer::from_writer(&mut buffer);
                match self.val.get_val() {
                    ValType::List(rows) => {
                        for row in rows {
                            let mut record = csv::StringRecord::new();
                            match row.get_val() {
                                ValType::List(row) => {
                                    for elem in row {
                                        match elem.get_val() {
                                            ValType::String(val) => record.push_field(val),
                                            _ => {
                                                let mut buf = Vec::<u8>::new();
                                                elem.write_to_str(&mut buf, 0, false).unwrap();
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    return Val::new_err(
                                        "tocsv() must be called on a list of lists",
                                    )
                                }
                            }

                            writer.write_record(record.iter()).unwrap();
                        }
                    }
                    _ => return Val::new_err("tocsv() must be called on a list"),
                }

                drop(writer);

                Val::new_bytes(buffer)
            }
            _ => Val::new_err(format!("Unknown function \"{}\"", name).as_str()),
        }
    }
}
