use crate::ir::{Statement, Source};

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
<<<<<<< HEAD
                let value = parts[1].parse::<i64>()?;
                let target = parts[3].to_string();
                statements.push(Statement::Add { target, value });
=======
                let value_source = if let Ok(num) = parts[1].parse::<i64>() {
                    Source::Literal(num)
                } else {
                    Source::Variable(parts[1].to_string())
                };
                let target = parts[3].to_string();
                statements.push(Statement::Add { target, value: value_source });
>>>>>>> 902dbcf1dd9dcf086aff99c41645f8732529de4b
            }
            _ => anyhow::bail!("Unknown statement: {}", line),
        }
    }
    Ok(statements)
}
