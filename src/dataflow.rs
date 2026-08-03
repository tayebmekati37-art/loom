use crate::cfg::*;
use std::collections::HashSet;

#[derive(Debug)]
pub struct DataFlowResult {
    pub live_in: Vec<HashSet<String>>,

    pub live_out: Vec<HashSet<String>>,
}

impl DataFlowResult {
    pub fn new(blocks: usize) -> Self {
        Self {
            live_in: vec![HashSet::new(); blocks],

            live_out: vec![HashSet::new(); blocks],
        }
    }
}

pub fn analyze(cfg: &ControlFlowGraph) -> DataFlowResult {
    let mut result = DataFlowResult::new(cfg.blocks.len());

    for block in cfg.blocks.iter().rev() {
        let (uses, defs) = compute_use_def(block);

        result.live_in[block.id] = uses.clone();

        result.live_out[block.id] = defs.clone();
    }

    result
}

fn compute_use_def(block: &BasicBlock) -> (HashSet<String>, HashSet<String>) {
    let mut uses = HashSet::new();

    let mut defs = HashSet::new();

    for stmt in &block.statements {
        match stmt {
            crate::ir::Statement::Move { source, target } => {
                if let crate::ir::Source::Variable(v) = source {
                    if !defs.contains(v) {
                        uses.insert(v.clone());
                    }
                }

                defs.insert(target.clone());
            }

            crate::ir::Statement::Compute { target, .. } => {
                defs.insert(target.clone());
            }

            crate::ir::Statement::Add { target, .. } => {
                defs.insert(target.clone());
            }

            crate::ir::Statement::Subtract { target, .. } => {
                defs.insert(target.clone());
            }

            crate::ir::Statement::Multiply { target, .. } => {
                defs.insert(target.clone());
            }

            crate::ir::Statement::Divide { target, .. } => {
                defs.insert(target.clone());
            }

            _ => {}
        }
    }

    (uses, defs)
}
