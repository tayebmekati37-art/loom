use crate::ir::{Function, Statement, Source, Literal, Condition, FileMode};
use std::fmt::Write;
use rust_decimal::Decimal;
use crate::parser_cobol::{PICTURES, RECORDS};

pub fn translate(function: &Function) -> String {
    let mut out = String::new();
    // Generate Rust structs for records
    let records = RECORDS.lock().unwrap();
    for (_, record) in records.iter() {
        writeln!(out, "#[derive(Debug)]").unwrap();
        writeln!(out, "struct {} {{", record.name).unwrap();
        for field in &record.fields {
            // Map COBOL picture to Rust type
            let rust_type = if let Some(pic) = &field.picture {
                if pic.to_lowercase().contains('x') {
                    "String".to_string()
                } else {
                    // For simplicity, use Decimal for all numeric pictures
                    "Decimal".to_string()
                }
            } else {
                "String".to_string()
            };
            writeln!(out, "    {}: {},", field.name, rust_type).unwrap();
        }
        writeln!(out, "}}\n").unwrap();
    }
    writeln!(out, "fn translated_func() -> Result<(), Box<dyn std::error::Error>> {{").unwrap();
    if function.body.is_empty() {
        writeln!(out, "    Ok(())").unwrap();
    } else {
        for stmt in &function.body {
            translate_statement(stmt, &mut out, "    ");
        }
        writeln!(out, "    Ok(())").unwrap();
    }
    writeln!(out, "}}").unwrap();
    out
}

fn translate_statement(stmt: &Statement, out: &mut String, indent: &str) {
    match stmt {
        Statement::Add { target, value } => {
            let pics = PICTURES.lock().unwrap();
            if let Some(pic) = pics.get(target) {
                writeln!(out, "{}{} = {} + Decimal::new({}, {});", indent, target, target, value, pic.fractional_digits).unwrap();
            } else {
                writeln!(out, "{}{} = {} + {};", indent, target, target, value).unwrap();
            }
        }
        Statement::Move { source, target } => {
            let src_expr = match source {
    Source::Literal(i) => i.to_string(),
    Source::LiteralString(s) => format!("{:?}", s),
    Source::Variable(v) => v.clone(),
};
            writeln!(out, "{}{} = {};", indent, target, src_expr).unwrap();
        }
        Statement::If { condition, then_branch, else_branch } => {
            let cond_str = format!("{} {} {}", condition.left, condition.operator, condition.right);
            writeln!(out, "{}if {} {{", indent, cond_str).unwrap();
            for stmt in then_branch {
                translate_statement(stmt, out, &format!("{}    ", indent));
            }
            if let Some(else_branch) = else_branch {
                writeln!(out, "{}}} else {{", indent).unwrap();
                for stmt in else_branch {
                    translate_statement(stmt, out, &format!("{}    ", indent));
                }
            }
            writeln!(out, "{}}}", indent).unwrap();
        }
        Statement::Perform { name } => {
            writeln!(out, "{}{}();", indent, name).unwrap();
        }
        Statement::While { condition, body } => {
            let cond_str = format!("{} {} {}", condition.left, condition.operator, condition.right);
            writeln!(out, "{}while {} {{", indent, cond_str).unwrap();
            for stmt in body {
                translate_statement(stmt, out, &format!("{}    ", indent));
            }
            writeln!(out, "{}}}", indent).unwrap();
        }
        Statement::Display { value } => {
            match value {
                Literal::Int(i) => {
                    writeln!(out, "{}println!(\"{}\");", indent, i).unwrap();
                }
                Literal::String(s) => {
                    if !s.starts_with('\'') {
                        writeln!(out, "{}println!(\"{}\", {});", indent, "{}", s).unwrap();
                    } else {
                        writeln!(out, "{}println!(\"{}\");", indent, s.trim_matches('\'')).unwrap();
                    }
                }
            }
        }
        Statement::OpenFile { mode, name } => {
            let mode_str = match mode {
                FileMode::Input => "std::fs::File::open",
                FileMode::Output => "std::fs::File::create",
                FileMode::IO => "std::fs::OpenOptions::new().read(true).write(true).open",
            };
            writeln!(out, "{}{} = {}?;", indent, name, mode_str).unwrap();
        }
        Statement::ReadFile { file, into } => {
            if let Some(into) = into {
                writeln!(out, "{}let mut buffer = String::new();", indent).unwrap();
                writeln!(out, "{}{}.read_to_string(&mut buffer)?;", indent, file).unwrap();
                writeln!(out, "{}{} = buffer;", indent, into).unwrap();
            } else {
                writeln!(out, "{}{}.read_to_string(&mut String::new())?;", indent, file).unwrap();
            }
        }
        Statement::WriteFile { file, from } => {
            if let Some(from) = from {
                writeln!(out, "{}{}.write_all({}.as_bytes())?;", indent, file, from).unwrap();
            } else {
                writeln!(out, "{}{}.write_all(b\"\")?;", indent, file).unwrap();
            }
        }
        Statement::CloseFile { name } => {
            writeln!(out, "{}{}.sync_all()?;", indent, name).unwrap();
        }
        _ => {
            writeln!(out, "{}// {:?} not implemented", indent, stmt).unwrap();
        }
    }
}
