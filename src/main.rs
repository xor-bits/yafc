use lazy_static::lazy_static;
use pest::{
    iterators::{Pair, Pairs},
    prec_climber::PrecClimber,
    Parser,
};
use rustyline::{error::ReadlineError, Editor};
use std::{
    fmt::{self, Display},
    iter::Product,
    ops::{Add, BitXor, Div, Mul, Sub},
    process::exit,
};

//

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct MathParser;

#[cfg(target_pointer_width = "64")]
type FSize = f64;
#[cfg(not(target_pointer_width = "64"))]
type FSize = f32;

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use pest::prec_climber::{Assoc, Operator};
        PrecClimber::new(vec![
            Operator::new(Rule::add, Assoc::Left) | Operator::new(Rule::sub, Assoc::Left),
            Operator::new(Rule::mul, Assoc::Left)
                | Operator::new(Rule::elided_mul, Assoc::Left)
                | Operator::new(Rule::div, Assoc::Left),
            Operator::new(Rule::pow, Assoc::Right),
        ])
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

impl Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Operator::Add => '+',
                Operator::Sub => '-',
                Operator::Mul => '*',
                Operator::Div => '/',
                Operator::Pow => '^',
            }
        )
    }
}

/* impl Operator {
    fn from_rule(rule: Rule) -> Option<Self> {
        match rule {
            Rule::add => Some(Self::Add),
            Rule::sub => Some(Self::Sub),
            Rule::mul => Some(Self::Mul),
            Rule::div => Some(Self::Div),
            Rule::pow => Some(Self::Pow),
            _ => None,
        }
    }
} */

impl From<Rule> for Option<Operator> {
    fn from(_: Rule) -> Self {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
enum AstNode {
    Binary {
        operator: Operator,
        operands: Box<(AstNode, AstNode)>,
    },
    /* Unary {
        operator: Operator,
        operand: Box<AstNode>,
    }, */
    Number(Number),
}

impl Display for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstNode::Binary { operator, operands } => {
                write!(f, "({}{operator}{})", operands.0, operands.1)
            }
            // AstNode::Unary { operator, operand } => write!(f, "({operator}{operand})"),
            AstNode::Number(Number::Integer(v)) => write!(f, "{v}"),
            AstNode::Number(Number::Decimal(v)) => write!(f, "{v}"),
            AstNode::Number(Number::Constant(v)) => write!(f, "{v}"),
        }
    }
}

impl AstNode {
    fn eval(self) -> AstNode {
        match self {
            AstNode::Binary { operator, operands } => {
                enum T {
                    A((isize, isize)),
                    B((FSize, FSize)),
                    C((AstNode, AstNode)),
                }
                let sides = match (operands.0.eval(), operands.1.eval()) {
                    (
                        AstNode::Number(Number::Integer(lhs)),
                        AstNode::Number(Number::Integer(rhs)),
                    ) => T::A((lhs, rhs)),
                    (
                        AstNode::Number(Number::Decimal(lhs)),
                        AstNode::Number(Number::Integer(rhs)),
                    ) => T::B((lhs, rhs as f64)),
                    (
                        AstNode::Number(Number::Integer(lhs)),
                        AstNode::Number(Number::Decimal(rhs)),
                    ) => T::B((lhs as f64, rhs)),
                    (
                        AstNode::Number(Number::Decimal(lhs)),
                        AstNode::Number(Number::Decimal(rhs)),
                    ) => T::B((lhs, rhs as f64)),
                    operands => T::C(operands),
                };

                match (sides, operator) {
                    (T::A((lhs, rhs)), Operator::Add) => {
                        AstNode::Number(Number::Integer(lhs + rhs))
                    }
                    (T::B((lhs, rhs)), Operator::Add) => {
                        AstNode::Number(Number::Decimal(lhs + rhs))
                    }

                    (T::A((lhs, rhs)), Operator::Sub) => {
                        AstNode::Number(Number::Integer(lhs - rhs))
                    }
                    (T::B((lhs, rhs)), Operator::Sub) => {
                        AstNode::Number(Number::Decimal(lhs - rhs))
                    }

                    (T::A((lhs, rhs)), Operator::Mul) => {
                        AstNode::Number(Number::Integer(lhs * rhs))
                    }
                    (T::B((lhs, rhs)), Operator::Mul) => {
                        AstNode::Number(Number::Decimal(lhs * rhs))
                    }

                    (T::A((lhs, rhs)), Operator::Div) => {
                        AstNode::Number(Number::Integer(lhs / rhs))
                    }
                    (T::B((lhs, rhs)), Operator::Div) => {
                        AstNode::Number(Number::Decimal(lhs / rhs))
                    }

                    (T::A((lhs, rhs)), Operator::Pow) => {
                        if let Ok(rhs) = rhs.try_into() {
                            AstNode::Number(Number::Integer(lhs.pow(rhs)))
                        } else {
                            AstNode::Number(Number::Decimal((lhs as FSize).powi(rhs as _)))
                        }
                    }
                    (T::B((lhs, rhs)), Operator::Pow) => {
                        AstNode::Number(Number::Decimal(lhs.powf(rhs)))
                    }

                    (T::C(operands), _) => AstNode::Binary {
                        operator,
                        operands: Box::new(operands),
                    },
                }
            }
            other => other,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Number {
    Integer(isize),
    Decimal(FSize),
    Constant(String),
}

impl Add for AstNode {
    type Output = AstNode;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Binary {
            operator: Operator::Add,
            operands: Box::new((self, rhs)),
        }
    }
}

impl Sub for AstNode {
    type Output = AstNode;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Binary {
            operator: Operator::Sub,
            operands: Box::new((self, rhs)),
        }
    }
}

impl Mul for AstNode {
    type Output = AstNode;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::Binary {
            operator: Operator::Mul,
            operands: Box::new((self, rhs)),
        }
    }
}

impl Div for AstNode {
    type Output = AstNode;

    fn div(self, rhs: Self) -> Self::Output {
        Self::Binary {
            operator: Operator::Div,
            operands: Box::new((self, rhs)),
        }
    }
}

impl BitXor for AstNode {
    type Output = AstNode;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::Binary {
            operator: Operator::Pow,
            operands: Box::new((self, rhs)),
        }
    }
}

//

fn main() {
    env_logger::init();

    let mut rl = Editor::<()>::new();
    let _ = rl.load_history("history.txt");
    for i in 0.. {
        match rl.readline("in: ") {
            Ok(line) => {
                rl.add_history_entry(&line);
                println!("out[{i}]: {}", run(&line));
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

fn run(line: &str) -> AstNode {
    let pair = MathParser::parse(Rule::expression, line).unwrap_or_else(|err| {
        eprintln!("{err}");
        exit(1);
    });
    log::debug!("{:?} = {}", pair.clone(), pair.clone().count());
    print_pair(pair.clone(), 0);

    let eval = PREC_CLIMBER.climb(pair, parse_primary, parse_infix);
    log::debug!("{:?}", eval);
    log::debug!("{}", eval);
    eval.eval()
}

fn parse_primary(pair: Pair<Rule>) -> AstNode {
    log::debug!("{} = {:?}", pair.as_str(), pair.as_rule());
    match pair.as_rule() {
        Rule::integer => AstNode::Number(Number::Integer(pair.as_str().parse().unwrap())),
        Rule::decimal => AstNode::Number(Number::Decimal(pair.as_str().parse().unwrap())),
        Rule::constant => AstNode::Number(Number::Constant(pair.as_str().to_string())),
        /* Rule::unary => {
            let mut operands = pair.into_inner().rev();
            let first = parse_primary(operands.next().unwrap());
            operands
                .into_iter()
                .fold(first, |operand, rule| AstNode::Unary {
                    operand: Box::new(operand),
                    operator: Operator::from_rule(rule.as_rule())
                        .unwrap_or_else(|| unreachable!("rule was: {:?}", rule.as_rule())),
                })
        } */
        Rule::expression => PREC_CLIMBER.climb(pair.into_inner(), parse_primary, parse_infix),
        Rule::term => {
            let mut operands = pair.into_inner().map(parse_primary);
            let first = operands.next().unwrap();
            operands.into_iter().fold(first, |lhs, rhs| lhs * rhs)
        }
        other => unreachable!("{:?}", other),
    }
}

fn parse_infix(lhs: AstNode, pair: Pair<Rule>, rhs: AstNode) -> AstNode {
    match pair.as_rule() {
        Rule::add => lhs + rhs,
        Rule::sub => lhs - rhs,
        Rule::mul => lhs * rhs,
        Rule::elided_mul => lhs * rhs,
        Rule::div => lhs / rhs,
        Rule::pow => lhs ^ rhs,
        other => unreachable!("{:?}", other),
    }
}

fn print_pair(pairs: Pairs<Rule>, nest: usize) {
    if !log::log_enabled!(log::Level::Debug) {
        return;
    }

    for pair in pairs {
        let mut buf = String::new();
        for _ in 0..nest {
            buf += "| ";
        }
        log::debug!("{buf}{:?}:{:?}:", pair.as_str(), pair.as_rule());
        print_pair(pair.into_inner(), nest + 1);
    }
}
