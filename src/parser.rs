use pest::Parser;
use pest_derive::Parser;
use crate::ir::{Statement, Source};

#[derive(Parser)]
#[grammar = "../grammars/simple.pest"]
pub struct SimpleParser;

pub fn parse_program(input: &str) -> Result<Vec<Statement>, anyhow::Error> {
    let input = input.trim();
    println!("Parsing input: {:?}", input);
    let mut pairs = SimpleParser::parse(Rule::program, input)?;
    let program = pairs.next().ok_or_else(|| anyhow::anyhow!("No program found"))?;
    let inner_pairs: Vec<_> = program.into_inner().collect(); // collect first to inspect
    println!("Number of inner pairs: {}", inner_pairs.len());
    for (i, pair) in inner_pairs.iter().enumerate() {
        println!("Inner pair {}: rule {:?}, text: {:?}", i, pair.as_rule(), pair.as_str());
    }
    // Now process
    let mut statements = Vec::new();
    for pair in inner_pairs {
        // ... rest as before
    }
    Ok(statements)
}

fn parse_add_stmt(pair: pest::iterators::Pair<Rule>) -> Result<(i64, String), anyhow::Error> {
    let mut number = None;
    let mut var = None;
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::number => number = Some(inner.as_str().parse::<i64>()?),
            Rule::identifier => var = Some(inner.as_str().to_string()),
            _ => {}
        }
    }
    if let (Some(num), Some(var_name)) = (number, var) {
        Ok((num, var_name))
    } else {
        anyhow::bail!("Invalid ADD statement")
    }
}

fn parse_move_stmt(pair: pest::iterators::Pair<Rule>) -> Result<(Source, String), anyhow::Error> {
    let mut source = None;
    let mut target = None;
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::number => source = Some(Source::Literal(inner.as_str().parse::<i64>()?)),
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