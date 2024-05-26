use std::collections::HashMap;

use super::evaluate::eval_ast_node;
use super::parser::Parser;
use super::val::Val;

pub fn run_repl(parser: &Parser) -> Result<(), rustyline::error::ReadlineError> {
    let mut rl = rustyline::DefaultEditor::new()?;
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                if line == "quit" {
                    return Ok(());
                }
                rl.add_history_entry(&line)?;
                let result = match parser.parse(line.as_str()) {
                    Ok(ast) => eval_ast_node(&Val::new_null(), &ast, &HashMap::new()),
                    Err(err) => Val::new_err(err.message.as_str()),
                };
                result.write_json_str(&mut std::io::stdout(), true);
            }
            Err(_) => {
                return Ok(());
            }
        }
    }
}
