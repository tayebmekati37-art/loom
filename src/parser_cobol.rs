use pest::Parser;
use pest_derive::Parser;
use crate::ir::{Statement, Source, Condition};

#[derive(Parser)]
#[grammar = "../grammars/cobol.pest"]
pub struct CobolParser;

pub fn parse_program(input: &str) -> Result<Vec<Statement>, anyhow::Error> {
    let input = input.trim();
    let mut statements = Vec::new();
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut pairs = CobolParser::parse(Rule::statement, line)?;
        let stmt_pair = pairs.next().unwrap();
        match stmt_pair.as_rule() {
            Rule::add_stmt => {
                let (value, target) = parse_add_stmt(stmt_pair)?;
                statements.push(Statement::Add { target, value });
            }
            Rule::move_stmt => {
                let (source, target) = parse_move_stmt(stmt_pair)?;
                statements.push(Statement::Move { source, target });
            }
            Rule::if_stmt => {
                let (cond, then_branch, else_branch) = parse_if_stmt(stmt_pair)?;
                statements.push(Statement::If { condition: cond, then_branch, else_branch });
            }
            Rule::perform_stmt => {
                let name = parse_perform_stmt(stmt_pair)?;
                statements.push(Statement::Perform { name });
            }
            Rule::while_stmt => {
                let (cond, body) = parse_while_stmt(stmt_pair)?;
                statements.push(Statement::While { condition: cond, body });
            }
            _ => {}
        }
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

fn parse_if_stmt(pair: pest::iterators::Pair<Rule>) -> Result<(Condition, Vec<Statement>, Option<Vec<Statement>>), anyhow::Error> {
    let mut inner = pair.into_inner();
    let cond_pair = inner.next().unwrap();
    let condition = parse_condition(cond_pair)?;
    let then_branch = parse_statement_block(inner.next().unwrap())?;
    let else_branch = inner.next().map(|p| parse_statement_block(p)).transpose()?;
    Ok((condition, then_branch, else_branch))
}

fn parse_perform_stmt(pair: pest::iterators::Pair<Rule>) -> Result<String, anyhow::Error> {
    let mut inner = pair.into_inner();
    let name_pair = inner.next().unwrap();
    Ok(name_pair.as_str().to_string())
}

fn parse_while_stmt(pair: pest::iterators::Pair<Rule>) -> Result<(Condition, Vec<Statement>), anyhow::Error> {
    let mut inner = pair.into_inner();
    let cond_pair = inner.next().unwrap();
    let condition = parse_condition(cond_pair)?;
    let body = parse_statement_block(inner.next().unwrap())?;
    Ok((condition, body))
}

fn parse_condition(pair: pest::iterators::Pair<Rule>) -> Result<Condition, anyhow::Error> {
    let mut parts = pair.into_inner();
    let left = parts.next().unwrap().as_str().to_string();
    let op = parts.next().unwrap().as_str().to_string();
    let right = parts.next().unwrap().as_str().parse::<i64>()?;
    Ok(Condition { left, operator: op, right })
}

fn parse_statement_block(pair: pest::iterators::Pair<Rule>) -> Result<Vec<Statement>, anyhow::Error> {
    let mut statements = Vec::new();
    for stmt_pair in pair.into_inner() {
        match stmt_pair.as_rule() {
            Rule::add_stmt => {
                let (value, target) = parse_add_stmt(stmt_pair)?;
                statements.push(Statement::Add { target, value });
            }
            Rule::move_stmt => {
                let (source, target) = parse_move_stmt(stmt_pair)?;
                statements.push(Statement::Move { source, target });
            }
            Rule::if_stmt => {
                let (cond, then_branch, else_branch) = parse_if_stmt(stmt_pair)?;
                statements.push(Statement::If { condition: cond, then_branch, else_branch });
            }
            Rule::perform_stmt => {
                let name = parse_perform_stmt(stmt_pair)?;
                statements.push(Statement::Perform { name });
            }
            Rule::while_stmt => {
                let (cond, body) = parse_while_stmt(stmt_pair)?;
                statements.push(Statement::While { condition: cond, body });
            }
            _ => {}
        }
    }
    Ok(statements)
}