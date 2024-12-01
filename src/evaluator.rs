use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};

use super::ast_node::{AstNode, AstNodeType};
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
                    self.eval(&ast)
                }
            },
        }
    }

    fn eval_bool(&self, node: &AstNode) -> Option<bool> {
        let val = self.eval(node).val;
        match val.get_val() {
            ValType::Bool(val) => Some(*val),
            _ => None,
        }
    }

    fn eval_i64(&self, node: &AstNode) -> Option<i64> {
        let val = self.eval(node).val;
        match val.get_val() {
            ValType::Float64(val) => {
                if *val == val.trunc() {
                    Some(*val as i64)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn eval_bool_expr(
        &self,
        left: &AstNode,
        right: &AstNode,
        left_err: &str,
        right_err: &str,
        callback: impl Fn(bool, bool) -> bool,
    ) -> Val {
        let left_val = match self.eval_bool(left) {
            None => {
                return Val::new_err(left_err);
            }
            Some(val) => val,
        };
        let right_val = match self.eval_bool(right) {
            None => {
                return Val::new_err(right_err);
            }
            Some(val) => val,
        };

        Val::new_bool(callback(left_val, right_val))
    }

    fn eval_f64_expr(
        &self,
        left: &AstNode,
        right: &AstNode,
        left_err: &str,
        right_err: &str,
        callback: impl Fn(f64, f64) -> f64,
    ) -> Val {
        let left_val = self.eval(left).val;
        let left_val = match left_val.get_val() {
            ValType::Float64(val) => *val,
            _ => {
                return Val::new_err(left_err);
            }
        };
        let right_val = self.eval(right).val;
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

    fn eval(&self, node: &AstNode) -> EvalCtx {
        match node.get_type() {
            AstNodeType::Null => self.with_val(Val::new_null()),
            AstNodeType::Pipe(left, right) => {
                let left_val = self.eval(left);
                left_val.eval(right)
            }
            AstNodeType::Coalesce(left, right) => {
                let left = self.eval(left);
                match left.val.get_val() {
                    ValType::Null => self.eval(right),
                    _ => left,
                }
            }
            AstNodeType::Dot => self.with_val(self.val.clone()),
            AstNodeType::Or(left, right) => self.with_val(self.eval_bool_expr(
                left,
                right,
                "Left side of OR has to be a boolean",
                "Right side of OR has to be a boolean",
                |left, right| left || right,
            )),
            AstNodeType::And(left, right) => self.with_val(self.eval_bool_expr(
                left,
                right,
                "Left side of OR has to be a boolean",
                "Right side of OR has to be a boolean",
                |left, right| left && right,
            )),
            AstNodeType::Not(expr) => {
                let left_val = match self.eval_bool(expr) {
                    None => {
                        return self.with_val(Val::new_err(
                            "\"not\" operator has to be called on a boolean",
                        ));
                    }
                    Some(val) => val,
                };

                self.with_val(Val::new_bool(!left_val))
            }
            AstNodeType::Equals(left, right) => {
                let left_val = self.eval(left);
                let right_val = self.eval(right);
                self.with_val(Val::new_bool(left_val.val == right_val.val))
            }
            AstNodeType::NotEquals(left, right) => {
                let left_val = self.eval(left);
                let right_val = self.eval(right);
                self.with_val(Val::new_bool(left_val.val != right_val.val))
            }
            AstNodeType::LessThan(left, right) => {
                let left_val = self.eval(left);
                let right_val = self.eval(right);
                self.with_val(Val::new_bool(left_val.val < right_val.val))
            }
            AstNodeType::LessThanOrEqual(left, right) => {
                let left_val = self.eval(left);
                let right_val = self.eval(right);
                self.with_val(Val::new_bool(left_val.val <= right_val.val))
            }
            AstNodeType::GreaterThan(left, right) => {
                let left_val = self.eval(left);
                let right_val = self.eval(right);
                self.with_val(Val::new_bool(left_val.val > right_val.val))
            }
            AstNodeType::GreaterThanOrEqual(left, right) => {
                let left_val = self.eval(left);
                let right_val = self.eval(right);
                self.with_val(Val::new_bool(left_val.val >= right_val.val))
            }
            AstNodeType::Add(left, right) => self.with_val(self.eval_f64_expr(
                left,
                right,
                "Left side of addition has to be a float",
                "Right side of addition has to be a float",
                |left, right| left + right,
            )),
            AstNodeType::Subtract(left, right) => self.with_val(self.eval_f64_expr(
                left,
                right,
                "Left side of subtraction has to be a float",
                "Right side of subtraction has to be a float",
                |left, right| left - right,
            )),
            AstNodeType::Multiply(left, right) => self.with_val(self.eval_f64_expr(
                left,
                right,
                "Left side of multiplication has to be a float",
                "Right side of multiplication has to be a float",
                |left, right| left * right,
            )),
            AstNodeType::Divide(left, right) => {
                let left_val = self.eval(left).val;
                let left_val = match left_val.get_val() {
                    ValType::Float64(val) => *val,
                    _ => {
                        return self
                            .with_val(Val::new_err("Left side of division has to be a float"));
                    }
                };
                let right_val = self.eval(right).val;
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
            AstNodeType::Negative(expr) => {
                let val = self.eval(expr).val;
                let val = match val.get_val() {
                    ValType::Float64(val) => *val,
                    _ => {
                        return self.with_val(Val::new_err("Negative expression must be a number"));
                    }
                };
                self.with_val(Val::new_f64(-val))
            }
            AstNodeType::Integer(val) => self.with_val(Val::new_f64(*val as f64)),
            AstNodeType::Float64(val) => self.with_val(Val::new_f64(*val)),
            AstNodeType::MapLiteral(contents) => {
                let mut map = OrderedMap::new();
                fn helper(this: &EvalCtx, map: &mut OrderedMap, node: &AstNode) -> Result<(), Val> {
                    match node.get_type() {
                        AstNodeType::ListNode(left, right) => {
                            helper(this, map, left)?;
                            helper(this, map, right)?;
                            Ok(())
                        }
                        AstNodeType::MapKeyValPair { key, val } => {
                            let key_obj = match key.get_type() {
                                AstNodeType::Identifier(key_name) => {
                                    Val::new_str(key_name.as_str())
                                }
                                _ => this.eval(key).val,
                            };
                            let val_obj = this.eval(val).val;
                            map.insert(&key_obj, &val_obj);
                            Ok(())
                        }
                        AstNodeType::Spread(spread) => {
                            let spread_val = this.eval(spread).val;
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
                        AstNodeType::MapDelete(delete) => {
                            let del_key = this.eval(delete).val;
                            match del_key.get_val() {
                                ValType::Err(_) => return Err(del_key),
                                _ => {
                                    map.delete(&del_key);
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
                        match helper(self, &mut map, contents) {
                            Ok(_) => {}
                            Err(err) => return self.with_val(err),
                        };
                    }
                };

                self.with_val(Val::new_map(map))
            }
            AstNodeType::ListLiteral(contents) => {
                let mut list = Vec::<Val>::new();

                fn helper(this: &EvalCtx, list: &mut Vec<Val>, node: &AstNode) -> Result<(), Val> {
                    match node.get_type() {
                        AstNodeType::ListNode(left, right) => {
                            helper(this, list, left)?;
                            helper(this, list, right)?;
                            Ok(())
                        }
                        AstNodeType::Spread(spread) => {
                            let spread_list = this.eval(spread).val;
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
                            let elem_val = this.eval(node).val;
                            list.push(elem_val);
                            Ok(())
                        }
                    }
                }

                match contents {
                    None => {}
                    Some(contents) => {
                        match helper(self, &mut list, contents) {
                            Ok(_) => {}
                            Err(err) => {
                                return self.with_val(err);
                            }
                        };
                    }
                };

                self.with_val(Val::new_list(list))
            }
            AstNodeType::FormatString(contents) => {
                let mut buffer = Vec::<u8>::new();
                fn helper(this: &EvalCtx, buffer: &mut Vec<u8>, node: &AstNode) {
                    match node.get_type() {
                        AstNodeType::ListNode(left, right) => {
                            helper(this, buffer, left);
                            helper(this, buffer, right);
                        }
                        AstNodeType::SubString(text) => {
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
                            let elem_val = this.eval(node).val;
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
                        helper(self, &mut buffer, contents);
                    }
                };

                self.with_val(Val::new_str(
                    std::str::from_utf8(buffer.as_slice()).unwrap(),
                ))
            }
            AstNodeType::AccessChain(expr, accessor) => {
                let val_to_access = self.eval(expr).val;
                if let ValType::Err(_) = val_to_access.get_val() {
                    return self.with_val(val_to_access);
                }

                match val_to_access.get_val() {
                    ValType::Map(map) => {
                        if let AstNodeType::ReverseIdx(_) = accessor.get_type() {
                            return self.with_val(Val::new_err(
                                "Maps cannot be accessed with a reverse index",
                            ));
                        }

                        let key_val = match accessor.get_type() {
                            AstNodeType::Identifier(key) => Val::new_str(key.as_str()),
                            _ => self.eval(accessor).val,
                        };

                        match map.get(&key_val) {
                            None => self.with_val(Val::new_null()),
                            Some(val) => self.with_val(val),
                        }
                    }
                    ValType::List(list) => match accessor.get_type() {
                        AstNodeType::SliceAccess(start, end) => {
                            let start_idx = match start {
                                None => 0,
                                Some(start_expr) => match self.eval_list_access(start_expr) {
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
                                },
                            };
                            let end_idx = match end {
                                None => list.len(),
                                Some(end_expr) => match self.eval_list_access(end_expr) {
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
                        _ => match self.eval_list_access(accessor) {
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
            AstNodeType::Bool(val) => self.with_val(Val::new_bool(*val)),
            AstNodeType::FcnCall { name, args } => {
                let name = match name.get_type() {
                    AstNodeType::Identifier(name) => name,
                    _ => panic!(),
                };

                let mut args_vec = Vec::<AstNode>::new();
                fn helper(obj: &Val, args_vec: &mut Vec<AstNode>, node: &AstNode) {
                    match node.get_type() {
                        AstNodeType::ListNode(left, right) => {
                            helper(obj, args_vec, left);
                            helper(obj, args_vec, right);
                        }
                        _ => {
                            args_vec.push(node.clone());
                        }
                    }
                }

                match args {
                    None => {}
                    Some(args) => {
                        helper(&self.val, &mut args_vec, args);
                    }
                };

                self.with_val(self.eval_fcn(name.as_str(), &args_vec))
            }
            AstNodeType::LetStmt { identifier, expr } => {
                let identifier = match identifier.get_type() {
                    AstNodeType::Identifier(identifier) => identifier.clone(),
                    _ => panic!(),
                };

                let val = self.eval(expr).val;
                let mut variables = self.variables.clone();

                variables.insert(identifier.to_string(), val);

                EvalCtx {
                    variables,
                    val: self.val.clone(),
                }
            }
            AstNodeType::Identifier(identifier) => match self.variables.get(identifier) {
                None => self.with_val(Val::new_err("undefined variable access")),
                Some(val) => self.with_val(val.clone()),
            },
            _ => panic!("Unimplemented {:?}", node.get_type()),
        }
    }

    fn eval_list_access(&self, expr: &AstNode) -> Result<(usize, bool), Val> {
        let (is_rev, idx_expr) = match expr.get_type() {
            AstNodeType::ReverseIdx(rev_expr) => (true, rev_expr),
            _ => (false, expr),
        };
        let idx = self.eval(idx_expr).val;
        match idx.get_val() {
            ValType::Float64(num) => {
                let num = *num;
                if num == num.trunc() && num >= 0.0 {
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

    pub fn eval_fcn(&self, name: &str, args: &Vec<AstNode>) -> Val {
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
                            result.push(self.with_val(elem.clone()).eval(&args[0]).val);
                        }
                        Val::new_list(result)
                    }
                    ValType::Map(_) => Val::new_err(
                        "map() must be called on a list. Consider using map_keys() or map_values()",
                    ),
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
                            let key_val = ctx.eval(&args[0]).val;
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
                        let sort_expr = if args.len() == 1 {
                            Some(&args[0])
                        } else {
                            None
                        };
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
                                    ctx.eval(&sort_expr).val
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
                            let filter_val = ctx.eval(&args[0]).val;
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
            "min" => match self.val.get_val() {
                ValType::List(list) => {
                    let mut has_val = false;
                    let mut total: f64 = f64::MAX;
                    for elem in list {
                        match elem.get_val() {
                            ValType::Float64(val) => {
                                total = total.min(*val);
                                has_val = true;
                            }
                            _ => {
                                return Val::new_err(
                                    "min() can only be called on a list of numbers",
                                )
                            }
                        }
                    }

                    if has_val {
                        Val::new_f64(total)
                    } else {
                        Val::new_null()
                    }
                }
                _ => Val::new_err("min() has to be called on a list"),
            },
            "max" => match self.val.get_val() {
                ValType::List(list) => {
                    let mut has_val = false;
                    let mut total: f64 = f64::MIN;
                    for elem in list {
                        match elem.get_val() {
                            ValType::Float64(val) => {
                                total = total.max(*val);
                                has_val = true;
                            }
                            _ => {
                                return Val::new_err(
                                    "max() can only be called on a list of numbers",
                                )
                            }
                        }
                    }

                    if has_val {
                        Val::new_f64(total)
                    } else {
                        Val::new_null()
                    }
                }
                _ => Val::new_err("max() has to be called on a list"),
            },
            "any" => {
                if args.len() != 0 && args.len() != 1 {
                    return Val::new_err("any() must be called with 0 or 1 arguments");
                }

                if args.len() == 1 {
                    return self
                        .eval(&AstNode::new_fcn_call("map", &[&args[0]]))
                        .eval(&AstNode::new_fcn_call("any", &[]))
                        .val;
                }

                match self.val.get_val() {
                    ValType::List(val) => {
                        for elem in val {
                            match elem.get_val() {
                                ValType::Bool(elem) => {
                                    if *elem {
                                        return Val::new_bool(true);
                                    }
                                }
                                _ => {
                                    return Val::new_err(
                                        "any() must be called on a list of booleans",
                                    )
                                }
                            }
                        }
                        return Val::new_bool(false);
                    }
                    _ => return Val::new_err("any() must be called on a list"),
                }
            }
            "all" => {
                if args.len() != 0 && args.len() != 1 {
                    return Val::new_err("all() must be called with 0 or 1 arguments");
                }

                if args.len() == 1 {
                    return self
                        .eval(&AstNode::new_fcn_call("map", &[&args[0]]))
                        .eval(&AstNode::new_fcn_call("all", &[]))
                        .val;
                }

                match self.val.get_val() {
                    ValType::List(val) => {
                        for elem in val {
                            match elem.get_val() {
                                ValType::Bool(elem) => {
                                    if !*elem {
                                        return Val::new_bool(false);
                                    }
                                }
                                _ => {
                                    return Val::new_err(
                                        "all() must be called on a list of booleans",
                                    )
                                }
                            }
                        }
                        return Val::new_bool(true);
                    }
                    _ => return Val::new_err("all() must be called on a list"),
                }
            }
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
                    let text = self.eval_fcn("str", &vec![]);
                    self.with_val(text).eval_fcn(name, args)
                }
                _ => Val::new_err("lines() must be called on a string or bytes"),
            },
            "join_lines" => match self.val.get_val() {
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
                                    "join_lines() must be called on a list of strings",
                                )
                            }
                        }
                    }
                    Val::new_str(result.as_str())
                }
                _ => Val::new_err("join_lines() must be called on a list"),
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
                        match self.eval(&args[0]).val.get_val() {
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
                    let text = self.eval_fcn("str", args);
                    self.with_val(text).eval_fcn(name, args)
                }
                _ => Val::new_err("split() must be called on a string"),
            },
            "join" => match self.val.get_val() {
                ValType::List(elems) => {
                    let joiner = self.eval(&args[0]).val;
                    let joiner = match joiner.get_val() {
                        ValType::String(joiner) => joiner.as_str(),
                        _ => return Val::new_err("join() pattern must be a string"),
                    };

                    let mut strings_to_join = Vec::<String>::with_capacity(elems.len());
                    for elem in elems {
                        match elem.get_val() {
                            ValType::String(elem_str) => {
                                strings_to_join.push(elem_str.clone());
                            }
                            ValType::Float64(_) => {
                                let mut buf = Vec::<u8>::new();
                                match elem.write_to_str(&mut buf, 0, false) {
                                    Err(_) => return Val::new_err("Unable to serialize element"),
                                    Ok(_) => {}
                                };
                                strings_to_join
                                    .push(std::str::from_utf8(buf.as_slice()).unwrap().to_string());
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
            "inj" => {
                // shorhand for in() | from_json()
                self.eval(&AstNode::new(AstNodeType::Pipe(
                    AstNode::new_fcn_call("in", &[]),
                    AstNode::new_fcn_call("from_json", &[]),
                )))
                .val
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
            "write" => {
                if args.len() != 1 {
                    return Val::new_err("write() must be called with 1 argument");
                }

                let file_path = self.eval(&args[0]);

                let file_path = match file_path.val.get_val() {
                    ValType::String(file_path) => file_path,
                    _ => {
                        return Val::new_err("write() file path must be a string");
                    }
                };

                match self.val.get_val() {
                    ValType::Err(_) => self.val.clone(),
                    _ => match std::fs::File::create(file_path) {
                        Err(_) => Val::new_err("Unable to open file"),
                        Ok(mut fp) => match self.val.write_to_str(&mut fp, 0, true) {
                            Err(_) => Val::new_err("Unable to write file"),
                            Ok(_) => Val::new_null(),
                        },
                    },
                }
            }
            "rj" => {
                // shorhand for "read json"
                self.eval(&AstNode::new(AstNodeType::Pipe(
                    AstNode::new_fcn_call("read", &[]),
                    AstNode::new_fcn_call("from_json", &[]),
                )))
                .val
            }
            "env" => {
                let kv_pairs = std::env::vars()
                    .map(|(key, val)| (Val::new_str(key.as_str()), Val::new_str(val.as_str())))
                    .collect::<Vec<_>>();
                Val::new_map(OrderedMap::from_kv_pair_slice(kv_pairs.as_slice()))
            }
            "from_json" => match self.val.get_val() {
                ValType::String(val) => Val::from_json_str(val.as_str()),
                ValType::Bytes(_) => {
                    let text = self.eval_fcn("str", args);
                    self.with_val(text).eval_fcn(name, args)
                }
                _ => Val::new_err("from_json() must be called on a string"),
            },
            "to_json" => match serde_json::to_string(&self.val) {
                Ok(string) => Val::new_str(string.as_str()),
                Err(_) => Val::new_err("Unable to serialize json"),
            },
            "from_toml" => match self.val.get_val() {
                ValType::String(val) => Val::from_toml_str(val.as_str()),
                ValType::Bytes(_) => {
                    let text = self.eval_fcn("str", args);
                    self.with_val(text).eval_fcn(name, args)
                }
                _ => Val::new_err("from_toml() must be called on a string"),
            },
            "to_toml" => match toml::to_string(&self.val) {
                Ok(string) => Val::new_str(string.as_str()),
                Err(_) => Val::new_err("Unable to serialize toml"),
            },
            "from_yaml" => match self.val.get_val() {
                ValType::String(val) => Val::from_yaml_str(val.as_str()),
                ValType::Bytes(_) => {
                    let text = self.eval_fcn("str", args);
                    self.with_val(text).eval_fcn(name, args)
                }
                _ => Val::new_err("from_yaml() must be called on a string"),
            },
            "to_yaml" => match serde_yaml::to_string(&self.val) {
                Ok(string) => Val::new_str(string.as_str()),
                Err(_) => Val::new_err("Unable to serialize yaml"),
            },
            "from_num" => match self.val.get_val() {
                ValType::String(val) => Val::new_f64(match val.parse::<f64>() {
                    Err(_) => return Val::new_err("unable to parse number"),
                    Ok(val) => val,
                }),
                _ => Val::new_err("from_num() must be called on a string"),
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
                _ => Val::new_err("items() must be called on a map"),
            },
            "from_items" => match self.val.get_val() {
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
                                    return Val::new_err("from_items() lists must have 2 elements");
                                }
                                (elems[0].clone(), elems[1].clone())
                            }
                            _ => {
                                return Val::new_err(
                                    "from_items() must be called on a list of maps",
                                )
                            }
                        };
                        kv_pairs.push(kv_pair);
                    }
                    Val::new_map(OrderedMap::from_kv_pair_slice(kv_pairs.as_slice()))
                }
                _ => Val::new_err("from_items() must be called on a map"),
            },
            "recursive_map" => {
                if args.len() != 2 {
                    return Val::new_err("recursive_map() must be called with 2 arguments.");
                }

                let sub_node_getter = &args[0];
                let mapper = args[1].clone();

                fn helper(this: &EvalCtx, sub_node_getter: &AstNode, mapper: &AstNode) -> Val {
                    let sub_nodes = this.eval(sub_node_getter).val;
                    match sub_nodes.get_val() {
                        ValType::List(sub_nodes) => {
                            let mut mapped_sub_nodes = Vec::<Val>::with_capacity(sub_nodes.len());
                            for sub_node in sub_nodes {
                                mapped_sub_nodes.push(helper(
                                    &this.with_val(sub_node.clone()),
                                    sub_node_getter,
                                    mapper,
                                ));
                            }
                            let new_node = Val::new_map(OrderedMap::from_kv_pair_slice(&[
                                (Val::new_str("node"), this.val.clone()),
                                (Val::new_str("vals"), Val::new_list(mapped_sub_nodes)),
                            ]));

                            this.with_val(new_node).eval(mapper).val
                        }
                        ValType::Err(_) => sub_nodes,
                        _ => Val::new_err("mapper function in recursive_map() must return a list"),
                    }
                }

                helper(self, &sub_node_getter, &mapper)
            }
            "recursive_flatten" => {
                if args.len() != 1 {
                    return Val::new_err("recursive_flatten must be called with 1 argument.");
                }

                fn helper(
                    this: &EvalCtx,
                    results: &mut Vec<Val>,
                    sub_node_getter: &AstNode,
                ) -> Result<(), Val> {
                    results.push(this.val.clone());
                    let sub_nodes = this.eval(sub_node_getter).val;
                    match sub_nodes.get_val() {
                        ValType::List(sub_nodes) => {
                            for sub_node in sub_nodes {
                                helper(&this.with_val(sub_node.clone()), results, sub_node_getter)?;
                            }
                            Ok(())
                        }
                        ValType::Err(_) => Err(sub_nodes),
                        _ => Err(Val::new_err(
                            "mapper function in recursive_flatten() must return a list",
                        )),
                    }
                }

                let sub_node_getter = &args[0];
                let mut results = Vec::<Val>::new();
                match helper(self, &mut results, &sub_node_getter) {
                    Ok(_) => {}
                    Err(err) => {
                        return err;
                    }
                }

                Val::new_list(results)
            }
            "exec" => {
                let input = match self.val.get_val() {
                    ValType::Bytes(input) => Some(input.as_slice()),
                    ValType::String(str) => Some(str.as_str().as_bytes()),
                    _ => None,
                };
                if args.len() == 0 {
                    return Val::new_err("exec() has to be called with at least one argument");
                }

                let mut arg_strs = Vec::<String>::new();
                let mut cwd: Option<String> = None;
                for arg in args {
                    match arg.get_type() {
                        AstNodeType::KeywordArgument(keyword, val) => {
                            let keyword = match keyword.get_type() {
                                AstNodeType::Identifier(keyword) => keyword,
                                _ => panic!(),
                            };
                            match keyword.as_str() {
                                "cwd" => {
                                    let cwd_val = self.eval(val).val;
                                    let cwd_val = match cwd_val.get_val() {
                                        ValType::String(cwd_val) => cwd_val,
                                        _ => {
                                            return Val::new_err(
                                                "In exec(), the :env keyword must be a string",
                                            );
                                        }
                                    };
                                    cwd = Some(cwd_val.clone());
                                }
                                _ => return Val::new_err("Unknown keyword passed to exec()"),
                            }
                        }
                        _ => {
                            let arg_val = self.eval(arg).val;
                            let arg_str = match arg_val.get_val() {
                                ValType::String(str) => str,
                                _ => {
                                    return Val::new_err("exec() arguments must be strings");
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
            "range" => {
                let mut step: Option<i64> = None;
                let mut first_idx: Option<i64> = None;
                let mut second_idx: Option<i64> = None;
                for arg in args {
                    match arg.get_type() {
                        AstNodeType::KeywordArgument(keyword, stmt) => {
                            let keyword = match keyword.get_type() {
                                AstNodeType::Identifier(keyword) => keyword,
                                _ => panic!(),
                            };
                            if keyword == "step" {
                                step =
                                    match self.eval_i64(stmt) {
                                        None => return Val::new_err(
                                            ":step keyword argument in range() must be an integer",
                                        ),
                                        Some(step) => Some(step),
                                    };
                            } else {
                                return Val::new_err("Invalid keyword argument in range()");
                            }
                        }
                        _ => {
                            match first_idx {
                                None => {
                                    first_idx = match self.eval_i64(arg) {
                                        None => {
                                            return Val::new_err(
                                                "first argument in range() must be an integer",
                                            )
                                        }
                                        Some(idx) => Some(idx),
                                    };
                                }
                                Some(_) => {
                                    match second_idx {
                                        None => {
                                            second_idx = match self.eval_i64(arg) {
                                                None => return Val::new_err(
                                                    "second argument in range() must be an integer",
                                                ),
                                                Some(idx) => Some(idx),
                                            };
                                        }
                                        Some(_) => return Val::new_err(
                                            "range() cannot be called with more than 2 arguments",
                                        ),
                                    }
                                }
                            }
                        }
                    }
                }

                let (start, end) = match first_idx {
                    None => {
                        return Val::new_err("range() must be called with at least one argument")
                    }
                    Some(first_idx) => match second_idx {
                        None => (0, first_idx),
                        Some(second_idx) => (first_idx, second_idx),
                    },
                };
                let step = match step {
                    None => {
                        if start <= end {
                            1
                        } else {
                            -1
                        }
                    }
                    Some(step) => step,
                };

                if step == 0 {
                    return Val::new_err(":step keyword argument in range() cannot be zero.");
                }

                let mut result = Vec::<Val>::new();
                if step > 0 {
                    let mut idx = start;
                    while idx < end {
                        result.push(Val::new_f64(idx as f64));
                        idx += step;
                    }
                } else {
                    let mut idx = start;
                    while idx > end {
                        result.push(Val::new_f64(idx as f64));
                        idx += step;
                    }
                }

                Val::new_list(result)
            }
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
            "combinations" => {
                if args.len() != 0 {
                    return Val::new_err("combinations() must be called with zero arguments");
                }
                let arg_vals = match self.val.get_val() {
                    ValType::List(vals) => vals,
                    _ => return Val::new_err("combinations() must be called on a list"),
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

                let mut indexes = vec![0 as usize; arg_lists.len()];

                let mut results = Vec::<Val>::new();

                loop {
                    let mut any_exceed = false;
                    for (l_idx, idx) in indexes.iter().enumerate().rev() {
                        if *idx >= arg_lists[l_idx].len() {
                            any_exceed = true;
                            break;
                        }
                    }
                    if any_exceed {
                        break;
                    }

                    let mut result = Vec::<Val>::new();
                    for (l_idx, idx) in indexes.iter().enumerate() {
                        result.push(arg_lists[l_idx][*idx].clone());
                    }

                    results.push(Val::new_list(result));

                    let mut incr_next = true;
                    for (l_idx, idx) in indexes.iter_mut().enumerate().rev() {
                        if incr_next {
                            *idx += 1;

                            incr_next = *idx >= arg_lists[l_idx].len();
                            if incr_next {
                                *idx = 0;
                            }
                        } else {
                            break;
                        }
                    }

                    if incr_next {
                        break;
                    }
                }

                Val::new_list(results)
            }
            "repeat" => {
                if args.len() != 1 {
                    return Val::new_err("repeat() has to be called with one argument");
                }

                let arg_val = self.eval(&args[0]).val;
                match arg_val.get_val() {
                    ValType::Float64(val) => {
                        let val = *val;
                        if val == val.trunc() && val >= 0.0 {
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
            "is_err" => match self.val.get_val() {
                ValType::Err(_) => Val::new_bool(true),
                _ => Val::new_bool(false),
            },
            "is_number" => match self.val.get_val() {
                ValType::Float64(_) => Val::new_bool(true),
                _ => Val::new_bool(false),
            },
            "is_bool" => match self.val.get_val() {
                ValType::Bool(_) => Val::new_bool(true),
                _ => Val::new_bool(false),
            },
            "is_string" => match self.val.get_val() {
                ValType::String(_) => Val::new_bool(true),
                _ => Val::new_bool(false),
            },
            "is_list" => match self.val.get_val() {
                ValType::List(_) => Val::new_bool(true),
                _ => Val::new_bool(false),
            },
            "is_map" => match self.val.get_val() {
                ValType::Map(_) => Val::new_bool(true),
                _ => Val::new_bool(false),
            },
            "is_bytes" => match self.val.get_val() {
                ValType::Bytes(_) => Val::new_bool(true),
                _ => Val::new_bool(false),
            },
            "from_text_table" => match self.val.get_val() {
                ValType::String(_) => {
                    let lines = self.eval_fcn("lines", &Vec::new());

                    let split_lines = match lines.get_val() {
                        ValType::Err(_) => {
                            return lines;
                        }
                        ValType::List(val) => {
                            if val.len() == 0 {
                                return Val::new_err("from_text_table() has to be called on a string with at least one line.");
                            }
                            val.iter()
                                .map(|elem| {
                                    self.with_val(elem.clone()).eval_fcn("split", &Vec::new())
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
                    let text = self.eval_fcn("str", args);
                    self.with_val(text).eval_fcn(name, args)
                }
                _ => Val::new_err("from_text_table() must be called on a string"),
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
            "from_csv" => {
                let mut reader = match self.val.get_val() {
                    ValType::Bytes(bytes) => csv::Reader::from_reader(bytes.as_slice()),
                    ValType::String(val) => csv::Reader::from_reader(val.as_bytes()),
                    _ => return Val::new_err("from_csv() must be called on string or bytes"),
                };

                let mut lines = Vec::<Val>::new();

                if reader.has_headers() {
                    match reader.headers() {
                        Err(_) => return Val::new_err("from_csv() is unable to read csv"),
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
                        Err(_) => return Val::new_err("from_csv() is unable to read csv"),
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
            "to_csv" => write_csv_to_val(&self.val, b',', "to_csv"),
            "to_tsv" => write_csv_to_val(&self.val, b'\t', "to_tsv"),
            "catch" => {
                if args.len() != 1 {
                    return Val::new_err("catch() must be called with one argument");
                }
                match self.val.get_val() {
                    ValType::Err(_) => self.eval(&args[0]).val,
                    _ => self.val.clone(),
                }
            }
            "if" => {
                if args.len() != 3 {
                    return Val::new_err("if() must be called with 3 arguments");
                }
                let cond = match self.eval(&args[0]).val.get_val() {
                    ValType::Bool(val) => *val,
                    _ => return Val::new_err("first argument in if() must be a boolean"),
                };

                if cond {
                    self.eval(&args[1]).val
                } else {
                    self.eval(&args[2]).val
                }
            }
            "has" => {
                if args.len() != 1 {
                    return Val::new_err("has() must be called with 1 argument");
                }

                match self.val.get_val() {
                    ValType::Map(val) => {
                        let key = self.eval(&args[0]).val;
                        Val::new_bool(val.has(&key))
                    }
                    ValType::List(vals) => {
                        let key = self.eval(&args[0]).val;
                        for elem in vals {
                            if *elem == key {
                                return Val::new_bool(true);
                            }
                        }
                        Val::new_bool(false)
                    }
                    _ => return Val::new_err("has() must be called on a list or map"),
                }
            }
            "map_keys" => {
                if args.len() != 1 {
                    return Val::new_err("map_keys() must be called with 1 argument");
                }

                match self.val.get_val() {
                    ValType::Map(val) => {
                        let mut result = OrderedMap::new();
                        for (key, val) in val.get_kv_pair_slice() {
                            let new_key = self.with_val(key.clone()).eval(&args[0]).val;
                            result.insert(&new_key, val);
                        }

                        Val::new_map(result)
                    }
                    _ => return Val::new_err("map_keys() must be called on a map"),
                }
            }
            "map_values" => {
                if args.len() != 1 {
                    return Val::new_err("map_values() must be called with 1 argument");
                }

                match self.val.get_val() {
                    ValType::Map(val) => {
                        let mut result = OrderedMap::new();
                        for (key, val) in val.get_kv_pair_slice() {
                            let new_val = self.with_val(val.clone()).eval(&args[0]).val;
                            result.insert(key, &new_val);
                        }

                        Val::new_map(result)
                    }
                    _ => return Val::new_err("map_values() must be called on a map"),
                }
            }
            "starts_with" => match self.val.get_val() {
                ValType::String(val) => {
                    if args.len() != 1 {
                        return Val::new_err("starts_with() must be called with one argument");
                    }

                    match self.eval(&args[0]).val.get_val() {
                        ValType::String(test) => Val::new_bool(val.starts_with(test)),
                        _ => return Val::new_err("starts_with() must be called with a string"),
                    }
                }
                _ => return Val::new_err("starts_with() must be called on a string."),
            },
            "ends_with" => match self.val.get_val() {
                ValType::String(val) => {
                    if args.len() != 1 {
                        return Val::new_err("ends_with() must be called with one argument");
                    }

                    match self.eval(&args[0]).val.get_val() {
                        ValType::String(test) => Val::new_bool(val.ends_with(test)),
                        _ => return Val::new_err("ends_with() must be called with a string"),
                    }
                }
                _ => return Val::new_err("ends_with() must be called on a string."),
            },
            "lower" => match self.val.get_val() {
                ValType::String(val) => Val::new_str(val.to_lowercase().as_str()),
                _ => Val::new_err("lower() must be called on a string."),
            },
            "upper" => match self.val.get_val() {
                ValType::String(val) => Val::new_str(val.to_uppercase().as_str()),
                _ => Val::new_err("upper() must be called on a string."),
            },
            "trim" => match self.val.get_val() {
                ValType::String(val) => Val::new_str(val.trim()),
                _ => Val::new_err("trim() must be called on a string"),
            },
            "abs" => match self.val.get_val() {
                ValType::Float64(val) => Val::new_f64(val.abs()),
                _ => Val::new_err("abs() must be called on a number"),
            },
            _ => Val::new_err(format!("Unknown function \"{}\"", name).as_str()),
        }
    }
}

fn write_csv_to_val(val: &Val, delimiter: u8, fcn_name: &str) -> Val {
    let mut buffer = Vec::<u8>::new();
    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(&mut buffer);
    match val.get_val() {
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
                                    let buf = std::str::from_utf8(buf.as_slice()).unwrap();
                                    record.push_field(buf);
                                }
                            }
                        }
                    }
                    _ => {
                        return Val::new_err(
                            format!("{}() must be called on a list of lists", fcn_name).as_str(),
                        )
                    }
                }

                writer.write_record(record.iter()).unwrap();
            }
        }
        _ => return Val::new_err(format!("{}() must be called on a list", fcn_name).as_str()),
    }

    drop(writer);

    Val::new_bytes(buffer)
}
