<<<<<<< HEAD
use crate::ir::{Function, Statement, Source, Literal, Condition};
=======
﻿use crate::ir::{Function, Statement, Source, Literal, Condition};
>>>>>>> 1660d98 (Add file I/O support (OPEN, READ, WRITE, CLOSE) for COBOL to Python; fix UTF-8 by using ASCII bytes)
use std::fmt::Write;

pub fn translate(function: &Function) -> String {
    let mut out = String::new();
<<<<<<< HEAD
    writeln!(out, "fn translated_func() void {{").unwrap();
=======
    writeln!(out, "fn translated_func() void {").unwrap();
>>>>>>> 1660d98 (Add file I/O support (OPEN, READ, WRITE, CLOSE) for COBOL to Python; fix UTF-8 by using ASCII bytes)
    if function.body.is_empty() {
        writeln!(out, "    // nothing").unwrap();
    } else {
        for stmt in &function.body {
            translate_statement(stmt, &mut out, "    ");
        }
    }
<<<<<<< HEAD
    writeln!(out, "}}").unwrap();
=======
    writeln!(out, "}").unwrap();
>>>>>>> 1660d98 (Add file I/O support (OPEN, READ, WRITE, CLOSE) for COBOL to Python; fix UTF-8 by using ASCII bytes)
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
            writeln!(out, "{}if ({}) {{", indent, cond_str).unwrap();
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
            writeln!(out, "{}while ({}) {{", indent, cond_str).unwrap();
            for stmt in body {
                translate_statement(stmt, out, &format!("{}    ", indent));
            }
            writeln!(out, "{}}}", indent).unwrap();
        }
        Statement::Display { value } => {
            let expr = match value {
                Literal::Int(i) => i.to_string(),
                Literal::String(s) => s.clone(),
            };
<<<<<<< HEAD
            writeln!(out, "{}std.debug.print(\"{{}}\\n\", .{{{expr}}});", indent).unwrap();
        }
        Statement::Evaluate { .. } => {}
    }
}
=======
            writeln!(out, "{}println!(\"{}\", {});", indent, expr, expr).unwrap();
        }
        _ => {}
    }
}
>>>>>>> 1660d98 (Add file I/O support (OPEN, READ, WRITE, CLOSE) for COBOL to Python; fix UTF-8 by using ASCII bytes)
