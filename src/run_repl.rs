use crate::EvalCtx;

use std::error::Error;

pub fn run_repl() -> Result<(), Box<dyn Error>> {
    let mut rl = rustyline::DefaultEditor::new()?;
    let mut stdout = std::io::stdout();
    let mut ctx = EvalCtx::new();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                if line == "quit" {
                    return Ok(());
                }
                rl.add_history_entry(&line)?;
                ctx = ctx.parse_and_eval(line.as_str());

                EvalCtx::write_val(&ctx.val, &mut stdout, true)?;
            }
            Err(_) => {
                return Ok(());
            }
        }
    }
}
