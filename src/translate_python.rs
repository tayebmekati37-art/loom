use crate::ir::{Function, Statement, Source, Literal, Condition, WhenClause, WhenCondition, LiteralOrVariable, StringSource};
use std::fmt::Write;

pub fn translate(function: &Function) -> String {
    let mut out = String::new();
    writeln!(out, "def {}():", function.name).unwrap();
    if function.body.is_empty() {
        writeln!(out, "    pass").unwrap();
    } else {
        for stmt in &function.body {
            translate_statement(stmt, &mut out, "    ");
        }
    }
    out
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
                translate_statement(stmt, out, &format!("{}    ", indent));
            }
            if let Some(else_branch) = else_branch {
                writeln!(out, "{}else:", indent).unwrap();
                for stmt in else_branch {
                    translate_statement(stmt, out, &format!("{}    ", indent));
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
                translate_statement(stmt, out, &format!("{}    ", indent));
            }
        }
        Statement::Display { value } => {
            let expr = match value {
                Literal::Int(i) => i.to_string(),
                Literal::String(s) => format!("'{}'", s),
            };
            writeln!(out, "{indent}print({expr})").unwrap();
        }
        Statement::Evaluate { subject, also_subject, when_clauses } => {
            writeln!(out, "{}match {}:", indent, subject).unwrap();
            for when in when_clauses {
                let cond_str = match &when.condition {
                    WhenCondition::Literal(lit) => match lit {
                        Literal::Int(i) => i.to_string(),
                        Literal::String(s) => format!("'{}'", s),
                    },
                    WhenCondition::Variable(v) => v.clone(),
                };
                writeln!(out, "{}    case {}:", indent, cond_str).unwrap();
                for stmt in &when.body {
                    translate_statement(stmt, out, &format!("{}        ", indent));
                }
            }
            if let Some(also) = also_subject {
                writeln!(out, "{}    # also subject {} not supported", indent, also).unwrap();
            }
        }
        Statement::String { sources, into, pointer: _ } => {
            let mut parts = Vec::new();
            for src in sources {
                let src_str = match &src.source {
                    LiteralOrVariable::Literal(lit) => match lit {
                        Literal::Int(i) => i.to_string(),
                        Literal::String(s) => format!("'{}'", s),
                    },
                    LiteralOrVariable::Variable(v) => v.clone(),
                };
                parts.push(src_str);
            }
            let joined = parts.join(" + ");
            writeln!(out, "{}{} = {}", indent, into, joined).unwrap();
        }
        _ => {}
    }
}