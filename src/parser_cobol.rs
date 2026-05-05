use crate::ir::{Statement, Source, Literal, Condition, WhenClause, WhenCondition, FileMode, LiteralOrVariable, StringSource};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

// Global map to store picture information for variables
pub static PICTURES: Lazy<Mutex<HashMap<String, Picture>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone)]
pub struct Picture {
    pub integer_digits: u32,
    pub fractional_digits: u32,
}

pub fn parse_program(input: &str) -> Result<Vec<Statement>, anyhow::Error> {
    let input = input.trim_start_matches('\u{feff}');
    let lines: Vec<&str> = input.lines().collect();
    let mut statements = Vec::new();
    let mut i = 0;
    let mut in_procedure = false;
    let mut in_data = false;

    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() {
            i += 1;
            continue;
        }
        let lower = line.to_lowercase();

        if lower.starts_with("data division") {
            in_data = true;
            i += 1;
            continue;
        }
        if lower.starts_with("procedure division") {
            in_procedure = true;
            in_data = false;
            i += 1;
            continue;
        }
        if lower.starts_with("identification division") || lower.starts_with("environment division") {
            i += 1;
            continue;
        }

        if in_data {
            // Parse PIC clause
            if lower.contains(" pic ") || lower.contains(" picture ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                // Expect something like "01 WS-AMOUNT PIC 9(5)V99."
                // Find the variable name (second token) and the picture after "PIC"
                let var_name = if parts.len() >= 2 { parts[1].to_string() } else { continue };
                let pic_idx = parts.iter().position(|&p| p.to_lowercase() == "pic" || p.to_lowercase() == "picture");
                if let Some(idx) = pic_idx {
                    if idx + 1 < parts.len() {
                        let pic_str = parts[idx + 1].to_lowercase();
                        let (int_digits, frac_digits) = parse_picture(&pic_str);
                        let pic = Picture { integer_digits: int_digits, fractional_digits: frac_digits };
                        PICTURES.lock().unwrap().insert(var_name, pic);
                    }
                }
            }
            i += 1;
            continue;
        }

        if !in_procedure {
            i += 1;
            continue;
        }

        // --- PROCEDURE DIVISION parsing (same as before) ---
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
                        } else {
                            if let Some(ref mut v) = else_branch {
                                v.push(stmt);
                            }
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
                // Paragraph label
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
        _ => anyhow::bail!("Unsupported statement in block: {}", line),
    }
}

fn parse_condition_str(s: &str) -> Result<Condition, anyhow::Error> {
    let s = s.trim_end_matches(" then").trim_end_matches(" THEN");
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid condition: {}", s);
    }
    let left = parts[0].to_string();
    let operator = parts[1].to_string();
    let right = parts[2].parse::<i64>()?;
    Ok(Condition { left, operator, right })
}

fn parse_picture(pic: &str) -> (u32, u32) {
    // Simple parser: e.g., "9(5)v99" -> 5 integer, 2 fractional
    let pic = pic.trim_end_matches('.');
    let mut int_digits = 0;
    let mut frac_digits = 0;
    if let Some(v_pos) = pic.find('v') {
        let int_part = &pic[..v_pos];
        let frac_part = &pic[v_pos+1..];
        int_digits = parse_digit_count(int_part);
        frac_digits = parse_digit_count(frac_part);
    } else {
        int_digits = parse_digit_count(pic);
    }
    (int_digits, frac_digits)
}

fn parse_digit_count(s: &str) -> u32 {
    if s.contains('(') {
        let start = s.find('(').unwrap() + 1;
        let end = s.find(')').unwrap();
        s[start..end].parse().unwrap_or(0)
    } else {
        s.len() as u32
    }
}