use crate::ir::{Function, Statement, Source, Literal, Condition};
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
            let src_expr = source_to_expression(value);
            writeln!(out, "{}{} = {} + {}", indent, target, target, src_expr).unwrap();
        }
    }
fn source_to_expression(src: &Source) -> String {
    match src {
        Source::Literal(i) => i.to_string(),
        Source::Variable(v) => v.clone(),
    }
}

}
