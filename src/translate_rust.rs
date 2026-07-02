use crate::ir::*;
use std::fmt::Write;

pub fn translate_program(program: &Program) -> String {
    let mut out = String::new();

    writeln!(out, "fn main() {{").unwrap();

    for stmt in &program.statements {
        translate_statement(stmt, 1, &mut out);
    }

    writeln!(out, "}}").unwrap();

    out
}

fn indent(level: usize) -> String {
    "    ".repeat(level)
}

fn translate_statement(
    stmt: &Statement,
    level: usize,
    out: &mut String,
) {
    let pad = indent(level);

    match stmt {

        Statement::NoOp => {}

        Statement::Display { value } => {
            match value {
                Literal::Int(i) => {
                    writeln!(out, "{}println!(\"{{}}\", {});", pad, i).unwrap();
                }

                Literal::String(s) => {
                    writeln!(out, "{}println!(\"{}\");", pad, s).unwrap();
                }
            }
        }

        Statement::Move { target, .. } => {
            writeln!(out, "{}// MOVE -> {}", pad, target).unwrap();
        }

        Statement::Add { target, value } => {
            writeln!(out, "{}{} += {};", pad, target, value).unwrap();
        }

        Statement::Compute { target, expr } => {
            writeln!(
                out,
                "{}// COMPUTE {} = {:?}",
                pad,
                target,
                expr
            ).unwrap();
        }

        Statement::If {
            condition,
            then_branch,
            else_branch,
        } => {

            writeln!(
                out,
                "{}if {} {} {} {{",
                pad,
                condition.left,
                condition.operator,
                condition.right
            ).unwrap();

            for stmt in then_branch {
                translate_statement(stmt, level + 1, out);
            }

            writeln!(out, "{}}}", pad).unwrap();

            if let Some(else_branch) = else_branch {

                writeln!(out, "{}else {{", pad).unwrap();

                for stmt in else_branch {
                    translate_statement(stmt, level + 1, out);
                }

                writeln!(out, "{}}}", pad).unwrap();
            }
        }

        Statement::Perform { body, .. } => {
            for stmt in body {
                translate_statement(stmt, level, out);
            }
        }

        Statement::PerformUntil {
            condition,
            body,
        } => {

            writeln!(
                out,
                "{}while !({} {} {}) {{",
                pad,
                condition.left,
                condition.operator,
                condition.right
            ).unwrap();

            for stmt in body {
                translate_statement(stmt, level + 1, out);
            }

            writeln!(out, "{}}}", pad).unwrap();
        }

        Statement::Call { program, .. } => {
            writeln!(
                out,
                "{}// CALL {}",
                pad,
                program
            ).unwrap();
        }

        _ => {
            writeln!(
                out,
                "{}// Unsupported statement",
                pad
            ).unwrap();
        }
    }
}
pub fn translate_function(func: &crate::ir::Function) -> String {
    format!("{:#?}", func)
}




