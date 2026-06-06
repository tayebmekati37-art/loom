use crate::ir::*;
use anyhow::Result;

pub fn parse_program(input: &str) -> Result<Vec<Statement>> {
    let mut statements = Vec::new();

    for raw_line in input.lines() {
        let line = raw_line.replace('\u{feff}', "").trim().to_string();

        let line = line.as_str();
        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Skip comments
        if line.starts_with('*') {
            continue;
        }

        // Normalize for matching
        let upper = line.to_uppercase();

        // Skip COBOL divisions / metadata
        if upper.starts_with("IDENTIFICATION DIVISION")
            || upper.starts_with("ENVIRONMENT DIVISION")
            || upper.starts_with("DATA DIVISION")
            || upper.starts_with("PROCEDURE DIVISION")
            || upper.starts_with("WORKING-STORAGE SECTION")
            || upper.starts_with("FILE SECTION")
            || upper.starts_with("LINKAGE SECTION")
            || upper.starts_with("PROGRAM-ID")
            || upper.starts_with("AUTHOR")
            || upper.starts_with("DATE-WRITTEN")
        {
            continue;
        }

        

        let stmt = parse_statement(line)?;
        statements.push(stmt);
    }

    Ok(statements)
}

fn parse_statement(line: &str) -> Result<Statement> {
    let upper = line.trim().to_uppercase();

    if upper.starts_with("END-") {
        return Ok(Statement::NoOp);
    }

    // Ignore PIC declarations
    if line.to_uppercase().contains("PIC ") {
        return Ok(Statement::NoOp);
    }

    let clean = line.replace('.', "");
    let parts: Vec<&str> = clean.split_whitespace().collect();

    if parts.is_empty() {
        anyhow::bail!("Empty statement");
    }

    match parts[0].to_lowercase().as_str() {
        "display" => {
            if parts.len() < 2 {
                anyhow::bail!("Invalid DISPLAY");
            }

            Ok(Statement::Display {
                value: Literal::String(parts[1..].join(" ")),
            })
        }

        "move" => {
            if parts.len() < 4 {
                anyhow::bail!("Invalid MOVE");
            }

            Ok(Statement::Move {
                source: Source::Variable(parts[1].to_string()),
                target: parts[3].to_string(),
            })
        }

        "add" => {
            if parts.len() < 4 {
                anyhow::bail!("Invalid ADD");
            }

            Ok(Statement::Add {
                value: parts[1].parse::<i64>().unwrap_or(0),
                target: parts[3].to_string(),
            })
        }

        "perform" => {
            if parts.len() >= 3 && parts[1].eq_ignore_ascii_case("until") {
                let cond = Condition {
                    left: parts[2].to_string(),
                    operator: ">".to_string(),
                    right: 0,
                };

                Ok(Statement::PerformUntil {
                    condition: cond,
                    body: Vec::new(),
                })
            } else if parts.len() >= 2 {
                Ok(Statement::Perform {
                    name: parts[1].to_string(),
                })
            } else {
                anyhow::bail!("Invalid PERFORM");
            }
        }

        "call" => {
            if parts.len() < 2 {
                anyhow::bail!("Invalid CALL");
            }

            Ok(Statement::Call {
                program: parts[1].replace('"', ""),
                using_args: Vec::new(),
            })
        }

        "if" => {
            let cond = Condition {
                left: "X".to_string(),
                operator: "=".to_string(),
                right: 1,
            };

            Ok(Statement::If {
                condition: cond,
                then_branch: Vec::new(),
                else_branch: Some(Vec::new()),
            })
        }

        "stop" => {
            if parts.len() >= 2 && parts[1].eq_ignore_ascii_case("run") {
                Ok(Statement::NoOp)
            } else {
                anyhow::bail!("Invalid STOP statement");
            }
        }
        "compute" => {
    if parts.len() < 4 {
        anyhow::bail!("Invalid COMPUTE");
    }

    let target = parts[1].to_string();

    let expression = parts[3..].join(" ");

    Ok(Statement::Compute {
        target,
        expression,
    })
}
        "end-perform" => Ok(Statement::NoOp),
        _ => {
            anyhow::bail!("Unknown statement: {}", line)
        }
    }
}

