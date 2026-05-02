use crate::ir::{Statement, Source, Literal, Condition, WhenClause, WhenCondition, FileMode, LiteralOrVariable, StringSource};

pub fn parse_program(input: &str) -> Result<Vec<Statement>, anyhow::Error> {
    let input = input.trim_start_matches('\u{feff}');
    let lines: Vec<&str> = input.lines().collect();
    let mut statements = Vec::new();
    let mut in_procedure = false;

    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let lower = line.to_lowercase();

        // Skip division and section headers
        if lower.starts_with("identification division")
            || lower.starts_with("environment division")
            || lower.starts_with("data division")
            || lower.starts_with("working-storage section")
            || lower.starts_with("file section")
        {
            continue;
        }

        if lower.starts_with("procedure division") {
            in_procedure = true;
            continue;
        }

        if !in_procedure {
            continue;
        }

        // Now we are inside PROCEDURE DIVISION – parse statements
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0].to_lowercase().as_str() {
            "add" => {
                if parts.len() < 4 || parts[2].to_lowercase() != "to" {
                    anyhow::bail!("Invalid ADD statement: {}", line);
                }
                let value = parts[1].parse::<i64>()?;
                let target = parts[3].to_string();
                statements.push(Statement::Add { target, value });
            }
            "move" => {
                if parts.len() < 4 || parts[2].to_lowercase() != "to" {
                    anyhow::bail!("Invalid MOVE statement: {}", line);
                }
                let source = if let Ok(num) = parts[1].parse::<i64>() {
                    Source::Literal(num)
                } else {
                    Source::Variable(parts[1].to_string())
                };
                let target = parts[3].to_string();
                statements.push(Statement::Move { source, target });
            }
            "if" => {
                // For simplicity, ignore complex IF (can be implemented later)
                eprintln!("IF not implemented, ignoring: {}", line);
            }
            "perform" => {
                if parts.len() != 2 {
                    anyhow::bail!("Invalid PERFORM statement: {}", line);
                }
                let name = parts[1].to_string();
                statements.push(Statement::Perform { name });
            }
            "display" => {
                if parts.len() < 2 {
                    anyhow::bail!("Invalid DISPLAY statement: {}", line);
                }
                let lit_str = parts[1..].join(" ");
                let lit = if let Ok(num) = lit_str.parse::<i64>() {
                    Literal::Int(num)
                } else {
                    Literal::String(lit_str.trim_matches('\'').to_string())
                };
                statements.push(Statement::Display { value: lit });
            }
            "evaluate" => {
                eprintln!("EVALUATE not implemented, ignoring: {}", line);
            }
            "string" | "unstring" => {
                eprintln!("{} not implemented, ignoring: {}", parts[0], line);
            }
            "open" | "read" | "write" | "close" => {
                eprintln!("File I/O not fully implemented, ignoring: {}", line);
            }
            _ => {
                // Ignore paragraph names (lines ending with a period) and other unknown words
                if !line.ends_with('.') || parts.len() == 1 {
                    eprintln!("Unknown statement: {} (ignored)", line);
                }
            }
        }
    }
    Ok(statements)
}