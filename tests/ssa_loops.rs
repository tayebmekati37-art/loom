use loom::ir::{Condition, Expression, Literal, Program, Statement};
use loom::ssa::convert_to_ssa;

fn int_expr(value: i64) -> Expression {
    Expression::Literal(Literal::Int(value))
}

fn variable_expr(name: &str) -> Expression {
    Expression::Variable(name.to_string())
}

fn add_expr(variable: &str, value: i64) -> Expression {
    Expression::Binary {
        left: Box::new(variable_expr(variable)),
        operator: "+".to_string(),
        right: Box::new(int_expr(value)),
    }
}

fn loop_condition(variable: &str, operator: &str, value: i64) -> Condition {
    Condition {
        left: variable.to_string(),
        operator: operator.to_string(),
        right: value.to_string(),
    }
}

fn make_loop_program() -> Program {
    Program {
        variables: Vec::new(),
        paragraphs: Vec::new(),
        statements: vec![
            Statement::Move {
                source: loom::ir::Source::Literal(0),
                target: "I".to_string(),
            },

            Statement::For {
                variable: "I".to_string(),
                start: int_expr(0),
                step: int_expr(1),
                until: loop_condition("I", "<", 10),
                body: vec![
                    Statement::Compute {
                        target: "SUM".to_string(),
                        expr: add_expr("SUM", 1),
                    },
                ],
            },
        ],
    }
}

#[test]
fn loop_ssa_conversion_does_not_panic() {
    let mut program = make_loop_program();

    convert_to_ssa(&mut program);

    assert!(
        !program.statements.is_empty(),
        "SSA conversion unexpectedly removed the program statements"
    );
}

#[test]
fn loop_ssa_preserves_loop_structure() {
    let mut program = make_loop_program();

    convert_to_ssa(&mut program);

    let has_for = program.statements.iter().any(|statement| {
        matches!(statement, Statement::For { .. })
    });

    assert!(
        has_for,
        "SSA conversion lost the loop structure"
    );
}

#[test]
fn loop_ssa_produces_versioned_definitions() {
    let mut program = make_loop_program();

    convert_to_ssa(&mut program);

    let debug = format!("{:#?}", program);

    assert!(
        debug.contains("I_"),
        "Expected SSA conversion to create a versioned I definition. Program:\n{}",
        debug
    );
}

#[test]
fn loop_ssa_handles_loop_carried_computation() {
    let mut program = Program {
        variables: Vec::new(),
        paragraphs: Vec::new(),
        statements: vec![
            Statement::Move {
                source: loom::ir::Source::Literal(0),
                target: "COUNT".to_string(),
            },

            Statement::For {
                variable: "I".to_string(),
                start: int_expr(0),
                step: int_expr(1),
                until: loop_condition("I", "<", 5),
                body: vec![
                    Statement::Compute {
                        target: "COUNT".to_string(),
                        expr: add_expr("COUNT", 1),
                    },
                ],
            },
        ],
    };

    convert_to_ssa(&mut program);

    let debug = format!("{:#?}", program);

    assert!(
        debug.contains("COUNT_"),
        "Expected loop-carried COUNT definition to be versioned. Program:\n{}",
        debug
    );
}

#[test]
fn loop_cfg_has_real_back_edge() {
    let program = make_loop_program();
    let cfg = loom::cfg::ControlFlowGraph::build(&program);

    println!();
    println!("=== LOOP CFG ===");
    cfg.print();

    // A real loop requires at least:
    //   entry/header
    //   body
    //   exit
    //
    // and at least one successor must point backward
    // to an earlier CFG block.
    assert!(
        cfg.blocks.len() >= 3,
        "expected at least 3 CFG blocks for a For loop, got {}",
        cfg.blocks.len()
    );

    let mut has_back_edge = false;

    for block in &cfg.blocks {
        for &successor in &block.successors {
            if successor <= block.id {
                has_back_edge = true;
            }
        }
    }

    assert!(
        has_back_edge,
        "expected a real loop back-edge in the CFG"
    );
}
