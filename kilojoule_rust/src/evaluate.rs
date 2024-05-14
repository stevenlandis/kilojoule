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
        _ => {
            panic!("Unimplemented eval for node={:?}", node);
        }
    }
}
