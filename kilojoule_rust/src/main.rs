use kilojoule_rust::{eval_ast_node, run_repl, Parser, Val};

fn main() {
    let parser = Parser::new();
    let mut args = std::env::args();
    if args.len() != 2 {
        let _ = run_repl(&parser);
        return;
    }
    let query = args.nth(1).unwrap();
    let ast = parser.parse(query.as_str());
    // println!("Ast: {:?}", ast);
    let result = eval_ast_node(&Val::new_null(), &ast);
    result.write_json_str(&mut std::io::stdout(), true);
    // println!("Result = {:?}", result);
}
