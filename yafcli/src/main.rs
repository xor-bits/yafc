use clap::Parser;
use cli::CliArgs;
use rustyline::{error::ReadlineError, Editor};
use std::io;
use yafc::{
    ast::{PrettyStyle, YafcExpr},
    simplifier::Simplifier,
};

//

pub mod cli;

//

fn main() {
    let cli: CliArgs = CliArgs::parse();

    tracing_subscriber::fmt::init();
    // env_logger::builder()
    //     .parse_default_env()
    //     .filter(Some("rustyline"), LevelFilter::Error)
    //     .init();

    if let Some(line) = &cli.direct {
        run_line(line, &cli, 0);
    } else if atty::isnt(atty::Stream::Stdin) {
        // if STDIN is piped
        io::stdin()
            .lines()
            .enumerate()
            .for_each(|(i, line)| match line {
                Ok(line) => run_line(&line, &cli, i),
                Err(err) => eprintln!("{err}"),
            });
    } else {
        let mut rl = Editor::<()>::new().unwrap();
        let _ = rl.load_history("history.txt");
        for i in 0.. {
            match rl.readline("in: ") {
                Ok(line) => {
                    rl.add_history_entry(&line);
                    run_line(&line, &cli, i);
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
        let _ = rl.save_history("history.txt");
    }
}

fn run_line(line: &str, cli: &CliArgs, i: usize) {
    match YafcExpr::parse_infix(line) {
        Ok(ast) => {
            let simplified = Simplifier::run(&ast);

            let style = if cli.latex_out {
                PrettyStyle::LaTeX
            } else {
                <_>::default()
            };
            let ast = ast.pretty_opt(style);
            let simplified = simplified.pretty_opt(style);

            if cli.debug {
                println!("dbg: {ast:?} = {simplified:?}");
                println!("dbg: => {ast} = {simplified}");
            }
            if cli.verbose {
                println!("out[{i}]: {simplified}\n");
            } else {
                println!("{simplified:#}");
            }
        }
        Err(err) => eprintln!("{err}"),
    };
}
