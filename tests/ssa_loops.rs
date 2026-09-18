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
                body: vec![Statement::Compute {
                    target: "SUM".to_string(),
                    expr: add_expr("SUM", 1),
                }],
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

    let has_for = program
        .statements
        .iter()
        .any(|statement| matches!(statement, Statement::For { .. }));

    assert!(has_for, "SSA conversion lost the loop structure");
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
                body: vec![Statement::Compute {
                    target: "COUNT".to_string(),
                    expr: add_expr("COUNT", 1),
                }],
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

    assert!(has_back_edge, "expected a real loop back-edge in the CFG");
}

#[test]
fn loop_ssa_inserts_phi_at_loop_header() {
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
                body: vec![Statement::Compute {
                    target: "COUNT".to_string(),
                    expr: add_expr("COUNT", 1),
                }],
            },
        ],
    };

    let cfg = loom::cfg::ControlFlowGraph::build(&program);
    let result = loom::ssa::insert_phi_nodes(&program, &cfg);

    let debug = format!("{:#?}", result);

    assert!(
        debug.contains("Phi"),
        "Expected a Phi node for loop-carried COUNT. Program:\n{}",
        debug
    );

    let phi_index = result
        .statements
        .iter()
        .position(
            |statement| matches!(statement, Statement::Phi { variable, .. } if variable == "COUNT"),
        )
        .expect("Expected COUNT Phi node");

    let for_index = result
        .statements
        .iter()
        .position(|statement| matches!(statement, Statement::For { .. }))
        .expect("Expected For statement");

    assert!(
        phi_index < for_index,
        "Expected COUNT Phi before the loop header For statement. Program:\n{}",
        debug
    );
}

#[test]
fn loop_ssa_renames_loop_body_with_dominator_state() {
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
                body: vec![Statement::Compute {
                    target: "COUNT".to_string(),
                    expr: add_expr("COUNT", 1),
                }],
            },
        ],
    };

    convert_to_ssa(&mut program);

    let debug = format!("{:#?}", program);

    assert!(
        debug.contains("Phi"),
        "Expected a Phi node for loop-carried COUNT. Program:\n{}",
        debug
    );

    assert!(
        debug.contains("COUNT_0"),
        "Expected an initial COUNT_0 definition. Program:\n{}",
        debug
    );

    assert!(
        debug.contains("COUNT_1"),
        "Expected the loop-body COUNT definition to become COUNT_1. Program:\n{}",
        debug
    );

    assert!(
        debug.contains("I_"),
        "Expected the loop variable to be versioned. Program:\n{}",
        debug
    );
}

#[test]
fn v414_loop_phi_has_incoming_values() {
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
                body: vec![Statement::Compute {
                    target: "COUNT".to_string(),
                    expr: add_expr("COUNT", 1),
                }],
            },
        ],
    };

    convert_to_ssa(&mut program);

    let phi = program
        .statements
        .iter()
        .find_map(|statement| {
            if let Statement::Phi {
                variable,
                incoming,
            } = statement
            {
                Some((variable, incoming))
            } else {
                None
            }
        })
        .expect("Expected a Phi node");

    let (variable, incoming) = phi;

    assert_eq!(variable, "COUNT_1");

    assert!(
        !incoming.is_empty(),
        "Expected Phi {} to contain incoming values. Program:\n{:#?}",
        variable,
        program
    );

    for (predecessor, version) in incoming {
        assert!(
            *predecessor < program.statements.len(),
            "Phi predecessor {} looks invalid. Program:\n{:#?}",
            predecessor,
            program
        );

        assert!(
            version.starts_with("COUNT_"),
            "Expected COUNT SSA version in Phi incoming value, got {}",
            version
        );
    }
}

#[test]
fn v415_phi_incomings_match_cfg_predecessors() {
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
                body: vec![Statement::Compute {
                    target: "COUNT".to_string(),
                    expr: add_expr("COUNT", 1),
                }],
            },
        ],
    };

    let cfg = loom::cfg::ControlFlowGraph::build(&program);

    convert_to_ssa(&mut program);

    let phi = program
        .statements
        .iter()
        .find_map(|statement| {
            if let Statement::Phi {
                variable,
                incoming,
            } = statement
            {
                Some((variable, incoming))
            } else {
                None
            }
        })
        .expect("Expected COUNT Phi node");

    let (variable, incoming) = phi;

    assert_eq!(variable, "COUNT_1");
    assert!(!incoming.is_empty());

    // Every Phi incoming must identify a real CFG predecessor
    // of some CFG block.
    for (predecessor, version) in incoming {
        assert!(
            *predecessor < cfg.blocks.len(),
            "Phi {} references nonexistent CFG block {}. CFG has {} blocks.",
            variable,
            predecessor,
            cfg.blocks.len()
        );

        assert!(
            version.starts_with("COUNT_"),
            "Phi {} has unexpected incoming version {} from block {}.",
            variable,
            version,
            predecessor
        );
    }

    // A loop must contain a back-edge. The Phi should therefore
    // have at least one incoming value from a block participating
    // in the loop CFG.
    let has_back_edge = cfg.blocks.iter().any(|block| {
        block
            .successors
            .iter()
            .any(|successor| *successor <= block.id)
    });

    assert!(
        has_back_edge,
        "Expected loop CFG to contain a back-edge. CFG:\n{:#?}",
        cfg
    );
}
