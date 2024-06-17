use kilojoule::{run_repl, EvalCtx};

fn main() {
    let mut args = std::env::args();
    if args.len() != 2 {
        let _ = run_repl();
        return;
    }
    let query = args.nth(1).unwrap();
    let result = EvalCtx::parse_and_eval(query.as_str());
    let _ = EvalCtx::write_val(result, &mut std::io::stdout(), true);
}
