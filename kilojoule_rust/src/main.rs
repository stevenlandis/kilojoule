use kilojoule_rust::{eval_ast_node, Parser, Val};

fn main() {
    let parser = Parser::new();
    let ast = parser.parse(".");
    println!("Ast: {:?}", ast);
    let result = eval_ast_node(&Val::from_json_str(r#"{"a": 1, "b": 2}"#), &ast);
    println!("Result = {:?}", result);
}
