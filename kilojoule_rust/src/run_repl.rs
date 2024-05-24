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
                let ast = parser.parse(line.as_str());
                let result = eval_ast_node(&Val::new_null(), &ast);
                result.write_json_str(&mut std::io::stdout(), true);
            }
            Err(_) => {
                return Ok(());
            }
        }
    }
}
