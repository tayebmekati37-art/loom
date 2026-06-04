use crate::ir::{FileMode, Function, Literal, LiteralOrVariable, Source, Statement, WhenCondition};
use std::fmt::Write;

pub fn translate(function: &Function) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "fn translated_func() -> Result<(), Box<dyn std::error::Error>> {{"
    )
    .unwrap();
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
                Source::LiteralString(s) => format!("{:?}.to_string()", s),
                Source::Variable(v) => v.clone(),
            };
            writeln!(out, "{}{} = {};", indent, target, src_expr).unwrap();
        }
        Statement::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let cond_str = format!(
                "{} {} {}",
                condition.left, condition.operator, condition.right
            );
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
            let cond_str = format!(
                "{} {} {}",
                condition.left, condition.operator, condition.right
            );
            writeln!(out, "{}while {} {{", indent, cond_str).unwrap();
            for stmt in body {
                translate_statement(stmt, out, &format!("{}    ", indent));
            }
            writeln!(out, "{}}}", indent).unwrap();
        }
        Statement::Display { value } => match value {
            Literal::Int(i) => writeln!(out, "{}println!(\"{}\");", indent, i).unwrap(),
            Literal::String(s) => writeln!(out, "{}println!(\"{}\");", indent, s).unwrap(),
        },
        Statement::Evaluate {
            subject,
            also_subject,
            when_clauses,
        } => {
            writeln!(out, "{}match {} {{", indent, subject).unwrap();
            for when in when_clauses {
                let cond_str = match &when.condition {
                    WhenCondition::Literal(lit) => match lit {
                        Literal::Int(i) => i.to_string(),
                        Literal::String(s) => s.clone(),
                    },
                    WhenCondition::Variable(v) => v.clone(),
                };
                writeln!(out, "{}    {} => {{", indent, cond_str).unwrap();
                for stmt in &when.body {
                    translate_statement(stmt, out, &format!("{}        ", indent));
                }
                writeln!(out, "{}}}", indent).unwrap();
            }
            if let Some(also) = also_subject {
                writeln!(out, "{}    // also subject {} not supported", indent, also).unwrap();
            }
            writeln!(out, "{}}}", indent).unwrap();
        }
        Statement::String {
            sources,
            into,
            pointer,
        } => {
            let mut parts = Vec::new();
            for src in sources {
                let src_str = match &src.source {
                    LiteralOrVariable::Literal(lit) => match lit {
                        Literal::Int(i) => i.to_string(),
                        Literal::String(s) => format!("{:?}", s),
                    },
                    LiteralOrVariable::Variable(v) => v.clone(),
                };
                parts.push(src_str);
            }
            let joined = parts.join(" + ");
            writeln!(out, "{}{} = {};", indent, into, joined).unwrap();
            if let Some(ptr) = pointer {
                writeln!(out, "{}// pointer {} not implemented", indent, ptr).unwrap();
            }
        }
        Statement::Unstring {
            source,
            delimited_by,
            into,
            pointer,
        } => {
            let delim = match delimited_by {
                Some(d) => match d {
                    LiteralOrVariable::Literal(lit) => match lit {
                        Literal::Int(i) => i.to_string(),
                        Literal::String(s) => s.clone(),
                    },
                    LiteralOrVariable::Variable(v) => v.clone(),
                },
                None => " ".to_string(),
            };
            for (i, var) in into.iter().enumerate() {
                writeln!(
                    out,
                    "{}{} = {}.split('{}').nth({}).unwrap_or(\"\").to_string();",
                    indent, var, source, delim, i
                )
                .unwrap();
            }
            if let Some(ptr) = pointer {
                writeln!(out, "{}// pointer {} not implemented", indent, ptr).unwrap();
            }
        }
        Statement::Redefines { name, redefines } => {
            writeln!(
                out,
                "{}// REDEFINES {} {} not implemented",
                indent, name, redefines
            )
            .unwrap();
        }
        Statement::Occurs { name, count } => {
            writeln!(out, "{}let mut {} = vec![0; {}];", indent, name, count).unwrap();
        }
        Statement::ConditionName { name, value } => {
            let val_str = match value {
                Literal::Int(i) => i.to_string(),
                Literal::String(s) => s.clone(),
            };
            writeln!(out, "{}const {}: &str = \"{}\";", indent, name, val_str).unwrap();
        }
        Statement::Compute { target, expr } => {
            writeln!(out, "{}{} = {};", indent, target, expr).unwrap();
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
                writeln!(
                    out,
                    "{}{}.read_to_string(&mut String::new())?;",
                    indent, file
                )
                .unwrap();
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
        Statement::ArrayGet {
            name,
            index,
            target,
        } => {
            writeln!(out, "{}{} = {}[{}];", indent, target, name, index).unwrap();
        }
        Statement::ArraySet { name, index, value } => {
            let src_expr = match value {
                Source::Literal(i) => i.to_string(),
                Source::LiteralString(s) => format!("{:?}.to_string()", s),
                Source::Variable(v) => v.clone(),
            };
            writeln!(out, "{}{}[{}] = {};", indent, name, index, src_expr).unwrap();
        }
        // New features
        Statement::Accept { target } => {
            writeln!(out, "{}let mut input = String::new();", indent).unwrap();
            writeln!(
                out,
                "{}{} = std::io::stdin().read_line(&mut input).unwrap();",
                indent, target
            )
            .unwrap();
            writeln!(out, "{}{} = input.trim().parse().unwrap();", indent, target).unwrap();
        }
        Statement::StopRun => {
            writeln!(out, "{}return Ok(());", indent).unwrap();
        }
        Statement::Continue => {
            writeln!(out, "{}// CONTINUE (no-op)", indent).unwrap();
        }
        Statement::Exit => {
            writeln!(out, "{}// EXIT (no-op)", indent).unwrap();
        }
        Statement::Inspect {
            source,
            target,
            pattern,
        } => {
            writeln!(
                out,
                "{}{} = {}.matches('{}').count();",
                indent, target, source, pattern
            )
            .unwrap();
        }
        _ => {
            writeln!(out, "{}// {:?} not implemented", indent, stmt).unwrap();
        }
    }
}

