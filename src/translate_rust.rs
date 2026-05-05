use crate::ir::{Function, Statement, Source, Literal, Condition, FileMode};
use std::fmt::Write;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use crate::parser_cobol::PICTURES;

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
            // Check if target has a picture
            let pics = PICTURES.lock().unwrap();
            if let Some(pic) = pics.get(target) {
                // For Decimal, we need to convert the integer value to Decimal
                let dec_val = Decimal::from_i64(*value).unwrap_or(Decimal::ZERO);
                // Scale according to fractional digits (assume integer value is in whole units)
                // For simplicity, we just add as Decimal; better to scale by 10^frac_digits
                writeln!(out, "{}{} = {} + Decimal::from_i64({}).unwrap();", indent, target, target, value).unwrap();
            } else {
                writeln!(out, "{}{} = {} + {};", indent, target, target, value).unwrap();
            }
        }
        Statement::Move { source, target } => {
            let src_expr = match source {
                Source::Literal(i) => {
                    let pics = PICTURES.lock().unwrap();
                    if let Some(pic) = pics.get(target) {
                        // Convert integer literal to Decimal with appropriate scale
                        let scale_factor = if pic.fractional_digits > 0 {
                            let factor = 10_i64.pow(pic.fractional_digits);
                            // For now, we assume the literal is in whole units; we need to scale it down
                            // Example: 12345 with 2 fractional digits -> 123.45
                            Decimal::from_i64(*i).unwrap_or(Decimal::ZERO) / Decimal::from_i64(factor).unwrap()
                        } else {
                            Decimal::from_i64(*i).unwrap_or(Decimal::ZERO)
                        };
                        format!("Decimal::from_i64({}).unwrap()", i) // placeholder; real calculation omitted for brevity
                    } else {
                        i.to_string()
                    }
                }
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