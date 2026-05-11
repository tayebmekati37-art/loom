use crate::ir::{Statement, Source, Literal, Condition, WhenClause, WhenCondition, StringSource, LiteralOrVariable};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

pub static PICTURES: Lazy<Mutex<HashMap<String, Picture>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone)]
pub struct Picture {
    pub integer_digits: u32,
    pub fractional_digits: u32,
}

// Full parser with PROCEDURE DIVISION only – skips DIVISION headers
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

        // Divisions
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
            // Capture PIC (simplified)
            if lower.contains(" pic ") || lower.contains("picture") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let var_name = parts[1].trim_end_matches('.').to_string();
                    let pic_idx = parts.iter().position(|&p| p.to_lowercase() == "pic" || p.to_lowercase() == "picture");
                    if let Some(idx) = pic_idx {
                        if idx + 1 < parts.len() {
                            let pic_str = parts[idx+1].to_lowercase();
                            let (int, frac) = parse_picture(&pic_str);
                            PICTURES.lock().unwrap().insert(var_name, Picture { integer_digits: int, fractional_digits: frac });
                        }
                    }
                }
            }
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
                let source = if let Ok(num) = parts[1].parse::<i64>() {
                    Source::Literal(num)
                } else if parts[1].starts_with('\'') {
                    // reconstruct quoted string
                    let mut full = parts[1].to_string();
                    let mut j = 2;
                    while j < parts.len() && !full.ends_with('\'') {
                        full.push(' ');
                        full.push_str(parts[j]);
                        j += 1;
                    }
                    Source::LiteralString(full[1..full.len()-1].to_string())
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
                        } else if let Some(ref mut v) = else_branch {
                            v.push(stmt);
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
                let subject = parts[1].to_string();
                let also_subject = if parts.len() > 2 && parts[2].to_lowercase() == "also" {
                    Some(parts[3].to_string())
                } else { None };
                i += 1;
                let mut when_clauses = Vec::new();
                while i < lines.len() {
                    let l = lines[i].trim();
                    if l.to_lowercase().starts_with("end-evaluate") {
                        i += 1;
                        break;
                    }
                    let l_parts: Vec<&str> = l.split_whitespace().collect();
                    if l_parts.is_empty() { i += 1; continue; }
                    if l_parts[0].to_lowercase() == "when" {
                        let cond_val = l_parts[1].to_string();
                        let condition = if let Ok(num) = cond_val.parse::<i64>() {
                            WhenCondition::Literal(Literal::Int(num))
                        } else {
                            WhenCondition::Variable(cond_val)
                        };
                        i += 1;
                        let mut when_body = Vec::new();
                        while i < lines.len() {
                            let bl = lines[i].trim();
                            if bl.is_empty() { i += 1; continue; }
                            if bl.to_lowercase().starts_with("when") || bl.to_lowercase().starts_with("end-evaluate") {
                                break;
                            }
                            when_body.push(parse_single_statement(bl)?);
                            i += 1;
                        }
                        when_clauses.push(WhenClause { condition, body: when_body });
                    } else {
                        anyhow::bail!("Expected WHEN inside EVALUATE, got: {}", l);
                    }
                }
                statements.push(Statement::Evaluate { subject, also_subject, when_clauses });
            }
            "string" => {
                let first = line.to_lowercase().strip_prefix("string").unwrap().trim().to_string();
                let mut parts_vec = vec![first];
                i += 1;
                while i < lines.len() {
                    let l = lines[i].trim();
                    if l.to_lowercase().starts_with("end-string") {
                        i += 1;
                        break;
                    }
                    if !l.is_empty() { parts_vec.push(l.to_string()); }
                    i += 1;
                }
                let full = parts_vec.join(" ");
                let into_idx = full.to_lowercase().find(" into ").ok_or_else(|| anyhow::anyhow!("Missing INTO in STRING"))?;
                let before = &full[..into_idx];
                let after = &full[into_idx + 6..];
                let into_var = after.split_whitespace().next().unwrap().to_string();
                let mut sources = Vec::new();
                for token in before.split_whitespace() {
                    if token.starts_with('\'') {
                        let s = token.trim_matches('\'');
                        sources.push(StringSource { source: LiteralOrVariable::Literal(Literal::String(s.to_string())), delimited_by: None });
                    } else if let Ok(num) = token.parse::<i64>() {
                        sources.push(StringSource { source: LiteralOrVariable::Literal(Literal::Int(num)), delimited_by: None });
                    } else {
                        sources.push(StringSource { source: LiteralOrVariable::Variable(token.to_string()), delimited_by: None });
                    }
                }
                statements.push(Statement::String { sources, into: into_var, pointer: None });
            }
            "unstring" => {
                eprintln!("UNSTRING not implemented in parser, ignoring");
                i += 1;
                // skip until end-unstring
                while i < lines.len() && !lines[i].trim().to_lowercase().starts_with("end-unstring") {
                    i += 1;
                }
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
    if parts.is_empty() { anyhow::bail!("Empty statement"); }
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
            Ok(Statement::Move { source, target: parts[3].to_string() })
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
            if parts.len() != 2 { anyhow::bail!("Invalid PERFORM: {}", line); }
            Ok(Statement::Perform { name: parts[1].to_string() })
        }
        _ => anyhow::bail!("Unsupported statement in block: {}", line),
    }
}

fn parse_condition_str(s: &str) -> Result<Condition, anyhow::Error> {
    let s = s.trim_end_matches(" then").trim_end_matches(" THEN");
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 3 { anyhow::bail!("Invalid condition: {}", s); }
    Ok(Condition { left: parts[0].to_string(), operator: parts[1].to_string(), right: parts[2].parse::<i64>()? })
}

fn parse_picture(pic: &str) -> (u32, u32) {
    let pic = pic.trim_end_matches('.');
    let (int, frac) = if let Some(v) = pic.find('v') {
        (digit_count(&pic[..v]), digit_count(&pic[v+1..]))
    } else {
        (digit_count(pic), 0)
    };
    (int, frac)
}
fn digit_count(s: &str) -> u32 {
    if s.contains('(') {
        let start = s.find('(').unwrap() + 1;
        let end = s.find(')').unwrap();
        s[start..end].parse().unwrap_or(0)
    } else {
        s.len() as u32
    }
}