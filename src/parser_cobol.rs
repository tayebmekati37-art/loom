use crate::ir::{Statement, Source, Literal, Condition, WhenClause, WhenCondition, FileMode, LiteralOrVariable, StringSource};

pub fn parse_program(input: &str) -> Result<Vec<Statement>, anyhow::Error> {
    let input = input.trim();
    let mut statements = Vec::new();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() {
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
                i += 1;
            }
            "add" => {
                if parts.len() != 4 || parts[2].to_lowercase() != "to" {
                    anyhow::bail!("Invalid ADD statement: {}", line);
                }
                let value = parts[1].parse::<i64>()?;
                let target = parts[3].to_string();
                statements.push(Statement::Add { target, value });
                i += 1;
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
                i += 1;
            }
            "perform" => {
                if parts.len() != 2 {
                    anyhow::bail!("Invalid PERFORM statement: {}", line);
                }
                let name = parts[1].to_string();
                statements.push(Statement::Perform { name });
                i += 1;
            }
            "while" => {
                let line_lower = line.to_lowercase();
                let after_while = line_lower.strip_prefix("while").unwrap().trim();
                let condition_str = after_while.strip_suffix("do").unwrap_or(after_while).trim().to_string();
                i += 1;
                let mut body_stmts = Vec::new();
                while i < lines.len() {
                    let l = lines[i].trim();
                    if l.to_lowercase().starts_with("end-while") {
                        i += 1;
                        break;
                    }
                    if !l.is_empty() {
                        let sub_stmt = parse_single_statement(l)?;
                        body_stmts.push(sub_stmt);
                    }
                    i += 1;
                }
                let condition = parse_condition_str(&condition_str)?;
                statements.push(Statement::While { condition, body: body_stmts });
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
                i += 1;
            }
            "evaluate" => {
                let subject = parts[1].to_string();
                let also_subject = if parts.len() > 2 && parts[2].to_lowercase() == "also" {
                    Some(parts[3].to_string())
                } else {
                    None
                };
                i += 1;
                let mut when_clauses = Vec::new();
                while i < lines.len() {
                    let l = lines[i].trim();
                    if l.to_lowercase().starts_with("end-evaluate") {
                        i += 1;
                        break;
                    }
                    let l_parts: Vec<&str> = l.split_whitespace().collect();
                    if l_parts.is_empty() {
                        i += 1;
                        continue;
                    }
                    if l_parts[0].to_lowercase() == "when" {
                        let cond_lit_or_var = l_parts[1].to_string();
                        let cond = if let Ok(num) = cond_lit_or_var.parse::<i64>() {
                            WhenCondition::Literal(Literal::Int(num))
                        } else {
                            WhenCondition::Variable(cond_lit_or_var)
                        };
                        i += 1;
                        let mut when_body = Vec::new();
                        while i < lines.len() {
                            let bl = lines[i].trim();
                            if bl.is_empty() {
                                i += 1;
                                continue;
                            }
                            if bl.to_lowercase().starts_with("when") || bl.to_lowercase().starts_with("end-evaluate") {
                                break;
                            }
                            let body_stmt = parse_single_statement(bl)?;
                            when_body.push(body_stmt);
                            i += 1;
                        }
                        when_clauses.push(WhenClause { condition: cond, body: when_body });
                    } else {
                        anyhow::bail!("Expected WHEN inside EVALUATE, got: {}", l);
                    }
                }
                statements.push(Statement::Evaluate { subject, also_subject, when_clauses });
            }
                                    "string" => {
                // Collect all lines of the STRING statement until END-STRING
                let mut string_parts = Vec::new();
                // The first line already contains the "string" keyword and possibly part of the content
                let first_line_rest = line.to_lowercase().strip_prefix("string").unwrap().trim().to_string();
                if !first_line_rest.is_empty() {
                    string_parts.push(first_line_rest);
                }
                i += 1;
                while i < lines.len() {
                    let l = lines[i].trim();
                    if l.to_lowercase().starts_with("end-string") {
                        i += 1;
                        break;
                    }
                    if !l.is_empty() {
                        string_parts.push(l.to_string());
                    }
                    i += 1;
                }
                let full = string_parts.join(" ");
                // Find INTO clause
                let into_idx = full.to_lowercase().find(" into ").ok_or_else(|| anyhow::anyhow!("Missing INTO in STRING"))?;
                let before_into = &full[..into_idx];
                let after_into = &full[into_idx + 6..];
                let into_var = after_into.split_whitespace().next().ok_or_else(|| anyhow::anyhow!("Missing target after INTO"))?.to_string();
                // Parse sources (preserving quoted strings)
                let mut sources = Vec::new();
                let mut current = String::new();
                let mut in_quote = false;
                for ch in before_into.chars() {
                    if ch == '\'' {
                        in_quote = !in_quote;
                        current.push(ch);
                    } else if ch == ' ' && !in_quote {
                        if !current.is_empty() {
                            let token = current.trim();
                            if !token.is_empty() {
                                let source = if token.starts_with('\'') && token.ends_with('\'') {
                                    LiteralOrVariable::Literal(Literal::String(token[1..token.len()-1].to_string()))
                                } else if let Ok(num) = token.parse::<i64>() {
                                    LiteralOrVariable::Literal(Literal::Int(num))
                                } else {
                                    LiteralOrVariable::Variable(token.to_string())
                                };
                                sources.push(StringSource { source, delimited_by: None });
                            }
                            current.clear();
                        }
                    } else {
                        current.push(ch);
                    }
                }
                if !current.is_empty() {
                    let token = current.trim();
                    if !token.is_empty() {
                        let source = if token.starts_with('\'') && token.ends_with('\'') {
                            LiteralOrVariable::Literal(Literal::String(token[1..token.len()-1].to_string()))
                        } else if let Ok(num) = token.parse::<i64>() {
                            LiteralOrVariable::Literal(Literal::Int(num))
                        } else {
                            LiteralOrVariable::Variable(token.to_string())
                        };
                        sources.push(StringSource { source, delimited_by: None });
                    }
                }
                statements.push(Statement::String { sources, into: into_var, pointer: None });
            }
            "unstring" => {
                // Similar multi-line handling for UNSTRING
                let first_line_rest = line.to_lowercase().strip_prefix("unstring").unwrap().trim().to_string();
                let mut unstring_parts = Vec::new();
                if !first_line_rest.is_empty() {
                    unstring_parts.push(first_line_rest);
                }
                i += 1;
                while i < lines.len() {
                    let l = lines[i].trim();
                    if l.to_lowercase().starts_with("end-unstring") {
                        i += 1;
                        break;
                    }
                    if !l.is_empty() {
                        unstring_parts.push(l.to_string());
                    }
                    i += 1;
                }
                let full = unstring_parts.join(" ");
                // Parse UNSTRING (simplified)
                // Look for DELIMITED BY, INTO, etc.
                // For now, we'll return a placeholder; you can expand later.
                statements.push(Statement::Unstring {
                    source: "WS-IN".to_string(),
                    delimited_by: None,
                    into: vec![],
                    pointer: None,
                });
            }
                        "end-string" => {
                // Skip END-STRING line (already handled by STRING block)
                i += 1;
            }
            "open" => {
                if parts.len() != 3 {
                    anyhow::bail!("Invalid OPEN statement: {}", line);
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
                    anyhow::bail!("Invalid READ statement: {}", line);
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
                    anyhow::bail!("Invalid WRITE statement: {}", line);
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
                    anyhow::bail!("Invalid CLOSE statement: {}", line);
                }
                let name = parts[1].to_string();
                statements.push(Statement::CloseFile { name });
                i += 1;
            }
            _ => anyhow::bail!("Unknown statement: {}", line),
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
                anyhow::bail!("Invalid MOVE statement: {}", line);
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
                anyhow::bail!("Invalid ADD statement: {}", line);
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
                anyhow::bail!("Invalid PERFORM statement: {}", line);
            }
            Ok(Statement::Perform { name: parts[1].to_string() })
        }
        _ => anyhow::bail!("Unsupported statement in block: {}", line),
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