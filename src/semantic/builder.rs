use crate::ir::*;
use crate::semantic::symbol_table::SymbolTable;

/// Build a symbol table from the parsed IR.
///
/// This is the central semantic pass that collects all declared symbols.
/// Later it will also resolve:
/// - 88-level condition names
/// - REDEFINES
/// - OCCURS
/// - SQL DECLARE SECTION variables
/// - COPYBOOK origins
pub fn build_symbol_table(program: &[Statement]) -> SymbolTable {
    let mut table = SymbolTable::new();

    for stmt in program {
        collect_symbols(stmt, &mut table);
    }

    table
}

fn collect_symbols(stmt: &Statement, table: &mut SymbolTable) {
    match stmt {

        Statement::VariableDefinition(var) => {
            table.insert(var);
        }

        Statement::If {
            then_branch,
            else_branch,
            ..
        } => {

            for s in then_branch {
                collect_symbols(s, table);
            }

            if let Some(branch) = else_branch {
                for s in branch {
                    collect_symbols(s, table);
                }
            }
        }

        Statement::Perform { body, .. } => {
            for s in body {
                collect_symbols(s, table);
            }
        }

        Statement::PerformUntil { body, .. } => {
            for s in body {
                collect_symbols(s, table);
            }
        }

        Statement::PerformVarying { body, .. } => {
            for s in body {
                collect_symbols(s, table);
            }
        }

        Statement::Evaluate { branches, .. } => {
            for (_, body) in branches {
                for s in body {
                    collect_symbols(s, table);
                }
            }
        }

        _ => {}
    }
}
