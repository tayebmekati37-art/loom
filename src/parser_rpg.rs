use crate::ir::{Statement, Source, Literal};

pub fn parse_program(input: &str) -> Result<Vec<Statement>, anyhow::Error> {
    let input = input.trim_start_matches('\u{feff}');
    let mut statements = Vec::new();
    for line in input.lines() {
        let code = if line.len() >= 72 { &line[6..72] } else { line };
        let trimmed = code.trim();
        if trimmed.is_empty() { continue; }
        let mut tokens: Vec<&str> = trimmed.split_whitespace().collect();
        if tokens.first() == Some(&"C") { tokens.remove(0); }
        if tokens.len() < 3 { continue; }
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
            "ADD" | "SUB" => {
                if tokens.len() < 4 || tokens[2].to_uppercase() != "TO" {
                    anyhow::bail!("Invalid {}: {}", op, line);
                }
                let value = tokens[1].parse::<i64>()?;
                let target = tokens[3].to_string();
                let signed_value = if op == "ADD" { value } else { -value };
                statements.push(Statement::Add { target, value: signed_value });
            }
            "MULT" => {
                // Multiplication not yet implemented in IR – emit a comment and ignore
                eprintln!("MULT not implemented, ignoring: {}", line);
                // Optionally, add a comment as a Display statement
                let comment = format!("# MULT {} TO {} not implemented", tokens[1], tokens[3]);
                statements.push(Statement::Display { value: Literal::String(comment) });
            }
            "DIV" => {
                // Division not yet implemented – emit a comment
                eprintln!("DIV not implemented, ignoring: {}", line);
                let comment = format!("# DIV {} TO {} not implemented", tokens[1], tokens[3]);
                statements.push(Statement::Display { value: Literal::String(comment) });
            }
            "DISPLAY" | "DSPLY" => {
                if tokens.len() < 2 { anyhow::bail!("Invalid DISPLAY: {}", line); }
                let lit_str = tokens[1..].join(" ");
                let lit = if let Ok(num) = lit_str.parse::<i64>() {
                    Literal::Int(num)
                } else {
                    Literal::String(lit_str.trim_matches('\'').to_string())
                };
                statements.push(Statement::Display { value: lit });
            }
            "IF" => {
                eprintln!("IF not implemented, ignoring: {}", line);
            }
            _ => eprintln!("Unsupported opcode: {} (ignored)", op),
        }
    }
    Ok(statements)
}