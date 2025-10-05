use std::collections::HashMap;

use super::ast_node::{AstNode, AstNodeType};
use super::parser::Parser;
use super::val::{OrderedMap, Val, ValType};

mod eval_fcn;

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
            AstNodeType::IntType => self.with_val(Val::new(ValType::IntType)),
            AstNodeType::FloatType => self.with_val(Val::new(ValType::FloatType)),
            AstNodeType::AnyType => self.with_val(Val::new(ValType::AnyType)),
            AstNodeType::StringType => self.with_val(Val::new(ValType::StringType)),
            AstNodeType::BoolType => self.with_val(Val::new(ValType::BoolType)),
            AstNodeType::ListType(elem_type) => {
                self.with_val(Val::new(ValType::ListType(self.eval(elem_type).val)))
            }
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
}
