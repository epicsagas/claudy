use std::env;
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    let argv0_raw = args.first().map(|s| s.as_str()).unwrap_or("claudy");
    let argv0 = Path::new(argv0_raw)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(argv0_raw);

    let code = match claudy::run(argv0, &args[1..]) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("Error: {}", err);
            1
        }
    };
    process::exit(code);
}
