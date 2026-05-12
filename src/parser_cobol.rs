use crate::ir::{Statement, Source, Literal, Condition, WhenClause, WhenCondition, StringSource, LiteralOrVariable, FileMode};

pub fn parse_program(input: &str) -> Result<Vec<Statement>, anyhow::Error> {
    let input = input.trim_start_matches('\u{feff}');
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;
    let mut statements = Vec::new();
    let mut in_procedure = false;
    let mut in_data = false;

    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() { i += 1; continue; }
        let lower = line.to_lowercase();

        if lower.starts_with("identification division") ||
           lower.starts_with("environment division") {
            i += 1; continue;
        }
        if lower.starts_with("data division") {
            in_data = true;
            i += 1; continue;
        }
        if lower.starts_with("procedure division") {
            in_procedure = true;
            in_data = false;
            i += 1; continue;
        }
        if in_data {
            // ignore data division for parsing statements
            i += 1; continue;
        }
        if !in_procedure { i += 1; continue; }

        // Now parse statements inside PROCEDURE DIVISION
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() { i += 1; continue; }

        match parts[0].to_lowercase().as_str() {
            "move" => {
                if parts.len() < 4 || parts[2].to_lowercase() != "to" {
                    anyhow::bail!("Invalid MOVE: {}", line);
                }
                let source_part = parts[1];
                let target_part = parts[3];
                // Check if target is an array element
                if let Some((name, idx)) = parse_array_ref(target_part) {
                    let source = if let Ok(num) = source_part.parse::<i64>() {
                        Source::Literal(num)
                    } else if source_part.starts_with('\'') {
                        let mut full = source_part.to_string();
                        let mut j = 2;
                        while j < parts.len() && !full.ends_with('\'') {
                            full.push(' ');
                            full.push_str(parts[j]);
                            j += 1;
                        }
                        Source::LiteralString(full[1..full.len()-1].to_string())
                    } else {
                        Source::Variable(source_part.to_string())
                    };
                    statements.push(Statement::ArraySet { name, index: idx, value: source });
                } else if let Some((name, idx)) = parse_array_ref(source_part) {
                    // Source is array element
                    let target = target_part.to_string();
                    statements.push(Statement::ArrayGet { name, index: idx, target });
                } else {
                    let source = if let Ok(num) = source_part.parse::<i64>() {
                        Source::Literal(num)
                    } else if source_part.starts_with('\'') {
                        let mut full = source_part.to_string();
                        let mut j = 2;
                        while j < parts.len() && !full.ends_with('\'') {
                            full.push(' ');
                            full.push_str(parts[j]);
                            j += 1;
                        }
                        Source::LiteralString(full[1..full.len()-1].to_string())
                    } else {
                        Source::Variable(source_part.to_string())
                    };
                    let target = target_part.to_string();
                    statements.push(Statement::Move { source, target });
                }
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
                let condition_str = line[2..].trim();
                let condition = parse_condition_str(condition_str)?;
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
                    if !l.is_empty() {
                        let stmt = parse_single_statement(l)?;
                        if in_then {
                            then_branch.push(stmt);
                        } else if let Some(ref mut v) = else_branch {
                            v.push(stmt);
                        }
                    }
                    i += 1;
                }
                statements.push(Statement::If { condition, then_branch, else_branch });
            }
            "perform" => {
                if parts.len() != 2 {
                    anyhow::bail!("Invalid PERFORM: {}", line);
                }
                statements.push(Statement::Perform { name: parts[1].to_string() });
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
            "evaluate" => {
                // Simplified version – implement if needed
                eprintln!("EVALUATE not fully implemented, ignoring");
                // skip until END-EVALUATE
                while i < lines.len() && !lines[i].trim().to_lowercase().starts_with("end-evaluate") {
                    i += 1;
                }
                i += 1;
            }
            "string" => {
                eprintln!("STRING not implemented, ignoring");
                while i < lines.len() && !lines[i].trim().to_lowercase().starts_with("end-string") {
                    i += 1;
                }
                i += 1;
            }
            "unstring" => {
                eprintln!("UNSTRING not implemented, ignoring");
                while i < lines.len() && !lines[i].trim().to_lowercase().starts_with("end-unstring") {
                    i += 1;
                }
                i += 1;
            }
            "compute" => {
                if parts.len() < 4 || parts[2].to_lowercase() != "=" {
                    anyhow::bail!("Invalid COMPUTE: {}", line);
                }
                let target = parts[1].to_string();
                let expr = parts[3..].join(" ");
                statements.push(Statement::Compute { target, expr });
                i += 1;
            }
            "open" | "read" | "write" | "close" => {
                eprintln!("File I/O stubbed, ignoring: {}", line);
                i += 1;
            }
            _ => {
                if line.ends_with('.') && parts.len() == 1 {
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
            if parts.len() < 4 || parts[2].to_lowercase() != "to" {
                anyhow::bail!("Invalid MOVE: {}", line);
            }
            let source = if let Ok(num) = parts[1].parse::<i64>() {
                Source::Literal(num)
            } else if parts[1].starts_with('\'') {
                let full = parts[1].to_string();
                Source::LiteralString(full[1..full.len()-1].to_string())
            } else {
                Source::Variable(parts[1].to_string())
            };
            let target = parts[3].to_string();
            Ok(Statement::Move { source, target })
        }
        "add" => {
            if parts.len() < 4 || parts[2].to_lowercase() != "to" {
                anyhow::bail!("Invalid ADD: {}", line);
            }
            Ok(Statement::Add { target: parts[3].to_string(), value: parts[1].parse::<i64>()? })
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
        _ => anyhow::bail!("Unsupported statement in block: {}", line),
    }
}

fn parse_condition_str(s: &str) -> Result<Condition, anyhow::Error> {
    let s = s.trim_end_matches(" then").trim_end_matches(" THEN");
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid condition: {}", s);
    }
    Ok(Condition { left: parts[0].to_string(), operator: parts[1].to_string(), right: parts[2].parse::<i64>()? })
}

fn parse_array_ref(s: &str) -> Option<(String, i64)> {
    let s = s.trim();
    if let Some(paren) = s.find('(') {
        if let Some(close) = s.find(')') {
            let name = s[..paren].to_string();
            let idx_str = &s[paren+1..close];
            if let Ok(idx) = idx_str.parse::<i64>() {
                return Some((name, idx));
            }
        }
    }
    None
}