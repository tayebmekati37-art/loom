
use crate::ir::*;
use anyhow::{bail, Result};
use std::collections::HashSet;

pub fn validate_program(program: &Program) -> Result<()> {
    let mut vars = HashSet::new();

    // =====================================
    // variable uniqueness
    // =====================================

    for var in &program.variables {
        if vars.contains(&var.name) {
            bail!("Duplicate variable definition: {}", var.name);
        }

        vars.insert(var.name.clone());
    }

    // =====================================
    // validate paragraphs
    // =====================================

    for para in &program.paragraphs {
        validate_statements(&para.statements, &vars)?;
    }

    // =====================================
    // validate top-level statements
    // =====================================

    validate_statements(&program.statements, &vars)?;

    Ok(())
}

fn validate_statements(
    statements: &[Statement],
    vars: &HashSet<String>,
) -> Result<()> {
    for stmt in statements {
        match stmt {

            Statement::Move { target, .. } => {
                validate_variable(target, vars)?;
            }

            Statement::Add { target, .. } => {
                validate_variable(target, vars)?;
            }

            Statement::Compute { target, .. } => {
                validate_variable(target, vars)?;
            }

            Statement::If {
                then_branch,
                else_branch,
                ..
            } => {
                validate_statements(then_branch, vars)?;

                if let Some(else_branch) = else_branch {
                    validate_statements(else_branch, vars)?;
                }
            }

            Statement::While { body, .. } => {
                validate_statements(body, vars)?;
            }

            Statement::Perform { body, .. } => {
                validate_statements(body, vars)?;
            }

            Statement::PerformUntil { body, .. } => {
                validate_statements(body, vars)?;
            }

            _ => {}
        }
    }

    Ok(())
}

fn validate_variable(
    name: &str,
    vars: &HashSet<String>,
) -> Result<()> {
    if !vars.contains(name) {
        bail!("Undefined variable: {}", name);
    }

    Ok(())
}
