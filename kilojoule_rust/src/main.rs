use kilojoule_rust::{AstNode, Parser};

fn main() {
    let parser = Parser::new();
    let ast = parser.parse("2*3 + 30*4 + 1 * 2 + 3");
    println!("Ast: {:?}", ast);
    let result = eval_ast_node(&ast);
    println!("Result = {}", result);
}

fn eval_ast_node(node: &AstNode) -> i64 {
    match node {
        AstNode::Int(val) => *val as i64,
        AstNode::Add(left, right) => eval_ast_node(&left) + eval_ast_node(&right),
        AstNode::Mul(left, right) => eval_ast_node(&left) * eval_ast_node(&right),
        _ => {
            panic!("Unimplemented eval");
        }
    }
}
