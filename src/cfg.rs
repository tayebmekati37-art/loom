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

        // Build derived dominance information after idoms are known.
        build_dominator_tree(&mut cfg);
        compute_dominance_frontier(&mut cfg);

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

    for block in &mut cfg.blocks {
        block.successors.clear();
    }

    let mut block_id = 0;

    while block_id < block_count {
        let last_statement = cfg.blocks[block_id].statements.last();

        if matches!(last_statement, Some(Statement::StopRun)) {
            block_id += 1;
            continue;
        }

        // --------------------------------------------------------
        // FOR loop
        //
        // split_into_blocks() creates:
        //
        //   block_id     = loop header
        //   block_id + 1 = loop body
        //   block_id + 2 = loop exit
        //
        // Therefore:
        //
        //   header -> body
        //   header -> exit
        //   body   -> header
        //   exit   -> normal fall-through
        // --------------------------------------------------------

        if matches!(last_statement, Some(Statement::For { .. }))
            && block_id + 2 < block_count
        {
            let body_block = block_id + 1;
            let exit_block = block_id + 2;

            cfg.blocks[block_id]
                .successors
                .push(body_block);

            cfg.blocks[block_id]
                .successors
                .push(exit_block);

            cfg.blocks[body_block]
                .successors
                .push(block_id);

            block_id += 2;
            continue;
        }

        // --------------------------------------------------------
        // IF
        // --------------------------------------------------------

        let is_if = matches!(last_statement, Some(Statement::If { .. }));

        if is_if && block_id + 3 < block_count {
            let then_block = block_id + 1;
            let else_block = block_id + 2;
            let join_block = block_id + 3;

            cfg.blocks[block_id]
                .successors
                .push(then_block);

            cfg.blocks[block_id]
                .successors
                .push(else_block);

            cfg.blocks[then_block]
                .successors
                .push(join_block);

            cfg.blocks[else_block]
                .successors
                .push(join_block);

            block_id += 3;
            continue;
        }

        // --------------------------------------------------------
        // Ordinary sequential fall-through
        // --------------------------------------------------------

        if block_id + 1 < block_count {
            cfg.blocks[block_id]
                .successors
                .push(block_id + 1);
        }

        block_id += 1;
    }
}

fn split_into_blocks(statements: &[Statement]) -> Vec<Vec<Statement>> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();

    for stmt in statements {
        match stmt {
            Statement::For {
                variable,
                start,
                step,
                until,
                body,
            } => {
                if !current.is_empty() {
                    blocks.push(std::mem::take(&mut current));
                }

                // Loop header.
                //
                // The body is intentionally removed from the header
                // block. The CFG represents the loop structurally as:
                //
                //   header -> body
                //   header -> exit
                //   body   -> header
                //
                // The original body is stored in its own block.
                blocks.push(vec![Statement::For {
                    variable: variable.clone(),
                    start: start.clone(),
                    step: step.clone(),
                    until: until.clone(),
                    body: Vec::new(),
                }]);

                // Loop body.
                //
                // Keep the body together for this first CFG step.
                // Nested structured control flow can be lowered
                // separately once the basic loop shape is correct.
                if body.is_empty() {
                    blocks.push(vec![Statement::NoOp]);
                } else {
                    blocks.push(body.clone());
                }

                // Explicit loop exit block.
                blocks.push(vec![Statement::NoOp]);
            }

            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if !current.is_empty() {
                    blocks.push(std::mem::take(&mut current));
                }

                // Keep the IF condition as its own branch-control block.
                blocks.push(vec![Statement::If {
                    condition: condition.clone(),
                    then_branch: Vec::new(),
                    else_branch: None,
                }]);

                let then_blocks = split_into_blocks(then_branch);
                blocks.extend(then_blocks);

                if let Some(else_statements) = else_branch {
                    let else_blocks = split_into_blocks(else_statements);
                    blocks.extend(else_blocks);
                }
            }

            Statement::Perform { .. }
            | Statement::PerformUntil { .. }
            | Statement::PerformVarying { .. }
            | Statement::Call { .. }
            | Statement::StopRun => {
                current.push(stmt.clone());
                blocks.push(std::mem::take(&mut current));
            }

            _ => {
                current.push(stmt.clone());
            }
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

    // First build the dominator tree from the already-computed idoms.
    build_dominator_tree(cfg);

    // Standard Cytron-style dominance frontier computation:
    //
    // DF_local(B):
    //   successors S of B where idom(S) != B
    //
    // DF_up(B):
    //   dominance frontiers of B's dominator-tree children,
    //   excluding nodes strictly dominated by B.
    //
    // This prevents ancestors such as the entry block from incorrectly
    // receiving a join point in their dominance frontier.

    fn compute_df(cfg: &mut ControlFlowGraph, node: usize) {
        let children = cfg.blocks[node].dom_children.clone();

        // Local contribution.
        let successors = cfg.blocks[node].successors.clone();

        for successor in successors {
            if cfg.blocks[successor].idom != Some(node)
                && !cfg.blocks[node].dominance_frontier.contains(&successor)
            {
                cfg.blocks[node].dominance_frontier.push(successor);
            }
        }

        // Up contribution from dominator-tree children.
        for child in children {
            compute_df(cfg, child);

            let child_frontier = cfg.blocks[child].dominance_frontier.clone();

            for frontier_block in child_frontier {
                if cfg.blocks[frontier_block].idom != Some(node)
                    && !cfg.blocks[node]
                        .dominance_frontier
                        .contains(&frontier_block)
                {
                    cfg.blocks[node].dominance_frontier.push(frontier_block);
                }
            }
        }

        cfg.blocks[node].dominance_frontier.sort_unstable();
    }

    compute_df(cfg, 0);
}

fn compute_df_recursive(cfg: &mut ControlFlowGraph, node: usize) {
    let children = cfg.blocks[node].dom_children.clone();

    for child in children {
        compute_df_recursive(cfg, child);
    }
}

#[cfg(test)]
mod structured_cfg_tests {
    use super::*;
    use crate::ir::{Condition, Literal, Program, Source, Statement};

    #[test]
    fn test_if_creates_branch_and_join() {
        let program = Program {
            variables: Vec::new(),
            paragraphs: Vec::new(),
            statements: vec![
                Statement::Move {
                    source: Source::Literal(1),
                    target: "A".to_string(),
                },
                Statement::If {
                    condition: Condition {
                        left: "A".to_string(),
                        operator: "=".to_string(),
                        right: "1".to_string(),
                    },

                    then_branch: vec![Statement::Move {
                        source: Source::Literal(10),
                        target: "B".to_string(),
                    }],

                    else_branch: Some(vec![Statement::Move {
                        source: Source::Literal(20),
                        target: "B".to_string(),
                    }]),
                },
                Statement::Display {
                    value: Literal::String("DONE".to_string()),
                },
            ],
        };

        let cfg = ControlFlowGraph::build(&program);

        println!("Structured IF CFG:");
        cfg.print();

        // Expected structure:
        //
        // block 0 -> block 1
        // block 1 -> then block + else block
        // then block -> join block
        // else block -> join block
        //
        // Therefore we need at least 5 blocks.

        assert!(
            cfg.blocks.len() >= 5,
            "expected at least 5 blocks for IF/ELSE CFG, got {}",
            cfg.blocks.len()
        );

        let branch_block = &cfg.blocks[1];

        assert_eq!(
            branch_block.successors.len(),
            2,
            "IF block should have two successors"
        );

        let then_block = branch_block.successors[0];
        let else_block = branch_block.successors[1];

        assert_ne!(
            then_block, else_block,
            "then and else branches must be different blocks"
        );

        let then_successors = &cfg.blocks[then_block].successors;
        let else_successors = &cfg.blocks[else_block].successors;

        assert_eq!(
            then_successors.len(),
            1,
            "then branch should flow into the join block"
        );

        assert_eq!(
            else_successors.len(),
            1,
            "else branch should flow into the join block"
        );

        assert_eq!(
            then_successors[0], else_successors[0],
            "then and else branches should converge"
        );
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

