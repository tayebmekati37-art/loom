use crate::ir::*;
use crate::semantic::symbol_table::*;

/// Performs semantic analysis using the SymbolTable.
///
/// Current responsibilities:
/// - Verify referenced variables exist
/// - Walk nested statements
///
/// Future responsibilities:
/// - Resolve 88-level condition names
/// - Resolve REDEFINES aliases
/// - Resolve OCCURS indexing
/// - Resolve SQL host variables
/// - Constant propagation
/// - Type checking
pub fn resolve(program: &[Statement], table: &SymbolTable) -> Result<(), String> {

    for stmt in program {
        resolve_stmt(stmt, table)?;
    }

    Ok(())
}

fn resolve_stmt(stmt: &Statement, table: &SymbolTable) -> Result<(), String> {

    match stmt {

        Statement::Move { source, target } => {

            if table.lookup(target).is_none() {
                return Err(format!("Unknown variable '{}'", target));
            }

            if let Source::Variable(v) = source {
                if table.lookup(v).is_none() {
                    return Err(format!("Unknown variable '{}'", v));
                }
            }
        }

        Statement::Add { target, .. } => {

            if table.lookup(target).is_none() {
                return Err(format!("Unknown variable '{}'", target));
            }
        }

        Statement::Compute { target, .. } => {

            if table.lookup(target).is_none() {
                return Err(format!("Unknown variable '{}'", target));
            }
        }

        Statement::If {
            then_branch,
            else_branch,
            ..
        } => {

            for s in then_branch {
                resolve_stmt(s, table)?;
            }

            if let Some(branch) = else_branch {
                for s in branch {
                    resolve_stmt(s, table)?;
                }
            }
        }

        Statement::Perform { body, .. }
        | Statement::PerformUntil { body, .. }
        | Statement::PerformVarying { body, .. } => {

            for s in body {
                resolve_stmt(s, table)?;
            }
        }

        _ => {}
    }

    Ok(())
}
