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

    for block in &cfg.blocks {
        println!("Analyzing block {}", block.id);
    }

    result
}
