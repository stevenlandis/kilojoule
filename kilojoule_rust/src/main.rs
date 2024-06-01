use kilojoule_rust::{run_repl, Evaluator};

fn main() {
    let mut evaluator = Evaluator::new();
    let mut args = std::env::args();
    if args.len() != 2 {
        let _ = run_repl();
        return;
    }
    let query = args.nth(1).unwrap();
    let result = evaluator.parse_and_eval(query.as_str());
    let _ = evaluator.write_val(result, &mut std::io::stdout(), true);
}
