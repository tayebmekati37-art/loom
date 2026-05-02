use crate::ir::{Function, Statement, Source, Literal, FileMode};
use std::fmt::Write;



pub fn translate(function: &Function) -> String {
    let mut out = String::new();
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
            writeln!(out, "{}{} = {} + {};", indent, target, target, value).unwrap();
        }
        Statement::Move { source, target } => {
            let src_expr = match source {
                Source::Literal(i) => i.to_string(),
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
            let expr = match value {
                Literal::Int(i) => i.to_string(),
                Literal::String(s) => format!("{:?}", s),
            };
            writeln!(out, "{}println!(\"{}\");", indent, expr).unwrap();
        }
        Statement::Evaluate { .. } => {
            writeln!(out, "{}// EVALUATE not implemented in Rust translator", indent).unwrap();
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
        _ => {}
    }
}
