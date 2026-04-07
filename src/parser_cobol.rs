use crate::ir::{Statement, Source, Literal, Condition, WhenClause, WhenCondition};

pub fn parse_program(input: &str) -> Result<Vec<Statement>, anyhow::Error> {
    let input = input.trim();
    let mut statements = Vec::new();
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0].to_lowercase().as_str() {
            "move" => {
                if parts.len() != 4 || parts[2].to_lowercase() != "to" {
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
            "add" => {
                if parts.len() != 4 || parts[2].to_lowercase() != "to" {
                    anyhow::bail!("Invalid ADD statement: {}", line);
                }
                let value = parts[1].parse::<i64>()?;
                let target = parts[3].to_string();
                statements.push(Statement::Add { target, value });
            }
            "if" => {
                eprintln!("IF not fully implemented, ignoring: {}", line);
            }
            "perform" => {
                if parts.len() != 2 {
                    anyhow::bail!("Invalid PERFORM statement: {}", line);
                }
                let name = parts[1].to_string();
                statements.push(Statement::Perform { name });
            }
            "while" => {
                eprintln!("WHILE not implemented, ignoring: {}", line);
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
            _ => anyhow::bail!("Unknown statement: {}", line),
        }
    }
    Ok(statements)
}
