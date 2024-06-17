use crate::EvalCtx;

use std::error::Error;

pub fn run_repl() -> Result<(), Box<dyn Error>> {
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
                let result = EvalCtx::parse_and_eval(line.as_str());

                EvalCtx::write_val(result, &mut stdout, true)?;
            }
            Err(_) => {
                return Ok(());
            }
        }
    }
}
