use crate::ir::{Function, Statement, Source, Literal, Condition};
use std::fmt::Write;

pub fn translate(function: &Function) -> String {
    let mut out = String::new();
    writeln!(out, "proc {}() =", function.name).unwrap();
    if function.body.is_empty() {
        writeln!(out, "  discard").unwrap();
    } else {
        for stmt in &function.body {
            translate_statement(stmt, &mut out, "  ");
        }
    }
    out
fn source_to_expression(src: &Source) -> String {
    match src {
        Source::Literal(i) => i.to_string(),
        Source::Variable(v) => v.clone(),
    }
}

}

fn translate_statement(stmt: &Statement, out: &mut String, indent: &str) {
    match stmt {
        Statement::Add { target, value } => {
            writeln!(out, "{}{} = {} + {}", indent, target, target, value).unwrap();
        }
        Statement::Move { source, target } => {
            let src_expr = match source {
                Source::Literal(i) => i.to_string(),
                Source::Variable(v) => v.clone(),
            };
            writeln!(out, "{}{} = {}", indent, target, src_expr).unwrap();
        }
        Statement::If { condition, then_branch, else_branch } => {
            let cond_str = format!("{} {} {}", condition.left, condition.operator, condition.right);
            writeln!(out, "{}if {}:", indent, cond_str).unwrap();
            for stmt in then_branch {
                translate_statement(stmt, out, &format!("{}  ", indent));
            }
            if let Some(else_branch) = else_branch {
                writeln!(out, "{}else:", indent).unwrap();
                for stmt in else_branch {
                    translate_statement(stmt, out, &format!("{}  ", indent));
                }
            }
        }
        Statement::Perform { name } => {
            writeln!(out, "{}{}()", indent, name).unwrap();
        }
        Statement::While { condition, body } => {
            let cond_str = format!("{} {} {}", condition.left, condition.operator, condition.right);
            writeln!(out, "{}while {}:", indent, cond_str).unwrap();
            for stmt in body {
                translate_statement(stmt, out, &format!("{}  ", indent));
            }
        }
        Statement::Display { value } => {
            let expr = match value {
                Literal::Int(i) => i.to_string(),
                Literal::String(s) => s.clone(),
            };
            writeln!(out, "{}echo {}", indent, expr).unwrap();
        }
    }
fn source_to_expression(src: &Source) -> String {
    match src {
        Source::Literal(i) => i.to_string(),
        Source::Variable(v) => v.clone(),
    }
}

}

