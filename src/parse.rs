use crate::{
    ast::{AstNode, BinOp, Number, UnOp},
    MathParser, Rule,
};
use pest::{
    error::{Error, ErrorVariant},
    iterators::{Pair, Pairs},
    Parser,
};
use std::{ops::Rem, str::FromStr};

//

pub fn parse(line: &str) -> Result<AstNode, Error<Rule>> {
    let mut tokens = MathParser::parse(Rule::input, line)?;

    let token = tokens.next().unwrap();
    assert_eq!(tokens.next(), None);

    assert_eq!(token.as_rule(), Rule::input);
    let mut tokens = token.into_inner();

    log::debug!("{:?} = {}", tokens.clone(), tokens.clone().count());
    print_pair(tokens.clone(), 0);

    let token = tokens.next().unwrap();
    if let Some(leftover) = tokens.next() {
        if leftover.as_rule() != Rule::EOI {
            return Err(Error::new_from_span(
                ErrorVariant::CustomError {
                    message: "leftover tokens".to_string(),
                },
                leftover.as_span(),
            ));
        }
    }

    parse_expr(token)
}

fn parse_expr(token: Pair<Rule>) -> Result<AstNode, Error<Rule>> {
    assert_eq!(token.as_rule(), Rule::expr);

    let mut tokens = token.into_inner();
    let token = tokens.next().unwrap();
    assert_eq!(tokens.next(), None);

    parse_binary(token)
}

fn parse_binary(token: Pair<Rule>) -> Result<AstNode, Error<Rule>> {
    assert_eq!(token.as_rule(), Rule::binary);

    let mut tokens = token.into_inner();
    let mut lhs = parse_unary(tokens.next().unwrap())?;

    while let (Some(sign), Some(unary)) = (tokens.next(), tokens.next()) {
        let operator = match sign.as_rule() {
            Rule::add => BinOp::Add,
            Rule::sub => BinOp::Sub,
            other => unreachable!("{:?}", other),
        };

        let rhs = parse_unary(unary)?;

        lhs = AstNode::BinExpr {
            operands: Box::new((lhs, rhs)),
            operator,
        };
    }

    Ok(lhs)
}

fn parse_unary(token: Pair<Rule>) -> Result<AstNode, Error<Rule>> {
    assert_eq!(token.as_rule(), Rule::unary);

    let mut tokens = token.into_inner().rev();

    let term = parse_term(tokens.next().unwrap())?;

    let neg_count = tokens.fold(0, |acc, sign| match sign.as_rule() {
        Rule::pos => acc,
        Rule::neg => acc + 1,
        other => unreachable!("{:?}", other),
    });

    Ok(if neg_count.rem(2) == 1 {
        AstNode::UnExpr {
            operand: Box::new(term),
            operator: UnOp::Neg,
        }
    } else {
        term
    })
}

fn parse_term(token: Pair<Rule>) -> Result<AstNode, Error<Rule>> {
    assert_eq!(token.as_rule(), Rule::term);

    let mut tokens = token.into_inner();
    let mut lhs = parse_atom(tokens.next().unwrap())?;

    while let (Some(sign), Some(unary)) = (tokens.next(), tokens.next()) {
        let operator = match sign.as_rule() {
            Rule::mul | Rule::elided_mul => BinOp::Mul,
            Rule::div => BinOp::Div,
            other => unreachable!("{:?}", other),
        };

        let rhs = parse_atom(unary)?;

        lhs = AstNode::BinExpr {
            operands: Box::new((lhs, rhs)),
            operator,
        };
    }

    Ok(lhs)
}

fn parse_atom(token: Pair<Rule>) -> Result<AstNode, Error<Rule>> {
    assert_eq!(token.as_rule(), Rule::atom);

    let mut tokens = token.into_inner().rev();
    let mut rhs = parse_operand(tokens.next().unwrap())?;

    while let (Some(sign), Some(unary)) = (tokens.next(), tokens.next()) {
        let operator = match sign.as_rule() {
            Rule::pow => BinOp::Pow,
            other => unreachable!("{:?}", other),
        };

        let lhs = parse_operand(unary)?;

        rhs = AstNode::BinExpr {
            operands: Box::new((lhs, rhs)),
            operator,
        };
    }

    Ok(rhs)
}

fn parse_operand(token: Pair<Rule>) -> Result<AstNode, Error<Rule>> {
    assert_eq!(token.as_rule(), Rule::operand);

    let mut tokens = token.into_inner();
    let token = tokens.next().unwrap();
    assert_eq!(tokens.next(), None);

    match token.as_rule() {
        Rule::int => Ok(AstNode::Number(Number::Integer(parse_or_err(token)?))),
        Rule::float => Ok(AstNode::Number(Number::Decimal(parse_or_err(token)?))),
        Rule::var => Ok(AstNode::Number(Number::Var(token.as_str().to_string()))),
        Rule::expr => parse_expr(token),
        other => unreachable!("{:?}", other),
    }
}

fn parse_or_err<T>(token: Pair<Rule>) -> Result<T, Error<Rule>>
where
    T: FromStr,
    <T as FromStr>::Err: ToString,
{
    match token.as_str().parse() {
        Ok(n) => Ok(n),
        Err(err) => Err(Error::new_from_span(
            ErrorVariant::CustomError {
                message: err.to_string(),
            },
            token.as_span(),
        )),
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
