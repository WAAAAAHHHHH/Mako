mod token;
mod lexer;
mod ast;
mod parser;
mod runtime;
mod studio;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "run" => {
            if args.len() < 3 {
                eprintln!("Usage: mako run <file.mako>");
                std::process::exit(1);
            }
            run_file(&args[2]);
        }
        "studio" => {
            if let Err(e) = studio::start_studio() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        "help" | "--help" | "-h" => {
            print_help();
        }
        "version" | "--version" | "-v" => {
            println!("Mako v0.1.0");
        }
        _ if command.ends_with(".mako") => {
            run_file(command);
        }
        _ => {
            eprintln!("Unknown command: '{}'. Run `mako help` for usage.", command);
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!();
    println!("  🌊  Mako Programming Language v0.1.0");
    println!();
    println!("  USAGE:");
    println!("    mako run <file.mako>    Run a Mako script");
    println!("    mako studio             Open Mako Studio (browser IDE)");
    println!("    mako version            Print version");
    println!("    mako help               Show this help message");
    println!();
    println!("  EXAMPLES:");
    println!("    mako run examples/hello.mako");
    println!("    mako studio");
    println!();
}

fn run_file(filename: &str) {
    let source = fs::read_to_string(filename).unwrap_or_else(|_| {
        eprintln!("Error: Could not read file '{}'", filename);
        std::process::exit(1);
    });

    let mut lexer = lexer::Lexer::new(&source);
    match lexer.tokenize() {
        Ok(tokens) => {
            let mut parser = parser::Parser::new(tokens);
            match parser.parse() {
                Ok(program) => {
                    let mut runtime = runtime::Runtime::new();
                    if let Err(e) = runtime.execute(&program) {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Parse Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Lexer Error: {}", e);
            std::process::exit(1);
        }
    }
}
