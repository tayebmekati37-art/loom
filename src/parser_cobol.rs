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
               let lower_line = line.to_lowercase();
               let then_idx = lower_line.find(" then ").or_else(|| lower_line.find(" then")).ok_or_else(|| anyhow::anyhow!("Missing THEN in IF statement: {}", line))?;
               let condition_str = lower_line[3..then_idx].trim().to_string();
               let after_then = lower_line[then_idx + 5..].trim_start().to_string();
               let (then_part, else_part_opt) = if let Some(else_idx) = after_then.find(" else ") {
               (after_then[..else_idx].to_string(), Some(after_then[else_idx + 5..].trim_start().to_string()))
                } else {
               (after_then, None)
             };
               let then_part = then_part.trim_end_matches(" end-if").trim_end().to_string();
               let condition = parse_condition_str(&condition_str)?;
               let then_stmts = parse_statements_from_line(&then_part)?;
               let else_stmts = if let Some(else_part) = else_part_opt {
                let else_part = else_part.trim_end_matches(" end-if").trim_end().to_string();
                parse_statements_from_line(&else_part)?
        } else {
          vec![]
        };
             statements.push(Statement::If {
              condition,
              then_branch: then_stmts,
             else_branch: if else_stmts.is_empty() { None } else { Some(else_stmts) },
         });
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

fn parse_condition_str(s: &str) -> Result<Condition, anyhow::Error> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid condition: {}", s);
    }
    let left = parts[0].to_string();
    let operator = parts[1].to_string();
    let right = parts[2].parse::<i64>()?;
    Ok(Condition { left, operator, right })
}

fn parse_statements_from_line(line: &str) -> Result<Vec<Statement>, anyhow::Error> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(vec![]);
    }
    let stmt = match parts[0].to_lowercase().as_str() {
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
            Statement::Move { source, target }
        }
        "add" => {
            if parts.len() != 4 || parts[2].to_lowercase() != "to" {
                anyhow::bail!("Invalid ADD statement: {}", line);
            }
            let value = parts[1].parse::<i64>()?;
            let target = parts[3].to_string();
            Statement::Add { target, value }
        }
        "display" => {
            let lit_str = parts[1..].join(" ");
            let lit = if let Ok(num) = lit_str.parse::<i64>() {
                Literal::Int(num)
            } else {
                Literal::String(lit_str.trim_matches('\'').to_string())
            };
            Statement::Display { value: lit }
        }
        "perform" => {
            if parts.len() != 2 {
                anyhow::bail!("Invalid PERFORM statement: {}", line);
            }
            Statement::Perform { name: parts[1].to_string() }
        }
        _ => anyhow::bail!("Unsupported statement in IF block: {}", line),
    };
    Ok(vec![stmt])
}