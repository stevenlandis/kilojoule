use std::collections::HashMap;

use super::ast_node::AstNode;
use super::val::{Val, ValHashMap, ValType};

pub fn eval_ast_node(obj: &Val, node: &AstNode) -> Val {
    match node {
        AstNode::Echo => obj.clone(),
        AstNode::Access(expr) => match &obj.val.val {
            ValType::Map(map) => {
                let key = eval_ast_node(obj, expr);
                match map.get(&key) {
                    None => Val::new_null(),
                    Some(val) => val.clone(),
                }
            }
            ValType::List(list) => {
                let key = eval_ast_node(obj, expr);
                match key.val.val {
                    ValType::Number(num) => {
                        if num == num.floor() {
                            return list[num as usize].clone();
                        } else {
                            panic!("Can only access a list with an integer.")
                        }
                    }
                    _ => {
                        panic!("Can only access a list with an integer.")
                    }
                }
            }
            _ => {
                panic!("Access on invalid object");
            }
        },
        AstNode::StringLiteral(val) => Val::new_string(val),
        AstNode::F64Literal(val) => Val::new_number(*val),
        AstNode::Bool(val) => Val::new_bool(*val),
        AstNode::Pipe(left, right) => eval_ast_node(&eval_ast_node(obj, left), right),
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
                            map.insert(&eval_ast_node(obj, key), &eval_ast_node(obj, value));
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
                    vals.push(eval_ast_node(obj, elem));
                }
                return Val::new_list(vals.as_slice());
            }
        },
        AstNode::FormatStringNode(parts) => {
            let mut write_buf = Vec::<u8>::new();
            for part in parts {
                let result = eval_ast_node(obj, part);
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

            return evaluate_fcn(fcn_name.as_str(), &args_vec, obj);
        }
        _ => {
            panic!("Unimplemented eval for node={:?}", node);
        }
    }
}

fn evaluate_fcn(fcn_name: &str, args: &Vec<&AstNode>, obj: &Val) -> Val {
    match fcn_name {
        "len" => match &obj.val.val {
            ValType::List(list) => Val::new_number(list.len() as f64),
            ValType::Map(map) => Val::new_number(map.len() as f64),
            _ => Val::new_err("Len has to be called on a list or map."),
        },
        "map" => {
            if args.len() != 1 {
                return Val::new_err("map() must be called with one argument");
            }
            match &obj.val.val {
                ValType::List(list) => {
                    let mut result = Vec::<Val>::with_capacity(list.len());
                    for elem in list {
                        result.push(eval_ast_node(elem, args[0]));
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
                        let group_key = eval_ast_node(elem, args[0]);
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
        _ => Val::new_err("Function does not exist."),
    }
}
