use crate::ir::*;
use std::collections::HashMap;

pub fn optimize(program: &mut Program) {
    constant_folding(program);
    constant_propagation(program);
    copy_propagation(program);
    dead_code(program);
}

fn constant_folding(program: &mut Program) {
    for para in &mut program.paragraphs {
        fold_block(&mut para.statements);
    }

    fold_block(&mut program.statements);
}

fn fold_block(block: &mut Vec<Statement>) {
    for stmt in block.iter_mut() {
        match stmt {
            Statement::Compute { target: _, expr } => {
                if let Expression::Binary {
                    left,
                    operator,
                    right,
                } = expr.clone()
                {
                    if let (
                        Expression::Literal(Literal::Int(a)),
                        Expression::Literal(Literal::Int(b)),
                    ) = (*left, *right)
                    {
                        let value = match operator.as_str() {
                            "+" => a + b,
                            "-" => a - b,
                            "*" => a * b,
                            "/" if b != 0 => a / b,
                            _ => continue,
                        };

                        *expr = Expression::Literal(Literal::Int(value));
                    }
                }
            }

            _ => {}
        }
    }
}

fn dead_code(program: &mut Program) {
    for para in &mut program.paragraphs {
        para.statements.retain(|s| !matches!(s, Statement::NoOp));
    }

    program.statements.retain(|s| !matches!(s, Statement::NoOp));
}

fn constant_propagation(program: &mut Program) {
    let mut constants: HashMap<String, Literal> = HashMap::new();

    for para in &mut program.paragraphs {
        propagate_block(&mut para.statements, &mut constants);
    }

    propagate_block(&mut program.statements, &mut constants);
}

fn propagate_block(block: &mut Vec<Statement>, constants: &mut HashMap<String, Literal>) {
    for stmt in block.iter_mut() {
        match stmt {
            Statement::Move { source, target } => match source {
                Source::Literal(i) => {
                    constants.insert(target.clone(), Literal::Int(*i));
                }

                Source::LiteralString(s) => {
                    constants.insert(target.clone(), Literal::String(s.clone()));
                }

                Source::Variable(v) => {
                    if let Some(value) = constants.get(v).cloned() {
                        match value {
                            Literal::Int(i) => {
                                *source = Source::Literal(i);
                            }

                            Literal::String(ref s) => {
                                *source = Source::LiteralString(s.clone());
                            }
                        }

                        constants.insert(target.clone(), value);
                    } else {
                        constants.remove(target);
                    }
                }
            },

            _ => {}
        }
    }
}

fn copy_propagation(program: &mut Program) {
    let mut aliases: HashMap<String, String> = HashMap::new();

    for para in &mut program.paragraphs {
        propagate_copies(&mut para.statements, &mut aliases);
    }

    propagate_copies(&mut program.statements, &mut aliases);
}

fn propagate_copies(block: &mut Vec<Statement>, aliases: &mut HashMap<String, String>) {
    for stmt in block.iter_mut() {
        match stmt {
            Statement::Move { source, target } => match source {
                Source::Variable(v) => {
                    if let Some(real) = aliases.get(v).cloned() {
                        *v = real.clone();
                    }

                    aliases.insert(target.clone(), v.clone());
                }

                _ => {
                    aliases.remove(target);
                }
            },

            Statement::Compute { target, .. }
            | Statement::Add { target, .. }
            | Statement::Subtract { target, .. }
            | Statement::Multiply { target, .. }
            | Statement::Divide { target, .. } => {
                aliases.remove(target);
            }

            _ => {}
        }
    }
}
