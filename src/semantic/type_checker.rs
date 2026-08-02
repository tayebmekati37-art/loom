use crate::ir::*;

/// Performs semantic type checking.
///
/// Current implementation is a stub.
///
/// Future checks:
/// - MOVE compatibility
/// - COMPUTE expressions
/// - PIC compatibility
/// - Numeric/string conversions
/// - OCCURS indexing
/// - REDEFINES compatibility
/// - SQL host variables
pub fn check(_program: &Vec<Statement>) -> Vec<String> {
    let diagnostics = Vec::new();

    diagnostics
}
