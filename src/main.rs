#![feature(box_patterns)]
#![feature(drain_filter)]

//

use ast::grammar::InputParser;
use log::LevelFilter;
use rustyline::{error::ReadlineError, Editor};
use simplifier::Simplifier;
use std::fmt::{Debug, Display};

//

mod ast;
mod eq;
mod simplifier;

//

fn main() {
    env_logger::builder()
        .parse_default_env()
        .filter(Some("rustyline"), LevelFilter::Error)
        .init();

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

//

pub fn assert_eq_display<T: PartialEq + Debug + Display>(lhs: T, rhs: T) {
    assert_eq!(lhs, rhs, "\n left: {lhs}\nright: {rhs}")
}
