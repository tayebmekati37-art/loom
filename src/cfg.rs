use crate::ir::*;

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: usize,

    pub statements: Vec<Statement>,

    pub successors: Vec<usize>,
}

#[derive(Debug)]
pub struct ControlFlowGraph {
    pub blocks: Vec<BasicBlock>,
}

impl ControlFlowGraph {
    pub fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    pub fn build(program: &Program) -> Self {
        let mut cfg = Self::new();

        let blocks = split_into_blocks(&program.statements);

        for (id, block) in blocks.into_iter().enumerate() {
            let mut successors = Vec::new();

            if id + 1 < program.statements.len() {
                successors.push(id + 1);
            }

            cfg.blocks.push(BasicBlock {
                id,
                statements: block,
                successors,
            });
        }

        cfg
    }

    pub fn print(&self) {
        println!("CFG");

        for block in &self.blocks {
            println!("Block {} ({} statements)", block.id, block.statements.len());

            println!("Successors: {:?}", block.successors);
        }
    }
}

fn split_into_blocks(statements: &[Statement]) -> Vec<Vec<Statement>> {
    let mut blocks = Vec::new();

    let mut current = Vec::new();

    for stmt in statements {
        current.push(stmt.clone());

        match stmt {
            Statement::If { .. }
            | Statement::Perform { .. }
            | Statement::PerformUntil { .. }
            | Statement::PerformVarying { .. }
            | Statement::Call { .. }
            | Statement::StopRun => {
                blocks.push(current);

                current = Vec::new();
            }

            _ => {}
        }
    }

    if !current.is_empty() {
        blocks.push(current);
    }

    blocks
}
