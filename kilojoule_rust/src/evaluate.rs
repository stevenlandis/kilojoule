use std::collections::HashMap;
use std::io::Read;
use std::rc::Rc;

use super::ast_node::AstNode;
use super::val::{Val, ValHashMap, ValType};

type Variables = HashMap<String, Val>;

pub fn eval_ast_node(obj: &Val, node: &AstNode, vars: &Variables) -> Val {
    match node {
        AstNode::Echo => obj.clone(),
        AstNode::Access(expr) => match &obj.val.val {
            ValType::Map(map) => {
                let key = eval_ast_node(obj, expr, vars);
                match map.get(&key) {
                    None => Val::new_null(),
                    Some(val) => val.clone(),
                }
            }
            ValType::List(list) => match &**expr {
                AstNode::SliceAccess(start, end) => match &obj.val.val {
                    ValType::List(list) => {
                        let start_idx = match start {
                            None => 0,
                            Some(start_expr) => match eval_list_access(obj, &start_expr, vars) {
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
                            },
                        };
                        let end_idx = match end {
                            None => list.len(),
                            Some(end_expr) => match eval_list_access(obj, &end_expr, vars) {
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
                        Val::new_list(&list[start_idx..end_idx])
                    }
                    _ => Val::new_err("Access on invalid object"),
                },
                _ => match eval_list_access(obj, expr, vars) {
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
            _ => Val::new_err("Access on invalid object"),
        },
        AstNode::StringLiteral(val) => Val::new_string(val),
        AstNode::F64Literal(val) => Val::new_number(*val),
        AstNode::Bool(val) => Val::new_bool(*val),
        AstNode::Null => Val::new_null(),
        AstNode::Pipe(left, right) => eval_ast_node(&eval_ast_node(obj, left, vars), right, vars),
        AstNode::Assign(var_name, val_expr, right) => {
            let value = eval_ast_node(obj, val_expr, vars);
            let mut next_vars = vars.clone();
            next_vars.insert(var_name.clone(), value.clone());
            eval_ast_node(obj, right, &next_vars)
        }
        AstNode::VarAccess(var_name) => match vars.get(var_name) {
            None => Val::new_err("Undefined variable access"),
            Some(val) => val.clone(),
        },
        AstNode::MapLiteral(elems_opt) => match elems_opt {
            None => Val::new_map_from_entries_iter(Vec::new()),
            Some(elems_node) => {
                let mut elems = Vec::<&AstNode>::new();
                let mut node = &**elems_node;
                loop {
                    match node {
                        AstNode::MapElemListNode(more_elems, elem) => {
                            elems.push(elem);
                            node = more_elems
                        }
                        _ => {
                            elems.push(node);
                            break;
                        }
                    }
                }
                elems.reverse();

                let mut map = ValHashMap::new();
                for elem in elems {
                    match elem {
                        AstNode::MapKeyValPair(key, value) => {
                            map.insert(
                                &eval_ast_node(obj, key, vars),
                                &eval_ast_node(obj, value, vars),
                            );
                        }
                        _ => {
                            panic!("Unimplemented map elem {:?}", elem)
                        }
                    }
                }

                return Val::new_map(map);
            }
        },
        AstNode::ListLiteral(list_contents) => match list_contents {
            None => Val::new_list(&[]),
            Some(list_contents) => {
                let mut elems = Vec::<&AstNode>::new();
                let mut node = &**list_contents;
                loop {
                    match node {
                        AstNode::ListElemListNode(more_elems, elem) => {
                            elems.push(elem);
                            node = more_elems;
                        }
                        _ => {
                            elems.push(node);
                            break;
                        }
                    }
                }
                elems.reverse();

                let mut vals = Vec::<Val>::new();
                for elem in elems {
                    vals.push(eval_ast_node(obj, elem, vars));
                }
                return Val::new_list(vals.as_slice());
            }
        },
        AstNode::FormatStringNode(parts) => {
            let mut write_buf = Vec::<u8>::new();
            for part in parts {
                let result = eval_ast_node(obj, part, vars);
                match &result.val.val {
                    ValType::String(part_text) => {
                        write_buf.extend(part_text.as_bytes());
                    }
                    _ => {
                        result.write_json_str(&mut write_buf, false);
                    }
                }
            }
            return Val::new_string(std::str::from_utf8(write_buf.as_slice()).unwrap());
        }
        AstNode::FcnCall(fcn_name, args) => {
            let mut args_vec = Vec::<&AstNode>::new();
            match args {
                None => {}
                Some(args) => {
                    let mut node = &**args;
                    loop {
                        match node {
                            AstNode::FcnCallArgNode(more_elems, elem) => {
                                args_vec.push(elem);
                                node = more_elems;
                            }
                            _ => {
                                args_vec.push(node);
                                break;
                            }
                        }
                    }
                    args_vec.reverse();
                }
            }

            let fcn_name = match &**fcn_name {
                AstNode::StringLiteral(val) => val.clone(),
                _ => {
                    panic!("unreachable");
                }
            };

            return evaluate_fcn(fcn_name.as_str(), &args_vec, obj, vars);
        }
        AstNode::Add(left, right) => {
            let left = eval_ast_node(obj, left, vars);
            let right = eval_ast_node(obj, right, vars);
            match &left.val.val {
                ValType::Number(left) => match &right.val.val {
                    ValType::Number(right) => Val::new_number(left + right),
                    ValType::Error(_) => right,
                    _ => Val::new_err("Right side of add has to be a number"),
                },
                ValType::Error(_) => left,
                _ => Val::new_err("Left side of add has to be a number"),
            }
        }
        AstNode::Subtract(left, right) => {
            let left = eval_ast_node(obj, left, vars);
            let right = eval_ast_node(obj, right, vars);
            match &left.val.val {
                ValType::Number(left) => match &right.val.val {
                    ValType::Number(right) => Val::new_number(left - right),
                    ValType::Error(_) => right,
                    _ => Val::new_err("Right side of subtraction has to be a number"),
                },
                ValType::Error(_) => left,
                _ => Val::new_err("Left side of subtraction has to be a number"),
            }
        }
        AstNode::Equals(left, right) => {
            let left = eval_ast_node(obj, left, vars);
            let right = eval_ast_node(obj, right, vars);
            Val::new_bool(left == right)
        }
        AstNode::NotEqual(left, right) => {
            let left = eval_ast_node(obj, left, vars);
            let right = eval_ast_node(obj, right, vars);
            Val::new_bool(left != right)
        }
        AstNode::LessThan(left, right) => {
            let left = eval_ast_node(obj, left, vars);
            let right = eval_ast_node(obj, right, vars);
            Val::new_bool(left < right)
        }
        AstNode::LessThanOrEqual(left, right) => {
            let left = eval_ast_node(obj, left, vars);
            let right = eval_ast_node(obj, right, vars);
            Val::new_bool(left <= right)
        }
        AstNode::GreaterThan(left, right) => {
            let left = eval_ast_node(obj, left, vars);
            let right = eval_ast_node(obj, right, vars);
            Val::new_bool(left > right)
        }
        AstNode::GreaterThanOrEqual(left, right) => {
            let left = eval_ast_node(obj, left, vars);
            let right = eval_ast_node(obj, right, vars);
            Val::new_bool(left >= right)
        }
        AstNode::Or(left, right) => {
            let left = eval_ast_node(obj, left, vars);
            let right = eval_ast_node(obj, right, vars);
            match &left.val.val {
                ValType::Bool(left) => match &right.val.val {
                    ValType::Bool(right) => Val::new_bool(*left || *right),
                    ValType::Error(_) => right,
                    _ => Val::new_err("Right side of OR has to be a boolean"),
                },
                ValType::Error(_) => left,
                _ => Val::new_err("Left side of OR has to be a boolean"),
            }
        }
        AstNode::And(left, right) => {
            let left = eval_ast_node(obj, left, vars);
            let right = eval_ast_node(obj, right, vars);
            match &left.val.val {
                ValType::Bool(left) => match &right.val.val {
                    ValType::Bool(right) => Val::new_bool(*left && *right),
                    ValType::Error(_) => right,
                    _ => Val::new_err("Right side of AND has to be a boolean"),
                },
                ValType::Error(_) => left,
                _ => Val::new_err("Left side of AND has to be a boolean"),
            }
        }
        AstNode::Multiply(left, right) => {
            let left = eval_ast_node(obj, left, vars);
            let right = eval_ast_node(obj, right, vars);
            match &left.val.val {
                ValType::Number(left) => match &right.val.val {
                    ValType::Number(right) => Val::new_number(left * right),
                    ValType::Error(_) => right,
                    _ => Val::new_err("Right side of multiply has to be a number"),
                },
                ValType::Error(_) => left,
                _ => Val::new_err("Left side of multiply has to be a number"),
            }
        }
        AstNode::Divide(left, right) => {
            let left = eval_ast_node(obj, left, vars);
            let right = eval_ast_node(obj, right, vars);
            match &left.val.val {
                ValType::Number(left) => match &right.val.val {
                    ValType::Number(right) => {
                        if *right == 0.0 {
                            Val::new_err("divide by zero")
                        } else {
                            Val::new_number(left / right)
                        }
                    }
                    ValType::Error(_) => right,
                    _ => Val::new_err("Right side of divide has to be a number"),
                },
                ValType::Error(_) => left,
                _ => Val::new_err("Left side of divide has to be a number"),
            }
        }
        _ => {
            panic!("Unimplemented eval for node={:?}", node);
        }
    }
}

fn evaluate_fcn(fcn_name: &str, args: &Vec<&AstNode>, obj: &Val, vars: &Variables) -> Val {
    match fcn_name {
        "len" => match &obj.val.val {
            ValType::List(list) => Val::new_number(list.len() as f64),
            ValType::Map(map) => Val::new_number(map.len() as f64),
            ValType::String(string) => Val::new_number(string.as_bytes().len() as f64),
            ValType::Bytes(bytes) => Val::new_number(bytes.len() as f64),
            _ => Val::new_err("len() called on unsupported object"),
        },
        "map" => {
            if args.len() != 1 {
                return Val::new_err("map() must be called with one argument");
            }
            match &obj.val.val {
                ValType::List(list) => {
                    let mut result = Vec::<Val>::with_capacity(list.len());
                    for elem in list {
                        result.push(eval_ast_node(elem, args[0], vars));
                    }
                    Val::new_list(result.as_slice())
                }
                _ => Val::new_err("map() must be called on a list"),
            }
        }
        "group" => {
            if args.len() != 1 {
                return Val::new_err("group() must be called with 1 argument");
            }
            match &obj.val.val {
                ValType::List(list) => {
                    let mut groups = Vec::<(Val, Vec<Val>)>::new();
                    let mut val_to_idx = HashMap::<Val, usize>::new();
                    for elem in list {
                        let group_key = eval_ast_node(elem, args[0], vars);
                        let group_idx =
                            *val_to_idx.entry(group_key.clone()).or_insert(groups.len());
                        if group_idx == groups.len() {
                            groups.push((group_key, vec![]));
                        }
                        groups[group_idx].1.push(elem.clone());
                    }

                    let mut results = Vec::<Val>::with_capacity(groups.len());

                    for (group_key, vals) in groups {
                        let pairs: Vec<(Val, Val)> = vec![
                            (Val::new_string("key"), group_key),
                            (Val::new_string("rows"), Val::new_list(vals.as_slice())),
                        ];
                        let map = ValHashMap::from_pairs(&pairs);
                        results.push(Val::new_map(map));
                    }

                    Val::new_list(results.as_slice())
                }
                _ => Val::new_err("group() must be called on a list"),
            }
        }
        "unique" => eval_ast_node(
            obj,
            &AstNode::Pipe(
                Rc::new(AstNode::FcnCall(
                    Rc::new(AstNode::StringLiteral("group".to_string())),
                    Some(Rc::new(AstNode::Echo)),
                )),
                Rc::new(AstNode::FcnCall(
                    Rc::new(AstNode::StringLiteral("map".to_string())),
                    Some(Rc::new(AstNode::Access(Rc::new(AstNode::StringLiteral(
                        "key".to_string(),
                    ))))),
                )),
            ),
            vars,
        ),
        "sort" => match &obj.val.val {
            ValType::List(list) => {
                let mut result = list.iter().cloned().collect::<Vec<_>>();
                result.sort();
                Val::new_list(result.as_slice())
            }
            _ => Val::new_err("sort() has to be called on a list."),
        },
        "filter" => match &obj.val.val {
            ValType::List(list) => {
                let mut result = Vec::<Val>::new();
                for elem in list {
                    if match eval_ast_node(elem, args[0], vars).val.val {
                        ValType::Bool(val) => val,
                        _ => false,
                    } {
                        result.push(elem.clone());
                    }
                }
                return Val::new_list(result.as_slice());
            }
            _ => Val::new_err("filter() has to be called on a list."),
        },
        "sum" => match &obj.val.val {
            ValType::List(list) => {
                let mut total: f64 = 0.0;
                for elem in list {
                    match elem.val.val {
                        ValType::Number(val) => {
                            total += val;
                        }
                        _ => {}
                    }
                }
                Val::new_number(total)
            }
            _ => Val::new_err("sum() has to be called on a list"),
        },
        "lines" => match &obj.val.val {
            ValType::String(val) => {
                let mut lines = val
                    .split("\n")
                    .map(|line| Val::new_string(line))
                    .collect::<Vec<_>>();
                if lines.len() > 0
                    && match &lines[lines.len() - 1].val.val {
                        ValType::String(text) => text == "",
                        _ => panic!(),
                    }
                {
                    lines.pop();
                }
                Val::new_list(lines.as_slice())
            }
            ValType::Bytes(_) => eval_ast_node(
                obj,
                &AstNode::Pipe(
                    Rc::new(AstNode::FcnCall(
                        Rc::new(AstNode::StringLiteral("str".to_string())),
                        None,
                    )),
                    Rc::new(AstNode::FcnCall(
                        Rc::new(AstNode::StringLiteral("lines".to_string())),
                        None,
                    )),
                ),
                vars,
            ),
            _ => Val::new_err("lines() must be called on a string"),
        },
        "in" => {
            let mut buffer = Vec::<u8>::new();
            std::io::stdin().read_to_end(&mut buffer).unwrap();
            return Val::new_bytes(buffer);
        }
        "str" => match &obj.val.val {
            ValType::Bytes(bytes) => match std::str::from_utf8(bytes) {
                Ok(str) => Val::new_string(str),
                Err(_) => Val::new_err("Unable to decode bytes as utf8 string."),
            },
            _ => Val::new_err("str() must be called on bytes"),
        },
        "bytes" => match &obj.val.val {
            ValType::String(text) => {
                Val::new_bytes(text.as_bytes().iter().cloned().collect::<Vec<_>>())
            }
            _ => Val::new_err("bytes() must be called on str"),
        },
        "split" => match &obj.val.val {
            ValType::String(text) => match &eval_ast_node(obj, args[0], vars).val.val {
                ValType::String(split_pattern) => Val::new_list(
                    text.split(split_pattern)
                        .map(|elem| Val::new_string(elem))
                        .collect::<Vec<_>>()
                        .as_slice(),
                ),
                _ => Val::new_err("split() pattern must be a string"),
            },
            ValType::Bytes(_) => evaluate_fcn(
                fcn_name,
                args,
                &eval_ast_node(
                    obj,
                    &AstNode::FcnCall(Rc::new(AstNode::StringLiteral("str".to_string())), None),
                    vars,
                ),
                vars,
            ),
            _ => Val::new_err("split() must be called on a string"),
        },
        "join" => match &obj.val.val {
            ValType::List(elems) => {
                let joiner = eval_ast_node(obj, args[0], vars);
                let joiner = match &joiner.val.val {
                    ValType::String(joiner) => joiner.as_str(),
                    _ => return Val::new_err("join() pattern must be a string"),
                };

                let mut strings_to_join = Vec::<&str>::with_capacity(elems.len());
                for elem in elems {
                    match &elem.val.val {
                        ValType::String(elem_str) => {
                            strings_to_join.push(elem_str.as_str());
                        }
                        _ => {
                            return Val::new_err("All elements passed to join() must be strings");
                        }
                    }
                }

                let result = strings_to_join.join(joiner);
                Val::new_string(result.as_str())
            }
            _ => Val::new_err("join() must be called on a list"),
        },
        "env" => {
            let kv_pairs = std::env::vars()
                .map(|(key, val)| (Val::new_string(key.as_str()), Val::new_string(val.as_str())))
                .collect::<Vec<_>>();
            Val::new_map_from_entries_iter(kv_pairs)
        }
        "read" => match &obj.val.val {
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
        _ => Val::new_err("Function does not exist."),
    }
}

fn eval_list_access(obj: &Val, expr: &Rc<AstNode>, vars: &Variables) -> Result<(usize, bool), Val> {
    let (is_rev, idx_expr) = match &**expr {
        AstNode::ReverseIdx(rev_expr) => (true, rev_expr),
        _ => (false, expr),
    };
    let idx = eval_ast_node(obj, idx_expr, vars);
    match idx.val.val {
        ValType::Number(num) => {
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
