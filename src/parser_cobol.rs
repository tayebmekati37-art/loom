use pest::Parser;
use pest_derive::Parser;
use crate::ir::{Statement, Source, Literal, Condition};

#[derive(Parser)]
#[grammar = "../grammars/cobol.pest"]
pub struct CobolParser;

pub fn parse_program(input: &str) -> Result<Vec<Statement>, anyhow::Error> {
    // Remove UTF-8 BOM if present and convert to lowercase
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

fn parse_add_stmt(pair: pest::iterators::Pair<Rule>) -> Result<(i64, String), anyhow::Error> {
    let mut number = None;
    let mut var = None;
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::number => {
                let s = inner.as_str().trim();
                number = Some(s.parse::<i64>()?);
            }
            Rule::identifier => var = Some(inner.as_str().to_string()),
            _ => {}
        }
    }
    if let (Some(num), Some(var_name)) = (number, var) {
        Ok((num, var_name))
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

fn parse_display_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Literal, anyhow::Error> {
    let mut inner = pair.into_inner();
    let lit_pair = inner.next().unwrap();
    match lit_pair.as_rule() {
        Rule::number => {
            let s = lit_pair.as_str().trim();
            Ok(Literal::Int(s.parse::<i64>()?))
        }
        Rule::string_literal => {
            let s = lit_pair.as_str();
            let inner_str = &s[1..s.len()-1];
            Ok(Literal::String(inner_str.to_string()))
        }
        _ => anyhow::bail!("Invalid DISPLAY argument"),
    }
}

fn parse_condition(pair: pest::iterators::Pair<Rule>) -> Result<Condition, anyhow::Error> {
    let mut parts = pair.into_inner();
    let left = parts.next().unwrap().as_str().to_string();
    let op = parts.next().unwrap().as_str().to_string();
    let right_part = parts.next().unwrap();
    let right = match right_part.as_rule() {
        Rule::number => right_part.as_str().parse::<i64>()?,
        _ => anyhow::bail!("Invalid condition right part"),
    };
    Ok(Condition { left, operator: op, right })
}

fn parse_statement_block(pair: pest::iterators::Pair<Rule>) -> Result<Vec<Statement>, anyhow::Error> {
    let mut statements = Vec::new();
    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::statement {
            let stmt = parse_statement(inner_pair)?;
            statements.push(stmt);
        }
    }
    Ok(statements)
}

fn parse_statement(pair: pest::iterators::Pair<Rule>) -> Result<Statement, anyhow::Error> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::add_stmt => {
            let (value, target) = parse_add_stmt(inner)?;
            Ok(Statement::Add { target, value })
        }
        Rule::move_stmt => {
            let (source, target) = parse_move_stmt(inner)?;
            Ok(Statement::Move { source, target })
        }
        Rule::if_stmt => {
            let (cond, then_branch, else_branch) = parse_if_stmt(inner)?;
            Ok(Statement::If { condition: cond, then_branch, else_branch })
        }
        Rule::perform_stmt => {
            let name = parse_perform_stmt(inner)?;
            Ok(Statement::Perform { name })
        }
        Rule::while_stmt => {
            let (cond, body) = parse_while_stmt(inner)?;
            Ok(Statement::While { condition: cond, body })
        }
        Rule::display_stmt => {
            let lit = parse_display_stmt(inner)?;
            Ok(Statement::Display { value: lit })
        }
        _ => anyhow::bail!("Unsupported statement"),
    }
}