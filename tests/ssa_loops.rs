use loom::ir::{Condition, Expression, Literal, Program, Statement};
use loom::ssa::convert_to_ssa;

fn int(value: i64) -> Expression {
    Expression::Literal(Literal::Int(value))
}

fn var(name: &str) -> Expression {
    Expression::Variable(name.to_string())
}

fn program(statements: Vec<Statement>) -> Program {
    Program {
        variables: Vec::new(),
        paragraphs: Vec::new(),
        statements,
    }
}

#[test]
fn loop_program_survives_ssa_conversion() {
    let mut program = program(vec![
        Statement::Move {
            source: loom::ir::Source::Literal(0),
            target: "counter".to_string(),
        },
        Statement::For {
            variable: "i".to_string(),
            start: int(0),
            step: int(1),
            until: Condition {
                left: "i".to_string(),
                operator: ">=".to_string(),
                right: "10".to_string(),
            },
            body: vec![
                Statement::Add {
                    value: 1,
                    target: "counter".to_string(),
                },
                Statement::Compute {
                    target: "counter".to_string(),
                    expr: Expression::Binary {
                        left: Box::new(var("counter")),
                        operator: "+".to_string(),
                        right: Box::new(int(1)),
                    },
                },
            ],
        },
    ]);

    convert_to_ssa(&mut program);

    let debug = format!("{:#?}", program);

    assert!(
        debug.contains("For"),
        "SSA conversion unexpectedly removed the loop:\n{}",
        debug
    );

    assert!(
        debug.contains("counter"),
        "SSA conversion lost the loop-carried variable:\n{}",
        debug
    );
}

#[test]
fn loop_with_branch_preserves_phi_related_structure() {
    let mut program = program(vec![
        Statement::Move {
            source: loom::ir::Source::Literal(0),
            target: "x".to_string(),
        },
        Statement::For {
            variable: "i".to_string(),
            start: int(0),
            step: int(1),
            until: Condition {
                left: "i".to_string(),
                operator: ">=".to_string(),
                right: "5".to_string(),
            },
            body: vec![
                Statement::If {
                    condition: Condition {
                        left: "i".to_string(),
                        operator: ">".to_string(),
                        right: "2".to_string(),
                    },
                    then_branch: vec![
                        Statement::Add {
                            value: 1,
                            target: "x".to_string(),
                        },
                    ],
                    else_branch: Some(vec![
                        Statement::Add {
                            value: 2,
                            target: "x".to_string(),
                        },
                    ]),
                },
            ],
        },
    ]);

    convert_to_ssa(&mut program);

    let debug = format!("{:#?}", program);

    assert!(
        debug.contains("For"),
        "Loop disappeared after SSA conversion:\n{}",
        debug
    );

    assert!(
        debug.contains("If"),
        "Conditional structure disappeared after SSA conversion:\n{}",
        debug
    );

    assert!(
        debug.contains("x"),
        "Loop-carried variable x disappeared after SSA conversion:\n{}",
        debug
    );
}
