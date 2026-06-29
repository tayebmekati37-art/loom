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
                value: Expression::Variable(value.clone()),
            }
        }

        AstStatement::Move {
            source,
            target,
        } => {

            Statement::Assign {
                target: target.clone(),
                value: Expression::Variable(source.clone()),
            }
        }

        AstStatement::Add {
            value,
            target,
        } => {

            Statement::Assign {
                target: target.clone(),

                value: Expression::Binary {
                    left: Box::new(
                        Expression::Variable(target.clone())
                    ),

                    operator: "+".to_string(),

                    right: Box::new(
                        Expression::Variable(value.clone())
                    ),
                },
            }
        }

        _ => Statement::NoOp,
    }
}
