use crate::ir::*;

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: usize,

    pub statements: Vec<Statement>,

    pub successors: Vec<usize>,

    pub idom: Option<usize>,

    pub dom_children: Vec<usize>,

    pub dominance_frontier: Vec<usize>,
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
            cfg.blocks.push(BasicBlock {
                id,
                statements: block,
                successors: Vec::new(),

                idom: None,

                dom_children: Vec::new(),

                dominance_frontier: Vec::new(),
            });
        }

        rebuild_successors(&mut cfg);

        let dom = compute_dominators(&cfg);

        let idom = compute_immediate_dominators(&cfg, &dom);

        for i in 0..cfg.blocks.len() {
            cfg.blocks[i].idom = idom[i];
        }

        cfg
    }

    pub fn print(&self) {
        println!("CFG");

        for block in &self.blocks {
            println!("Block {} ({} statements)", block.id, block.statements.len());

            println!("Successors: {:?}", block.successors);

            println!("idom({}) = {:?}", block.id, block.idom);

            println!("Dominator Children: {:?}", block.dom_children);

            println!("Dominance Frontier: {:?}", block.dominance_frontier);
        }
    }
}

fn rebuild_successors(cfg: &mut ControlFlowGraph) {
    let block_count = cfg.blocks.len();

    for block_id in 0..block_count {
        cfg.blocks[block_id].successors.clear();

        let is_terminal = cfg.blocks[block_id]
            .statements
            .last()
            .map(|statement| matches!(statement, Statement::StopRun))
            .unwrap_or(false);

        if is_terminal {
            continue;
        }

        if block_id + 1 < block_count {
            cfg.blocks[block_id].successors.push(block_id + 1);
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

pub fn compute_immediate_dominators(
    cfg: &ControlFlowGraph,
    dom: &[std::collections::HashSet<usize>],
) -> Vec<Option<usize>> {
    let count = cfg.blocks.len();

    let mut idom = vec![None; count];

    if count == 0 {
        return idom;
    }

    // Entry block has no immediate dominator.
    idom[0] = None;

    for block in 1..count {
        let strict_dominators: Vec<usize> =
            dom[block].iter().copied().filter(|d| *d != block).collect();

        let mut immediate = None;

        for candidate in &strict_dominators {
            let mut dominated_by_another = false;

            for other in &strict_dominators {
                if candidate == other {
                    continue;
                }

                if dom[*other].contains(candidate) {
                    dominated_by_another = true;
                    break;
                }
            }

            if !dominated_by_another {
                immediate = Some(*candidate);
                break;
            }
        }

        idom[block] = immediate;
    }

    idom
}

pub fn build_dominator_tree(cfg: &mut ControlFlowGraph) {
    for block in &mut cfg.blocks {
        block.dom_children.clear();
    }

    let count = cfg.blocks.len();

    for child in 1..count {
        if let Some(parent) = cfg.blocks[child].idom {
            cfg.blocks[parent].dom_children.push(child);
        }
    }
}

pub fn compute_dominance_frontier(cfg: &mut ControlFlowGraph) {
    for block in &mut cfg.blocks {
        block.dominance_frontier.clear();
    }

    if cfg.blocks.is_empty() {
        return;
    }

    let count = cfg.blocks.len();

    for block in 0..count {
        let predecessors = predecessors(cfg, block);

        // A block with two or more predecessors is a join point.
        if predecessors.len() >= 2 {
            for predecessor in predecessors {
                let mut runner = predecessor;

                while runner != block {
                    if !cfg.blocks[runner].dominance_frontier.contains(&block) {
                        cfg.blocks[runner].dominance_frontier.push(block);
                    }

                    match cfg.blocks[runner].idom {
                        Some(parent) => runner = parent,
                        None => break,
                    }
                }
            }
        }
    }

    for block in &mut cfg.blocks {
        block.dominance_frontier.sort_unstable();
    }
}

fn compute_df_recursive(cfg: &mut ControlFlowGraph, node: usize) {
    let children = cfg.blocks[node].dom_children.clone();

    for child in children {
        compute_df_recursive(cfg, child);
    }
}

#[cfg(test)]
mod dominance_tests {

    use super::*;

    #[test]
    fn test_dominance_frontier_branch_join() {
        let mut cfg = ControlFlowGraph {
            blocks: vec![
                BasicBlock {
                    id: 0,
                    statements: Vec::new(),
                    successors: vec![1, 2],
                    idom: None,
                    dom_children: Vec::new(),
                    dominance_frontier: Vec::new(),
                },
                BasicBlock {
                    id: 1,
                    statements: Vec::new(),
                    successors: vec![3],
                    idom: None,
                    dom_children: Vec::new(),
                    dominance_frontier: Vec::new(),
                },
                BasicBlock {
                    id: 2,
                    statements: Vec::new(),
                    successors: vec![3],
                    idom: None,
                    dom_children: Vec::new(),
                    dominance_frontier: Vec::new(),
                },
                BasicBlock {
                    id: 3,
                    statements: Vec::new(),
                    successors: Vec::new(),
                    idom: None,
                    dom_children: Vec::new(),
                    dominance_frontier: Vec::new(),
                },
            ],
        };

        let dom = compute_dominators(&cfg);

        let idom = compute_immediate_dominators(&cfg, &dom);

        for i in 0..cfg.blocks.len() {
            cfg.blocks[i].idom = idom[i];
        }

        build_dominator_tree(&mut cfg);

        compute_dominance_frontier(&mut cfg);

        assert_eq!(cfg.blocks[0].idom, None);

        assert_eq!(cfg.blocks[1].idom, Some(0));

        assert_eq!(cfg.blocks[2].idom, Some(0));

        assert_eq!(cfg.blocks[3].idom, Some(0));

        assert_eq!(cfg.blocks[1].dominance_frontier, vec![3]);

        assert_eq!(cfg.blocks[2].dominance_frontier, vec![3]);
    }
}

