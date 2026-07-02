use crate::ir::*;
use anyhow::Result;

pub fn parse_program(input: &str) -> Result<Vec<Statement>> {

    let mut statements = Vec::new();

    let lines: Vec<&str> = input.lines().collect();

    let mut i = 0;

    while i < lines.len() {

        let raw_line = lines[i];

        let line = raw_line
            .replace('\u{feff}', "")
            .trim()
            .to_string();

        let line = line.as_str();

        if line.is_empty() {
            i += 1;
            continue;
        }

        if line.starts_with('*') {
            i += 1;
            continue;
        }

        let upper = line.to_uppercase();

        if upper.starts_with("IDENTIFICATION DIVISION")
            || upper.starts_with("ENVIRONMENT DIVISION")
            || upper.starts_with("DATA DIVISION")
            || upper.starts_with("PROCEDURE DIVISION")
            || upper.starts_with("WORKING-STORAGE SECTION")
            || upper.starts_with("FILE SECTION")
            || upper.starts_with("LINKAGE SECTION")
        {
            i += 1;
            continue;
        }

        if upper.starts_with("PERFORM VARYING") {

            let parts: Vec<&str> = line.split_whitespace().collect();

            let variable = parts[2].to_string();

            let from_value =
                parts[4].parse::<i64>().unwrap_or(0);

            let by_value =
                parts[6].parse::<i64>().unwrap_or(1);

            let until_text =
                parts[8..].join(" ");

            let mut body = Vec::new();

            i += 1;

            while i < lines.len() {

                let inner = lines[i].trim();

                if inner.eq_ignore_ascii_case("END-PERFORM")
                    || inner.eq_ignore_ascii_case("END-PERFORM.")
                {
                    break;
                }

                body.push(parse_statement(inner)?);

                i += 1;
            }

            statements.push(
                Statement::PerformVarying {
                    variable,

                    from: Expression::Literal(
                        Literal::Int(from_value)
                    ),

                    by: Expression::Literal(
                        Literal::Int(by_value)
                    ),

                    until: parse_condition_expression(&until_text),

                    body,
                }
            );

            i += 1;
            continue;
        }

        if upper == "PERFORM" {

            let mut body = Vec::new();

            i += 1;

            while i < lines.len() {

                let inner = lines[i].trim();

                if inner.eq_ignore_ascii_case("END-PERFORM")
                    || inner.eq_ignore_ascii_case("END-PERFORM.")
                {
                    break;
                }

                body.push(parse_statement(inner)?);

                i += 1;
            }

            statements.push(
                Statement::Perform {
                    name: None,
                    body,
                }
            );

            i += 1;
            continue;
        }

        statements.push(parse_statement(line)?);

        i += 1;
    }

    Ok(statements)
}

fn parse_statement(line: &str) -> Result<Statement> {

    let clean = line.replace('.', "");

    let parts: Vec<&str> =
        clean.split_whitespace().collect();

    if parts.is_empty() {
        return Ok(Statement::NoOp);
    }

    match parts[0].to_lowercase().as_str() {

        "display" => {
            Ok(
                Statement::Display {
                    value: Literal::String(
                        parts[1..].join(" ")
                    ),
                }
            )
        }

        "move" => {
            Ok(
                Statement::Move {
                    source: Source::Variable(
                        parts[1].to_string()
                    ),
                    target: parts[3].to_string(),
                }
            )
        }

        "add" => {
            Ok(
                Statement::Add {
                    value: parts[1]
                        .parse::<i64>()
                        .unwrap_or(0),

                    target: parts[3].to_string(),
                }
            )
        }

        "compute" => {

            let target = parts[1].to_string();

            let expr =
                parse_expression(&parts[3..]);

            Ok(
                Statement::Compute {
                    target,
                    expr,
                }
            )
        }

        "if" => {

            let cond = Condition {
                left: parts[1].to_string(),
                operator: parts[2].to_string(),
                right: parts[3].to_string(),
            };

            Ok(
                Statement::If {
                    condition: cond,
                    then_branch: Vec::new(),
                    else_branch: None,
                }
            )
        }

        "call" => {
            Ok(
                Statement::Call {
                    program: parts[1]
                        .replace('"', ""),
                    using_args: Vec::new(),
                }
            )
        }

        "perform" => {
            Ok(
                Statement::Perform {
                    name: Some(
                        parts[1].to_string()
                    ),
                    body: Vec::new(),
                }
            )
        }

        "stop" => Ok(Statement::NoOp),

        _ => Ok(Statement::NoOp),
    }
}

fn parse_expression(tokens: &[&str]) -> Expression {

    if tokens.len() == 1 {

        let token = tokens[0];

        if let Ok(num) = token.parse::<i64>() {
            return Expression::Literal(
                Literal::Int(num)
            );
        }

        return Expression::Variable(
            token.to_string()
        );
    }

    if tokens.len() >= 3 {

        let left =
            parse_expression(&tokens[0..1]);

        let operator =
            tokens[1].to_string();

        let right =
            parse_expression(&tokens[2..]);

        return Expression::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        };
    }

    Expression::Literal(
        Literal::Int(0)
    )
}

fn parse_condition_expression(text: &str) -> Condition {

    let parts: Vec<&str> =
        text.split_whitespace().collect();

    Condition {
        left: parts.get(0)
            .unwrap_or(&"")
            .to_string(),

        operator: parts.get(1)
            .unwrap_or(&"=")
            .to_string(),

        right: parts.get(2)
            .unwrap_or(&"0")
            .to_string(),
    }
}


