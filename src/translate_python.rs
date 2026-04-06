use crate::ir::{Function, Statement, Source, Literal, Condition, WhenClause, WhenCondition};
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
            writeln!(out, "{}{} = {} + {};", indent, target, target, value).unwrap();
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
    }
            _ => {}
    
}








