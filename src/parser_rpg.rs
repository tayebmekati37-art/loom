use crate::ir::{Statement, Source};

pub fn parse_program(input: &str) -> Result<Vec<Statement>, anyhow::Error> {
    // Remove UTF-8 BOM if present
    let input = input.trim_start_matches('\u{feff}');
    let mut statements = Vec::new();
    for line in input.lines() {
        // For fixed-format RPG, code starts at column 7 (index 6)
        let code = if line.len() >= 72 { &line[6..72] } else { line };
        let trimmed = code.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Split by whitespace
        let mut tokens: Vec<&str> = trimmed.split_whitespace().collect();
        // If the first token is 'C' (calculation spec), remove it
        if tokens.first() == Some(&"C") {
            tokens.remove(0);
        }
        if tokens.len() < 3 {
            continue;
        }
        let op = tokens[0].to_uppercase();
        match op.as_str() {
            "MOVE" => {
                if tokens.len() < 4 || tokens[2].to_uppercase() != "TO" {
                    anyhow::bail!("Invalid MOVE: {}", line);
                }
                let source = if let Ok(num) = tokens[1].parse::<i64>() {
                    Source::Literal(num)
                } else {
                    Source::Variable(tokens[1].to_string())
                };
                let target = tokens[3].to_string();
                statements.push(Statement::Move { source, target });
            }
            "ADD" => {
                if tokens.len() < 4 || tokens[2].to_uppercase() != "TO" {
                    anyhow::bail!("Invalid ADD: {}", line);
                }
                let value = tokens[1].parse::<i64>()?;
                let target = tokens[3].to_string();
                statements.push(Statement::Add { target, value });
            }
            _ => {
                eprintln!("Unsupported opcode: {} (ignored)", op);
            }
        }
    }
    Ok(statements)
}
