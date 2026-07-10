use crate::ir::*;

pub fn optimize(program: &mut Program) {
    constant_folding(program);
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
