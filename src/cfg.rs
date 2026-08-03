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

        let block_count = blocks.len();

        for (id, block) in blocks.into_iter().enumerate() {
            let mut successors = Vec::new();

            if id + 1 < block_count {
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

use std::collections::HashSet;

pub fn compute_dominators(cfg: &ControlFlowGraph) -> Vec<HashSet<usize>> {
    let n = cfg.blocks.len();

    let mut dom = vec![HashSet::new(); n];

    for i in 0..n {
        if i == 0 {
            dom[i].insert(0);
        } else {
            for j in 0..n {
                dom[i].insert(j);
            }
        }
    }

    let mut changed = true;

    while changed {
        changed = false;

        for b in 1..n {
            let preds: Vec<usize> = predecessors(cfg, b);

            if preds.is_empty() {
                continue;
            }

            let mut new_dom = dom[preds[0]].clone();

            for p in preds.iter().skip(1) {
                new_dom = new_dom.intersection(&dom[*p]).cloned().collect();
            }

            new_dom.insert(b);

            if new_dom != dom[b] {
                dom[b] = new_dom;

                changed = true;
            }
        }
    }

    dom
}

fn predecessors(cfg: &ControlFlowGraph, block: usize) -> Vec<usize> {
    let mut preds = Vec::new();

    for b in &cfg.blocks {
        if b.successors.contains(&block) {
            preds.push(b.id);
        }
    }

    preds
}
