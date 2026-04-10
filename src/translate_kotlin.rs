<<<<<<< HEAD
﻿use crate::ir::{Function, Statement, Source, Literal, Condition, WhenClause, WhenCondition};
=======
﻿use crate::ir::{Function, Statement, Source, Literal, Condition};
>>>>>>> 1660d98 (Add file I/O support (OPEN, READ, WRITE, CLOSE) for COBOL to Python; fix UTF-8 by using ASCII bytes)
use std::fmt::Write;

pub fn translate(function: &Function) -> String {
    let mut out = String::new();
<<<<<<< HEAD
    writeln!(out, "fun {}() {{", function.name).unwrap();
=======
    writeln!(out, "fun translated_func() {").unwrap();
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
<<<<<<< HEAD
            writeln!(out, "{}{} = {}", indent, target, src_expr).unwrap();
=======
            writeln!(out, "{}{} = {};", indent, target, src_expr).unwrap();
>>>>>>> 1660d98 (Add file I/O support (OPEN, READ, WRITE, CLOSE) for COBOL to Python; fix UTF-8 by using ASCII bytes)
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
<<<<<<< HEAD
            writeln!(out, "{}{}()", indent, name).unwrap();
=======
            writeln!(out, "{}{}();", indent, name).unwrap();
>>>>>>> 1660d98 (Add file I/O support (OPEN, READ, WRITE, CLOSE) for COBOL to Python; fix UTF-8 by using ASCII bytes)
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
<<<<<<< HEAD
                Literal::String(s) => format!("\"{}\"", s),
            };
            writeln!(out, "{}println({})", indent, expr).unwrap();
        }
        Statement::Evaluate { subject, also_subject, when_clauses } => {
            writeln!(out, "{}match {}:", indent, subject).unwrap();
            for when in when_clauses {
                let cond_str = match &when.condition {
                    WhenCondition::Literal(lit) => match lit {
                        Literal::Int(i) => i.to_string(),
                        Literal::String(s) => format!("\"{}\"", s),
                    },
                    WhenCondition::Variable(v) => v.clone(),
                };
                writeln!(out, "{}    case {}:", indent, cond_str).unwrap();
                for stmt in &when.body {
                    translate_statement(stmt, out, &format!("{}        ", indent));
                }
            }
            if let Some(also) = also_subject {
                writeln!(out, "{}    // also subject {} not supported", indent, also).unwrap();
            }
=======
                Literal::String(s) => s.clone(),
            };
            writeln!(out, "{}println!(\"{}\", {});", indent, expr, expr).unwrap();
>>>>>>> 1660d98 (Add file I/O support (OPEN, READ, WRITE, CLOSE) for COBOL to Python; fix UTF-8 by using ASCII bytes)
        }
        _ => {}
    }
}
<<<<<<< HEAD


=======
>>>>>>> 1660d98 (Add file I/O support (OPEN, READ, WRITE, CLOSE) for COBOL to Python; fix UTF-8 by using ASCII bytes)
