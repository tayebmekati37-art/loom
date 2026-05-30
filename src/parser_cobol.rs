use anyhow::Result;
use crate::ir::*;

pub fn parse_program(input: &str) -> Result<Vec<Statement>> {
    let mut statements = Vec::new();

    for raw_line in input.lines() {

        let line = raw_line.trim();

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Skip comments
        if line.starts_with("*") {
            continue;
        }

        // Skip COBOL divisions / metadata
        if line.starts_with("IDENTIFICATION DIVISION")
            || line.starts_with("ENVIRONMENT DIVISION")
            || line.starts_with("DATA DIVISION")
            || line.starts_with("PROCEDURE DIVISION")
            || line.starts_with("WORKING-STORAGE SECTION")
            || line.starts_with("PROGRAM-ID")
            || line.starts_with("AUTHOR")
            || line.starts_with("DATE-WRITTEN")
        {
            continue;
        }

        let stmt = parse_statement(line)?;

        statements.push(stmt);
    }

    Ok(statements)
}

fn parse_statement(line: &str) -> Result<Statement> {
    // Ignore PIC declarations
    if line.contains("PIC") {
        return Ok(Statement::NoOp);
    }

    let clean = line.replace(".", "");
    let parts: Vec<&str> = clean.split_whitespace().collect();

    if parts.is_empty() {
        anyhow::bail!("Empty statement");
    }

    match parts[0].to_lowercase().as_str() {

        "display" => {

            if parts.len() < 2 {
                anyhow::bail!("Invalid DISPLAY");
            }

            Ok(
                Statement::Display {
                    value: Literal::String(
                        parts[1..].join(" ")
                    )
                }
            )
        }

        "move" => {

            if parts.len() < 4 {
                anyhow::bail!("Invalid MOVE");
            }

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

            if parts.len() < 4 {
                anyhow::bail!("Invalid ADD");
            }

            Ok(
                Statement::Add {
                    value: parts[1].parse::<i64>().unwrap_or(0),
                    target: parts[3].to_string(),
                }
            )
        }

        "perform" => {

            if parts.len() >= 3 &&
               parts[1].to_lowercase() == "until" {

                let cond = Condition {
                    left: parts[2].to_string(),
                    operator: ">".to_string(),
                    right: 0,
                };

                Ok(
                    Statement::PerformUntil {
                        condition: cond,
                        body: Vec::new(),
                    }
                )

            } else if parts.len() >= 2 {

                Ok(
                    Statement::Perform {
                        name: parts[1].to_string(),
                    }
                )

            } else {

                anyhow::bail!("Invalid PERFORM");
            }
        }

        "call" => {

            if parts.len() < 2 {
                anyhow::bail!("Invalid CALL");
            }

            Ok(
                Statement::Call {
                    program: parts[1]
                        .replace('"', ""),
                    using_args: Vec::new(),
                }
            )
        }

        "if" => {

            let cond = Condition {
                left: "X".to_string(),
                operator: "=".to_string(),
                right: 1,
            };

            Ok(
                Statement::If {
                    condition: cond,
                    then_branch: Vec::new(),
                    else_branch: Some(Vec::new()),
                }
            )
        }
        "stop" => {
    if parts.len() >= 2 && parts[1].to_lowercase() == "run" {
        Ok(Statement::NoOp)
    } else {
        anyhow::bail!("Invalid STOP statement");
    }
}
        _ => {
            anyhow::bail!(
                "Unknown statement: {}",
                line
            )
        }
    }
}