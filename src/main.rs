use std::io::{stdin, stdout};
use std::{fs::read_to_string, io::Write};

use environment::Environment;
mod environment;
use error::Error;
mod callable;
mod error;
mod expressions;
mod interpreter;
mod parser;
mod resolver;
mod scanner;
mod statements;
mod tokens;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = match args.get(1) {
        Some(path) => path.to_owned(),
        None => String::from("REPL"),
    };

    let run = |source: String| {
        let err = Error::new(&path, Some(source.to_owned()));

        let mut scanner = scanner::Scanner::new(&err);
        let tokens = match scanner.scan_tokens(source) {
            Ok(tokens) => tokens,
            Err(_) => return,
        };

        let statements = match parser::Parser::new(&err).parse(tokens) {
            Ok(stmts) => stmts,
            Err(_) => return,
        };

        _ = interpreter::Interpreter::new(&err, Environment::new(None), false)
            .interpret(statements);
    };

    let run_repl = || {
        let err = Error::new(&path, None);

        let mut scanner = scanner::Scanner::new(&err);
        let mut parser = parser::Parser::new(&err);
        let mut interpreter = interpreter::Interpreter::new(&err, Environment::new(None), true);

        loop {
            print!("> ");
            stdout().flush().unwrap();
            let mut line = String::new();
            if let Ok(_) = stdin().read_line(&mut line) {
                let tokens = match scanner.scan_tokens(line) {
                    Ok(tokens) => tokens,
                    Err(_) => continue,
                };

                let statements = match parser.parse(tokens) {
                    Ok(stmts) => stmts,
                    Err(_) => continue,
                };

                _ = interpreter.interpret(statements);
            } else {
                break;
            }
        }
    };

    match args.len() {
        1 => run_repl(),
        2 => {
            run(read_to_string(&path).unwrap_or_else(|_| panic!("Could not read file: {}", &path)));
        }
        _ => {
            println!("Usage: jlox [script]");
            std::process::exit(1);
        }
    }
}
