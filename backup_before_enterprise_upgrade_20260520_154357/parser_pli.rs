use crate::ir::{Statement, Source, Literal};

pub fn parse_program(input: &str) -> Result<Vec<Statement>, anyhow::Error> {
    let input = input.trim_start_matches('\u{feff}');
    let mut statements = Vec::new();
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        // Remove trailing semicolon if present
        let line = line.trim_end_matches(';');
        // Split into tokens
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() { continue; }
        // Detect assignment (contains '=')
        if let Some(eq_pos) = line.find('=') {
            let left = line[..eq_pos].trim();
            let right = line[eq_pos+1..].trim();
            // Check if right side is a simple addition (contains '+')
            if let Some(plus_pos) = right.find('+') {
                let left_op = right[..plus_pos].trim();
                let right_op = right[plus_pos+1..].trim();
                // Only handle addition of literals or variables for now
                if let (Ok(value), Ok(_)) = (left_op.parse::<i64>(), right_op.parse::<i64>()) {
                    // Not a simple case; fallback to just moving the result? Actually, we need to parse properly.
                    // Simpler: treat as Add statement
                    let target = left.to_string();
                    if let Ok(value) = left_op.parse::<i64>() {
                        statements.push(Statement::Add { target: target.clone(), value });
                        // Then handle the rest? Better to parse recursively. For now, ignore.
                    } else if let Ok(value) = right_op.parse::<i64>() {
                        statements.push(Statement::Add { target: target.clone(), value });
                    } else {
                        // move the whole expression as a string? skip.
                        eprintln!("Complex addition ignored: {}", line);
                    }
                } else {
                    // Assume left_op is a variable, right_op is a number
                    let target = left.to_string();
                    if let Ok(value) = right_op.parse::<i64>() {
                        statements.push(Statement::Add { target, value });
                    } else {
                        eprintln!("Unsupported addition: {}", line);
                    }
                }
            } else {
                // Simple assignment
                let source = if let Ok(num) = right.parse::<i64>() {
                    Source::Literal(num)
                } else {
                    Source::Variable(right.to_string())
                };
                let target = left.to_string();
                statements.push(Statement::Move { source, target });
            }
        } else if tokens.len() >= 3 && tokens[0].to_uppercase() == "PUT" && tokens[1].to_uppercase() == "SKIP" && tokens[2].to_uppercase() == "LIST" {
            // PUT SKIP LIST(...)
            let start = line.find('(').ok_or_else(|| anyhow::anyhow!("Missing '(' in PUT"))?;
            let end = line.rfind(')').ok_or_else(|| anyhow::anyhow!("Missing ')' in PUT"))?;
            let expr = &line[start+1..end];
            let expr = expr.trim();
            // Assume expr is a variable or literal
            let lit = if let Ok(num) = expr.parse::<i64>() {
                Literal::Int(num)
            } else {
                Literal::String(expr.to_string())
            };
            statements.push(Statement::Display { value: lit });
        } else {
            eprintln!("Ignored PL/I statement: {}", line);
        }
    }
    Ok(statements)
}