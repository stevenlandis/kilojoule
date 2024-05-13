use super::ast_node::AstNode;
use super::val::{Val, ValType};

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
            _ => {
                panic!("Access on invalid object");
            }
        },
        AstNode::StringLiteral(val) => Val::new_string(val),
        AstNode::Pipe(left, right) => eval_ast_node(&eval_ast_node(obj, left), right),
        _ => {
            panic!("Unimplemented eval for node={:?}", node);
        }
    }
}
