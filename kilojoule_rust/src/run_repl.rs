use crate::Evaluator;

use std::error::Error;

pub fn run_repl() -> Result<(), Box<dyn Error>> {
    let mut evaluator = Evaluator::new();
    let mut rl = rustyline::DefaultEditor::new()?;
    let mut stdout = std::io::stdout();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                if line == "quit" {
                    return Ok(());
                }
                rl.add_history_entry(&line)?;
                let result = evaluator.parse_and_eval(line.as_str());

                evaluator.write_val(result, &mut stdout, true)?;
            }
            Err(_) => {
                return Ok(());
            }
        }
    }
}
