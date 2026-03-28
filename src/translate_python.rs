use crate::ir::{Function, Statement, Source};
use std::fmt::Write;

pub fn translate(function: &Function) -> String {
    let mut out = String::new();
    writeln!(out, "def {}():", function.name).unwrap();
    for stmt in &function.body {
        match stmt {
            Statement::Add { target, value } => {
                writeln!(out, "    {} = {} + {}", target, target, value).unwrap();
            }
            Statement::Move { source, target } => {
                let src_expr = match source {
                    Source::Literal(v) => v.to_string(),
                    Source::Variable(v) => v.clone(),
                };
                writeln!(out, "    {} = {}", target, src_expr).unwrap();
            }
        }
    }
    out
}