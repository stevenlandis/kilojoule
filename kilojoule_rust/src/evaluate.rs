use super::ast_node::AstNode;
use super::val::Val;

pub fn eval_ast_node(obj: &Val, node: &AstNode) -> Val {
    match node {
        AstNode::Echo => obj.clone(),
        _ => {
            panic!("Unimplemented eval for node={:?}", node);
        }
    }
}
