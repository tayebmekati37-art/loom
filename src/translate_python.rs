<<<<<<< HEAD
﻿use crate::ir::{Function, Statement, Source, Literal, Condition, WhenClause, WhenCondition};
=======
﻿use crate::ir::{Function, Statement, Source, Literal, Condition};
>>>>>>> 902dbcf1dd9dcf086aff99c41645f8732529de4b
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
<<<<<<< HEAD
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
=======
            let src_expr = source_to_expression(value);
            writeln!(out, "{}{} = {} + {}", indent, target, target, src_expr).unwrap();
        }
        Statement::Move { source, target } => {
            let src_expr = source_to_expression(source);
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
>>>>>>> 902dbcf1dd9dcf086aff99c41645f8732529de4b
                }
            }
            if let Some(also) = also_subject {
                writeln!(out, "{}    # also subject {} not supported", indent, also).unwrap();
            }
        }
    }
<<<<<<< HEAD
            _ => {}
    
}








=======
}
fn source_to_expression(src: &Source) -> String {
    match src {
        Source::Literal(i) => i.to_string(),
        Source::Variable(v) => v.clone(),
    }
}
>>>>>>> 902dbcf1dd9dcf086aff99c41645f8732529de4b
