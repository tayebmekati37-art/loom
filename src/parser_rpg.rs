use crate::ir::{Statement, Source, Literal};

pub fn parse_program(input: &str) -> Result<Vec<Statement>, anyhow::Error> {
    let input = input.trim_start_matches('\u{feff}');
    let mut statements = Vec::new();
    for line in input.lines() {
        // For fixed-format RPG, code starts at column 7 (index 6) if line length >= 72
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
                // Look for "TO" in the tokens (case-insensitive)
                let to_pos = tokens.iter().position(|&t| t.to_uppercase() == "TO");
                if to_pos.is_none() || to_pos.unwrap() + 1 >= tokens.len() {
                    anyhow::bail!("Invalid MOVE statement: {}", line);
                }
                let to_idx = to_pos.unwrap();
                // Source is everything between op and TO? For simplicity, we take token[1] as source
                let source_token = tokens[1];
                let source = if let Ok(num) = source_token.parse::<i64>() {
                    Source::Literal(num)
                } else {
                    Source::Variable(source_token.to_string())
                };
                let target = tokens[to_idx + 1].to_string();
                statements.push(Statement::Move { source, target });
            }
            "ADD" | "SUB" | "MULT" | "DIV" => {
                let to_pos = tokens.iter().position(|&t| t.to_uppercase() == "TO");
                if to_pos.is_none() || to_pos.unwrap() + 1 >= tokens.len() {
                    anyhow::bail!("Invalid {} statement: {}", op, line);
                }
                let to_idx = to_pos.unwrap();
                let value_token = tokens[1];
                let value = value_token.parse::<i64>()?;
                let target = tokens[to_idx + 1].to_string();
                match op.as_str() {
                    "ADD" => statements.push(Statement::Add { target, value }),
                    "SUB" => statements.push(Statement::Add { target, value: -value }),
                    "MULT" | "DIV" => {
                        statements.push(Statement::Display { value: Literal::String(format!("# {} {} TO {} not implemented", op, value, target)) });
                    }
                    _ => {}
                }
            }
            "DISPLAY" | "DSPLY" => {
                if tokens.len() < 2 {
                    anyhow::bail!("Invalid DISPLAY: {}", line);
                }
                let lit_str = tokens[1..].join(" ");
                let lit = if let Ok(num) = lit_str.parse::<i64>() {
                    Literal::Int(num)
                } else {
                    Literal::String(lit_str.trim_matches('\'').to_string())
                };
                statements.push(Statement::Display { value: lit });
            }
            _ => {
                eprintln!("Unsupported opcode: {} (ignored)", op);
            }
        }
    }
    Ok(statements)
}