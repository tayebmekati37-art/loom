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

        cfg.blocks.push(BasicBlock {
            id: 0,

            statements: program.statements.clone(),

            successors: Vec::new(),
        });

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
