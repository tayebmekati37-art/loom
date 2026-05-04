use crate::ir::{Statement, Source, Literal, Condition, WhenClause, WhenCondition, FileMode, LiteralOrVariable, StringSource};

pub fn parse_program(input: &str) -> Result<Vec<Statement>, anyhow::Error> {
    let input = input.trim_start_matches('\u{feff}');
    let lines: Vec<&str> = input.lines().collect();
    let mut statements = Vec::new();
    let mut i = 0;
    let mut in_procedure = false;

    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() {
            i += 1;
            continue;
        }
        let lower = line.to_lowercase();

        if lower.starts_with("procedure division") {
            in_procedure = true;
            i += 1;
            continue;
        }
        if !in_procedure {
            i += 1;
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            i += 1;
            continue;
        }

        match parts[0].to_lowercase().as_str() {
            "move" => {
                if parts.len() < 4 || parts[2].to_lowercase() != "to" {
                    anyhow::bail!("Invalid MOVE: {}", line);
                }
                let source = if let Ok(num) = parts[1].parse::<i64>() {
                    Source::Literal(num)
                } else {
                    Source::Variable(parts[1].to_string())
                };
                let target = parts[3].to_string();
                statements.push(Statement::Move { source, target });
                i += 1;
            }
            "add" => {
                if parts.len() < 4 || parts[2].to_lowercase() != "to" {
                    anyhow::bail!("Invalid ADD: {}", line);
                }
                let value = parts[1].parse::<i64>()?;
                let target = parts[3].to_string();
                statements.push(Statement::Add { target, value });
                i += 1;
            }
            "if" => {
                // Parse IF condition THEN ... ELSE ... END-IF
                // This implementation assumes that the IF statement spans multiple lines.
                // We'll collect the condition, then the entire block until END-IF.
                let condition_str = line[2..].trim();; let condition_str = condition_str.trim_end_matches(" then").trim_end_matches(" THEN"); let condition = parse_condition_str(condition_str)?;
                i += 1;
                let mut then_branch = Vec::new();
                let mut else_branch = None;
                let mut in_then = true;
                while i < lines.len() {
                    let l = lines[i].trim();
                    if l.to_lowercase().starts_with("end-if") {
                        i += 1;
                        break;
                    }
                    if l.to_lowercase().starts_with("else") {
                        in_then = false;
                        else_branch = Some(Vec::new());
                        i += 1;
                        continue;
                    }
                    if l.is_empty() {
                        i += 1;
                        continue;
                    }
                    let stmt = parse_single_statement(l)?;
                    if in_then {
                        then_branch.push(stmt);
                    } else {
                        if let Some(ref mut else_vec) = else_branch {
                            else_vec.push(stmt);
                        }
                    }
                    i += 1;
                }
                statements.push(Statement::If {
                    condition,
                    then_branch,
                    else_branch,
                });
            }
            "perform" => {
                if parts.len() != 2 {
                    anyhow::bail!("Invalid PERFORM: {}", line);
                }
                let name = parts[1].to_string();
                statements.push(Statement::Perform { name });
                i += 1;
            }
            "display" => {
                if parts.len() < 2 {
                    anyhow::bail!("Invalid DISPLAY: {}", line);
                }
                let lit_str = parts[1..].join(" ");
                let lit = if let Ok(num) = lit_str.parse::<i64>() {
                    Literal::Int(num)
                } else {
                    Literal::String(lit_str.trim_matches('\'').to_string())
                };
                statements.push(Statement::Display { value: lit });
                i += 1;
            }
            "open" => {
                if parts.len() != 3 {
                    anyhow::bail!("Invalid OPEN: {}", line);
                }
                let mode_str = parts[1].to_lowercase();
                let mode = match mode_str.as_str() {
                    "input" => FileMode::Input,
                    "output" => FileMode::Output,
                    "i-o" => FileMode::IO,
                    _ => anyhow::bail!("Invalid file mode: {}", mode_str),
                };
                let name = parts[2].to_string();
                statements.push(Statement::OpenFile { mode, name });
                i += 1;
            }
            "read" => {
                if parts.len() < 2 {
                    anyhow::bail!("Invalid READ: {}", line);
                }
                let file = parts[1].to_string();
                let into = if parts.len() >= 4 && parts[2].to_lowercase() == "into" {
                    Some(parts[3].to_string())
                } else {
                    None
                };
                statements.push(Statement::ReadFile { file, into });
                i += 1;
            }
            "write" => {
                if parts.len() < 2 {
                    anyhow::bail!("Invalid WRITE: {}", line);
                }
                let file = parts[1].to_string();
                let from = if parts.len() >= 4 && parts[2].to_lowercase() == "from" {
                    Some(parts[3].to_string())
                } else {
                    None
                };
                statements.push(Statement::WriteFile { file, from });
                i += 1;
            }
            "close" => {
                if parts.len() != 2 {
                    anyhow::bail!("Invalid CLOSE: {}", line);
                }
                let name = parts[1].to_string();
                statements.push(Statement::CloseFile { name });
                i += 1;
            }
            _ => {
                // Possible paragraph name (ends with a period)
                if line.ends_with('.') && parts.len() == 1 {
                    // Paragraph label Ã¢â‚¬â€œ skip (no statement generated)
                    i += 1;
                } else {
                    anyhow::bail!("Unknown statement: {}", line);
                }
            }
        }
    }
    Ok(statements)
}

fn parse_single_statement(line: &str) -> Result<Statement, anyhow::Error> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        anyhow::bail!("Empty statement");
    }
    match parts[0].to_lowercase().as_str() {
        "move" => {
            if parts.len() != 4 || parts[2].to_lowercase() != "to" {
                anyhow::bail!("Invalid MOVE: {}", line);
            }
            let source = if let Ok(num) = parts[1].parse::<i64>() {
                Source::Literal(num)
            } else {
                Source::Variable(parts[1].to_string())
            };
            let target = parts[3].to_string();
            Ok(Statement::Move { source, target })
        }
        "add" => {
            if parts.len() != 4 || parts[2].to_lowercase() != "to" {
                anyhow::bail!("Invalid ADD: {}", line);
            }
            let value = parts[1].parse::<i64>()?;
            let target = parts[3].to_string();
            Ok(Statement::Add { target, value })
        }
        "display" => {
            let lit_str = parts[1..].join(" ");
            let lit = if let Ok(num) = lit_str.parse::<i64>() {
                Literal::Int(num)
            } else {
                Literal::String(lit_str.trim_matches('\'').to_string())
            };
            Ok(Statement::Display { value: lit })
        }
        "perform" => {
            if parts.len() != 2 {
                anyhow::bail!("Invalid PERFORM: {}", line);
            }
            Ok(Statement::Perform { name: parts[1].to_string() })
        }
        _ => anyhow::bail!("Unsupported statement in IF block: {}", line),
    }
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