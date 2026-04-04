use pest::Parser;
use pest_derive::Parser;
use crate::ir::{Statement, Source, Literal, Condition};

#[derive(Parser)]
#[grammar = "../grammars/cobol.pest"]
pub struct CobolParser;

pub fn parse_program(input: &str) -> Result<Vec<Statement>, anyhow::Error> {
    let input = input.trim_start_matches('\u{feff}').to_lowercase();
    let input = input.trim();
    let mut statements = Vec::new();
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut pairs = CobolParser::parse(Rule::statement, line)?;
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
            Rule::if_stmt => {
                let (cond, then_branch, else_branch) = parse_if_stmt(inner)?;
                statements.push(Statement::If { condition: cond, then_branch, else_branch });
            }
            Rule::perform_stmt => {
                let name = parse_perform_stmt(inner)?;
                statements.push(Statement::Perform { name });
            }
            Rule::while_stmt => {
                let (cond, body) = parse_while_stmt(inner)?;
                statements.push(Statement::While { condition: cond, body });
            }
            Rule::display_stmt => {
                let lit = parse_display_stmt(inner)?;
                statements.push(Statement::Display { value: lit });
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
        anyhow::bail!("Invalid ADD")
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
        anyhow::bail!("Invalid MOVE")
    }
}

// The rest of the functions (parse_if_stmt, parse_perform_stmt, etc.) remain unchanged.
// Copy them from your existing parser_cobol.rs – they are identical to the previous version.
// For brevity, I'm not repeating them here, but they must be kept exactly as before.