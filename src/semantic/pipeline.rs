use crate::ir::*;
use crate::semantic::builder::*;
use crate::semantic::resolver::*;
use crate::semantic::type_checker::*;
use crate::semantic::validator::*;
use crate::semantic::constant_folder::*;

pub struct SemanticResult {

    pub symbol_table: crate::semantic::symbol_table::SymbolTable,

    pub diagnostics: Vec<String>,
}

pub fn analyze(program: &mut Vec<Statement>) -> SemanticResult {

    let table = build_symbol_table(program);

    let mut diagnostics = Vec::new();

    if let Err(e) = resolve(program, &table) {
        diagnostics.push(e);
    }

    diagnostics.extend(check(program));

    diagnostics.extend(validate(program));

    fold_program(program);

    SemanticResult {

        symbol_table: table,

        diagnostics,
    }
}
