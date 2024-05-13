use kilojoule_rust::{eval_ast_node, Parser, Val};

fn main() {
    let parser = Parser::new();
    let mut args = std::env::args();
    if args.len() != 2 {
        println!("Please call with a single argument.");
        std::process::exit(1);
    }
    let query = args.nth(1).unwrap();
    let ast = parser.parse(query.as_str());
    println!("Ast: {:?}", ast);
    let result = eval_ast_node(&Val::new_null(), &ast);
    println!("Result = {:?}", result);
}
