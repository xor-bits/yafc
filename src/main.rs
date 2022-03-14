#![feature(box_patterns)]

//

use crate::parse::parse;
use rustyline::{error::ReadlineError, Editor};

//

mod ast;
mod parse;
mod simplify;

//

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct MathParser;

//

fn main() {
    env_logger::init();

    let mut rl = Editor::<()>::new();
    let _ = rl.load_history("history.txt");
    for i in 0.. {
        match rl.readline("in: ") {
            Ok(line) => {
                rl.add_history_entry(&line);

                match parse(&line)
                    .map_err(|err| err.to_string())
                    .and_then(|ast| ast.eval().map_err(str::to_string))
                {
                    Ok(ast) => println!("out[{i}]: {ast}"),
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
