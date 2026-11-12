use crate::data_division::pic::PictureClause;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringOperationSource {
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhenClause {
    pub condition: String,
    pub body: Vec<Statement>,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    Literal(Literal),
    Variable(String),

    Binary {
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub left: String,
    pub operator: String,
    pub right: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    Int(i64),
    String(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Source {
    Literal(i64),
    LiteralString(String),
    Variable(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Phi {
        variable: String,
        incoming: Vec<(usize, String)>,
    },

    Replace {
        from: String,
        to: String,
    },

    Subtract {
        value: String,
        target: String,
    },

    Multiply {
        value: String,
        target: String,
    },

    Divide {
        value: String,
        target: String,
    },

    String {
        sources: Vec<String>,
        into: String,
    },

    Unstring {
        source: String,
        into: Vec<String>,
    },

    For {
        variable: String,
        start: Expression,
        step: Expression,
        until: Condition,
        body: Vec<Statement>,
    },

    NoOp,

    Display {
        value: Literal,
    },

    Move {
        source: Source,
        target: String,
    },

    Add {
        value: i64,
        target: String,
    },

    Initialize {
        variable: String,
    },

    ExecSql {
        sql: String,
    },

    DeclareSection {
        begin: bool,
    },

    Compute {
        target: String,
        expr: Expression,
    },

    If {
        condition: Condition,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },

    Perform {
        name: Option<String>,
        body: Vec<Statement>,
    },

    PerformUntil {
        condition: Condition,
        body: Vec<Statement>,
    },

    PerformVarying {
        variable: String,
        from: Expression,
        by: Expression,
        until: Condition,
        body: Vec<Statement>,
    },

    Call {
        program: String,
        using_args: Vec<String>,
    },

    StopRun,

    Continue,

    Exit,

    Copybook {
        name: String,
        replacing: Vec<(String, String)>,
    },

    OpenFile {
        name: String,
        mode: FileMode,
    },

    ReadFile {
        name: String,
    },

    WriteFile {
        name: String,
    },

    CloseFile {
        name: String,
    },

    ArrayGet {
        array: String,
        index: usize,
        target: String,
    },

    ArraySet {
        array: String,
        index: usize,
        value: String,
    },

    Accept {
        variable: String,
    },

    Evaluate {
        value: String,
        when_clauses: Vec<WhenClause>,
    },

    Redefines {
        name: String,
    },

    Occurs {
        name: String,
        times: usize,
    },

    ConditionName {
        name: String,
        value: String,
    },

    Inspect {
        source: String,
        replacing: String,
        with: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileMode {
    Input,
    Output,
    IO,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PicType {
    Numeric,
    Decimal,
    PackedDecimal,
    Alpha,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompType {
    Comp,
    Comp3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataField {
    pub level: u32,
    pub name: String,
    pub pic: Option<PicClause>,
    pub occurs: Option<u32>,
    pub redefines: Option<String>,
    pub usage: Option<UsageClause>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PicClause {
    pub raw: String,
    pub category: PicCategory,
    pub length: usize,
    pub scale: usize,
    pub signed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PicCategory {
    Numeric,
    AlphaNumeric,
    Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageClause {
    Display,
    Comp,
    Comp3,
    Binary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionName {
    pub name: String,
    pub value: Literal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDefinition {
    pub picture: Option<PictureClause>,
    pub name: String,
    pub pic: Option<PicType>,
    pub occurs: Option<usize>,
    pub redefines: Option<String>,
    pub initial_value: Option<Literal>,
    pub comp_type: Option<CompType>,

    pub ssa_version: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    pub name: String,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub variables: Vec<VariableDefinition>,
    pub paragraphs: Vec<Paragraph>,
    pub statements: Vec<Statement>,
}

// =====================================================================
// IR invariants — structural validation for every Loom Program
// =====================================================================
//
// These checks catch malformed IR early, before SSA conversion or
// translation, so downstream passes fail with clear diagnostics
// rather than cryptic panics.
//
// The checks are deliberately conservative. They reject:
//   - Empty variable / paragraph / statement identifiers
//   - Duplicate variable or paragraph declarations
//   - Empty operator strings in expressions and conditions
//   - Phi nodes with duplicate predecessor blocks
//
// Future milestones will extend this module as the IR grows source
// locations, explicit unsupported constructs, and richer types.
// =====================================================================
pub mod ir_invariants {
    use super::*;
    use std::collections::HashSet;

    /// Validate structural invariants that every Loom `Program` must
    /// satisfy before it enters later pipeline stages (SSA, optimization,
    /// translation).
    ///
    /// Returns `Ok(())` when all invariants hold, or `Err(Vec<String>)`
    /// listing every violation found. The error list is not truncated;
    /// callers may print or log all entries.
    pub fn validate_ir(program: &Program) -> Result<(), Vec<String>> {
        let mut errors: Vec<String> = Vec::new();

        // 1. Variable declarations: non-empty, unique names.
        let mut seen_vars: HashSet<&str> = HashSet::new();
        for (i, v) in program.variables.iter().enumerate() {
            if v.name.is_empty() {
                errors.push(format!("variables[{}]: empty name", i));
            } else if !seen_vars.insert(v.name.as_str()) {
                errors.push(format!("variables[{}]: duplicate name '{}'", i, v.name));
            }
        }

        // 2. Paragraphs: non-empty, unique names.
        let mut seen_paras: HashSet<&str> = HashSet::new();
        for (i, p) in program.paragraphs.iter().enumerate() {
            if p.name.is_empty() {
                errors.push(format!("paragraphs[{}]: empty name", i));
            } else if !seen_paras.insert(p.name.as_str()) {
                errors.push(format!("paragraphs[{}]: duplicate name '{}'", i, p.name));
            }
        }

        // 3. Program statements: recursive structural checks.
        check_statements(&program.statements, "statements", &mut errors);

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    /// Walk a statement list and record any invariant violations.
    pub fn check_statements(statements: &[Statement], path: &str, errors: &mut Vec<String>) {
        for (i, s) in statements.iter().enumerate() {
            let at = format!("{}[{}]", path, i);
            match s {
                Statement::Move { source, target } => {
                    if target.is_empty() {
                        errors.push(format!("{}: Move has empty target", at));
                    }
                    if let Source::Variable(name) = source {
                        if name.is_empty() {
                            errors.push(format!("{}: Move has empty source variable", at));
                        }
                    }
                }
                Statement::Compute { target, expr } => {
                    if target.is_empty() {
                        errors.push(format!("{}: Compute has empty target", at));
                    }
                    check_expression(expr, &format!("{}.expr", at), errors);
                }
                Statement::If { condition, then_branch, else_branch } => {
                    check_condition(condition, &format!("{}.condition", at), errors);
                    check_statements(then_branch, &format!("{}.then_branch", at), errors);
                    if let Some(eb) = else_branch {
                        check_statements(eb, &format!("{}.else_branch", at), errors);
                    }
                }
                Statement::For { variable, start, step, until, body } => {
                    if variable.is_empty() {
                        errors.push(format!("{}: For has empty variable", at));
                    }
                    check_expression(start, &format!("{}.start", at), errors);
                    check_expression(step, &format!("{}.step", at), errors);
                    check_condition(until, &format!("{}.until", at), errors);
                    check_statements(body, &format!("{}.body", at), errors);
                }
                Statement::Phi { variable, incoming } => {
                    if variable.is_empty() {
                        errors.push(format!("{}: Phi has empty variable", at));
                    }
                    let mut seen_preds: HashSet<usize> = HashSet::new();
                    for (pred, _) in incoming {
                        if !seen_preds.insert(*pred) {
                            errors.push(format!(
                                "{}: Phi has duplicate predecessor block {}", at, pred
                            ));
                        }
                    }
                }
                Statement::Display { .. } | Statement::NoOp => {}
                _ => {}
            }
        }
    }

    /// Recursively validate an expression tree.
    pub fn check_expression(expr: &Expression, path: &str, errors: &mut Vec<String>) {
        match expr {
            Expression::Variable(name) => {
                if name.is_empty() {
                    errors.push(format!("{}: empty variable name in Expression", path));
                }
            }
            Expression::Binary { left, operator, right } => {
                if operator.is_empty() {
                    errors.push(format!("{}: empty operator in Binary expression", path));
                }
                check_expression(left, &format!("{}.left", path), errors);
                check_expression(right, &format!("{}.right", path), errors);
            }
            Expression::Literal(_) => {}
        }
    }

    /// Validate a Condition struct.
    pub fn check_condition(cond: &Condition, path: &str, errors: &mut Vec<String>) {
        if cond.operator.is_empty() {
            errors.push(format!("{}: empty operator in Condition", path));
        }
    }

    /// Render a `Statement` variant name for logs and diagnostics.
    pub fn describe_statement(stmt: &Statement) -> &'static str {
        match stmt {
            Statement::Phi { .. } => "Phi",
            Statement::Replace { .. } => "Replace",
            Statement::Subtract { .. } => "Subtract",
            Statement::Multiply { .. } => "Multiply",
            Statement::Divide { .. } => "Divide",
            Statement::String { .. } => "String",
            Statement::Unstring { .. } => "Unstring",
            Statement::For { .. } => "For",
            Statement::NoOp => "NoOp",
            Statement::Display { .. } => "Display",
            Statement::Move { .. } => "Move",
            Statement::Add { .. } => "Add",
            Statement::Initialize { .. } => "Initialize",
            Statement::ExecSql { .. } => "ExecSql",
            Statement::DeclareSection { .. } => "DeclareSection",
            Statement::Compute { .. } => "Compute",
            Statement::If { .. } => "If",
            Statement::Perform { .. } => "Perform",
            Statement::PerformUntil { .. } => "PerformUntil",
            Statement::PerformVarying { .. } => "PerformVarying",
            Statement::Call { .. } => "Call",
            Statement::StopRun => "StopRun",
            Statement::Continue => "Continue",
            Statement::Exit => "Exit",
            Statement::Copybook { .. } => "Copybook",
            Statement::OpenFile { .. } => "OpenFile",
            Statement::ReadFile { .. } => "ReadFile",
            Statement::WriteFile { .. } => "WriteFile",
            Statement::CloseFile { .. } => "CloseFile",
            Statement::ArrayGet { .. } => "ArrayGet",
            Statement::ArraySet { .. } => "ArraySet",
            Statement::Accept { .. } => "Accept",
            Statement::Evaluate { .. } => "Evaluate",
            Statement::Redefines { .. } => "Redefines",
            Statement::Occurs { .. } => "Occurs",
            Statement::ConditionName { .. } => "ConditionName",
            Statement::Inspect { .. } => "Inspect",
        }
    }
}

// =====================================================================
// IR invariant tests
// =====================================================================
#[cfg(test)]
mod ir_invariant_tests {
    use super::*;
    use super::ir_invariants::validate_ir;

    fn empty_program() -> Program {
        Program {
            variables: Vec::new(),
            paragraphs: Vec::new(),
            statements: Vec::new(),
        }
    }

    fn fresh_var(name: &str) -> VariableDefinition {
        VariableDefinition {
            picture: None,
            name: name.to_string(),
            pic: None,
            occurs: None,
            redefines: None,
            initial_value: None,
            comp_type: None,
            ssa_version: None,
        }
    }

    #[test]
    fn empty_program_is_valid() {
        let p = empty_program();
        assert!(validate_ir(&p).is_ok(), "empty Program should be valid");
    }

    #[test]
    fn duplicate_variable_rejected() {
        let mut p = empty_program();
        p.variables.push(fresh_var("X"));
        p.variables.push(fresh_var("X"));
        let errs = validate_ir(&p).unwrap_err();
        assert!(
            errs.iter().any(|m| m.contains("duplicate name 'X'")),
            "expected duplicate-variable diagnostic, got: {:?}", errs
        );
    }

    #[test]
    fn empty_move_target_rejected() {
        let mut p = empty_program();
        p.statements.push(Statement::Move {
            source: Source::Literal(0),
            target: String::new(),
        });
        let errs = validate_ir(&p).unwrap_err();
        assert!(
            errs.iter().any(|m| m.contains("Move has empty target")),
            "expected empty-target diagnostic, got: {:?}", errs
        );
    }

    #[test]
    fn duplicate_phi_predecessor_rejected() {
        let mut p = empty_program();
        p.statements.push(Statement::Phi {
            variable: "X_1".to_string(),
            incoming: vec![
                (0, "X_0".to_string()),
                (0, "X_0".to_string()),
            ],
        });
        let errs = validate_ir(&p).unwrap_err();
        assert!(
            errs.iter().any(|m| m.contains("duplicate predecessor block 0")),
            "expected duplicate-predecessor diagnostic, got: {:?}", errs
        );
    }

    #[test]
    fn empty_condition_operator_rejected() {
        let mut p = empty_program();
        p.statements.push(Statement::If {
            condition: Condition {
                left: "A".to_string(),
                operator: String::new(),
                right: "1".to_string(),
            },
            then_branch: Vec::new(),
            else_branch: None,
        });
        let errs = validate_ir(&p).unwrap_err();
        assert!(
            errs.iter().any(|m| m.contains("empty operator in Condition")),
            "expected empty-operator diagnostic, got: {:?}", errs
        );
    }
}
