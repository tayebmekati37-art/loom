use pest::Parser;
use pest_derive::Parser;
use crate::ir::{Statement, Source};

#[derive(Parser)]
#[grammar = "../grammars/simple.pest"]
pub struct SimpleParser;

pub fn parse_program(input: &str) -> Result<Vec<Statement>, anyhow::Error> {
    let input = input.trim();
    let mut statements = Vec::new();
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut pairs = SimpleParser::parse(Rule::statement, line)?;
        let stmt_pair = pairs.next().unwrap();
        let inner = stmt_pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::add_stmt => {
                let (value, target) = parse_add_stmt(inner)?;
                statements.push(Statement::Add { target, value });
            }
            Rule::move_stmt => {
                let (source, target) = parse_move_stmt(inner)?;
                statements.push(Statement::Move { source, target });
            }
            _ => {}
        }
    }
    Ok(statements)
}

fn parse_add_stmt(pair: pest::iterators::Pair<Rule>) -> Result<(Source, String), anyhow::Error> {
    let mut source = None;
    let mut target = None;
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::number => {
                let s = inner.as_str().trim();
                source = Some(Source::Literal(s.parse::<i64>()?));
            }
            Rule::identifier => {
                if source.is_none() {
                    source = Some(Source::Variable(inner.as_str().to_string()));
                } else {
                    target = Some(inner.as_str().to_string());
                }
            }
            _ => {}
        }
    }
    if let (Some(src), Some(tgt)) = (source, target) {
        Ok((src, tgt))
    } else {
        anyhow::bail!("Invalid ADD statement")
    }
}

fn parse_move_stmt(pair: pest::iterators::Pair<Rule>) -> Result<(Source, String), anyhow::Error> {
    let mut source = None;
    let mut target = None;
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::number => {
                let s = inner.as_str().trim();
                source = Some(Source::Literal(s.parse::<i64>()?));
            }
            Rule::identifier => {
                if source.is_none() {
                    source = Some(Source::Variable(inner.as_str().to_string()));
                } else {
                    target = Some(inner.as_str().to_string());
                }
            }
            _ => {}
        }
    }
    if let (Some(src), Some(tgt)) = (source, target) {
        Ok((src, tgt))
    } else {
        anyhow::bail!("Invalid MOVE statement")
    }
}