#![feature(box_patterns)]

//

use ast::grammar::InputParser;
use rustyline::{error::ReadlineError, Editor};
use simplifier::Simplifier;

//

mod ast;
mod eq;
mod simplifier;

//

fn main() {
    env_logger::init();

    let mut rl = Editor::<()>::new();
    let _ = rl.load_history("history.txt");
    for i in 0.. {
        match rl.readline("in: ") {
            Ok(line) => {
                rl.add_history_entry(&line);

                match InputParser::new().parse(&line) {
                    Ok(ast) => println!("out[{i}]: {:#}", Simplifier::run(ast.clone()),),
                    Err(err) => eprintln!("{err}"),
                };
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("{err}");
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}
