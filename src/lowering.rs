use crate::ast::*;
use crate::ir::*;

pub fn lower_program(
    ast: &[AstStatement],
) -> Vec<Statement> {

    ast.iter()
        .map(lower_statement)
        .collect()
}

pub fn lower_statement(
    stmt: &AstStatement,
) -> Statement {

    match stmt {

        AstStatement::Display { value } => {

            Statement::Display {
                value: Literal::String(value.clone()),
            }
        }

        AstStatement::Move {
            source,
            target,
        } => {

            Statement::Move {
                source: Source::Variable(source.clone()),
                target: target.clone(),
            }
        }

        AstStatement::Add {
            value,
            target,
        } => {

            Statement::Add {
                value: value.parse::<i64>().unwrap_or(0),
                target: target.clone(),
            }
        }

        AstStatement::If {
            condition,
            then_branch,
            else_branch,
        } => {

            Statement::If {

                condition: Condition {
                    left: condition.left.clone(),
                    operator: condition.operator.clone(),
                    right: condition.right.clone(),
                },

                then_branch: then_branch
                    .iter()
                    .map(lower_statement)
                    .collect(),

                else_branch: else_branch.as_ref().map(|b| {
                    b.iter()
                        .map(lower_statement)
                        .collect()
                }),
            }
        }

        AstStatement::PerformUntil {
            condition,
            body,
        } => {

            Statement::PerformUntil {

                condition: Condition {
                    left: condition.left.clone(),
                    operator: condition.operator.clone(),
                    right: condition.right.clone(),
                },

                body: body
                    .iter()
                    .map(lower_statement)
                    .collect(),
            }
        }

        _ => Statement::NoOp,
    }
}

