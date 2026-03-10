use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "-h" | "--help" => {
            print_usage();
        }
        "-v" | "--version" => {
            println!("WB-Rust {}", env!("CARGO_PKG_VERSION"));
        }
        "repl" | "--repl" => {
            if let Err(err) = run_repl() {
                eprintln!("Error: {}", err.message);
                std::process::exit(1);
            }
        }
        path => {
            if let Err(err) = run_file(path) {
                eprintln!("Error: {}", err.message);
                std::process::exit(1);
            }
        }
    }
}

fn run_file(path: &str) -> Result<(), wb_diagnostics::Diagnostic> {
    let mut session = wb_core::Session::new();
    session.exec_file(std::path::Path::new(path))
}

fn run_repl() -> Result<(), wb_diagnostics::Diagnostic> {
    use std::io::{self, Write};

    let mut session = wb_core::Session::new();
    println!("WB-Rust REPL. Ketik 'keluar' untuk berhenti.");
    loop {
        print!("> ");
        io::stdout().flush().ok();

        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            return Err(wb_diagnostics::Diagnostic::new("Gagal membaca input"));
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "keluar" || trimmed == "exit" {
            break;
        }

        if let Err(err) = session.exec(&line) {
            eprintln!("Error: {}", err.message);
        }
    }
    Ok(())
}

fn print_usage() {
    println!("WB-Rust Interpreter");
    println!("Usage:");
    println!("  wb-cli <file.wb>");
    println!("  wb-cli --repl");
    println!("  wb-cli --help");
    println!("  wb-cli --version");
}
