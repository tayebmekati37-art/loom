use crate::cfg::ControlFlowGraph;
use crate::ir::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct VersionCounter {
    versions: HashMap<String, usize>,
}

impl VersionCounter {
    pub fn next_version(&mut self, variable: &str) -> usize {
        let version = self.versions.entry(variable.to_string()).or_insert(0);

        *version += 1;

        *version
    }
}

pub fn convert_to_ssa(_program: &mut Program) {
    let mut counter = VersionCounter::default();

    println!("SSA placeholder version {}", counter.next_version("TEMP"));
}

pub fn rename_variable(name: &str, version: usize) -> String {
    format!("{}_{}", name, version)
}

pub fn rename_move_targets(program: &mut Program) {
    let mut counter = VersionCounter::default();

    for stmt in &mut program.statements {
        if let Statement::Move { target, .. } = stmt {
            let version = counter.next_version(target);

            *target = rename_variable(target, version);
        }
    }
}

pub fn rename_compute_targets(program: &mut Program) {
    let mut counter = VersionCounter::default();

    for stmt in &mut program.statements {
        if let Statement::Compute { target, .. } = stmt {
            let version = counter.next_version(target);

            *target = rename_variable(target, version);
        }
    }
}

pub fn rename_arithmetic_targets(program: &mut Program, counter: &mut VersionCounter) {
    for stmt in &mut program.statements {
        let target = match stmt {
            Statement::Add { target, .. } => target,

            Statement::Subtract { target, .. } => target,

            Statement::Multiply { target, .. } => target,

            Statement::Divide { target, .. } => target,

            Statement::Initialize { variable } => variable,

            _ => continue,
        };

        let version = counter.next_version(target);

        *target = rename_variable(target, version);
    }
}

fn rename_condition(cond: &mut Condition, latest: &std::collections::HashMap<String, String>) {
    if let Some(name) = latest.get(&cond.left) {
        cond.left = name.clone();
    }

    if let Some(name) = latest.get(&cond.right) {
        cond.right = name.clone();
    }
}

pub fn rename_variable_uses(program: &mut Program) {
    let mut latest: HashMap<String, String> = HashMap::new();

    for stmt in &mut program.statements {
        match stmt {
            Statement::Move { source, target } => {
                if let Source::Variable(name) = source {
                    if let Some(current) = latest.get(name) {
                        *name = current.clone();
                    }
                }

                latest.insert(
                    target.split("_").next().unwrap().to_string(),
                    target.clone(),
                );
            }

            Statement::Compute { expr, target } => {
                rename_expression(expr, &latest);

                latest.insert(
                    target.split("_").next().unwrap().to_string(),
                    target.clone(),
                );
            }

            Statement::If { condition, .. } => {
                rename_condition(condition, &latest);
            }

            Statement::PerformUntil { condition, .. } => {
                rename_condition(condition, &latest);
            }

            Statement::PerformVarying { until, .. } => {
                rename_condition(until, &latest);
            }

            _ => {}
        }
    }
}

fn rename_expression(expr: &mut Expression, latest: &std::collections::HashMap<String, String>) {
    match expr {
        Expression::Variable(name) => {
            if let Some(current) = latest.get(name) {
                *name = current.clone();
            }
        }

        Expression::Binary { left, right, .. } => {
            rename_expression(left, latest);

            rename_expression(right, latest);
        }

        _ => {}
    }
}

#[derive(Debug, Default)]

pub struct UseDefChains {
    pub defs: HashMap<String, Vec<usize>>,

    pub uses: HashMap<String, Vec<usize>>,
}

pub fn build_use_def_chains(program: &Program) -> UseDefChains {
    let mut chains = UseDefChains::default();

    collect_use_def_chains(&program.statements, &mut chains, &mut 0);

    chains
}

fn collect_use_def_chains(statements: &[Statement], chains: &mut UseDefChains, index: &mut usize) {
    for stmt in statements {
        let current_index = *index;
        *index += 1;

        match stmt {
            Statement::Move { source, target } => {
                chains
                    .defs
                    .entry(target.clone())
                    .or_default()
                    .push(current_index);

                if let Source::Variable(name) = source {
                    chains
                        .uses
                        .entry(name.clone())
                        .or_default()
                        .push(current_index);
                }
            }

            Statement::Compute { target, expr } => {
                chains
                    .defs
                    .entry(target.clone())
                    .or_default()
                    .push(current_index);

                collect_expression_uses(expr, current_index, chains);
            }

            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                collect_condition_uses(condition, current_index, chains);

                collect_use_def_chains(then_branch, chains, index);

                if let Some(else_branch) = else_branch {
                    collect_use_def_chains(else_branch, chains, index);
                }
            }

            Statement::Perform { body, .. } => {
                collect_use_def_chains(body, chains, index);
            }

            Statement::PerformUntil { condition, body } => {
                collect_condition_uses(condition, current_index, chains);
                collect_use_def_chains(body, chains, index);
            }

            Statement::PerformVarying {
                variable,
                from,
                by,
                until,
                body,
            } => {
                chains
                    .defs
                    .entry(variable.clone())
                    .or_default()
                    .push(current_index);

                collect_expression_uses(from, current_index, chains);
                collect_expression_uses(by, current_index, chains);
                collect_condition_uses(until, current_index, chains);

                collect_use_def_chains(body, chains, index);
            }

            Statement::Call { using_args, .. } => {
                for name in using_args {
                    chains
                        .uses
                        .entry(name.clone())
                        .or_default()
                        .push(current_index);
                }
            }

            Statement::String { sources, into } => {
                for name in sources {
                    chains
                        .uses
                        .entry(name.clone())
                        .or_default()
                        .push(current_index);
                }

                chains
                    .defs
                    .entry(into.clone())
                    .or_default()
                    .push(current_index);
            }

            Statement::Unstring { source, into } => {
                chains
                    .uses
                    .entry(source.clone())
                    .or_default()
                    .push(current_index);

                for name in into {
                    chains
                        .defs
                        .entry(name.clone())
                        .or_default()
                        .push(current_index);
                }
            }

            Statement::Subtract { value, target }
            | Statement::Multiply { value, target }
            | Statement::Divide { value, target } => {
                chains
                    .uses
                    .entry(value.clone())
                    .or_default()
                    .push(current_index);

                chains
                    .defs
                    .entry(target.clone())
                    .or_default()
                    .push(current_index);
            }

            Statement::For {
                variable,
                start,
                step,
                until,
                body,
            } => {
                chains
                    .defs
                    .entry(variable.clone())
                    .or_default()
                    .push(current_index);

                collect_expression_uses(start, current_index, chains);
                collect_expression_uses(step, current_index, chains);
                collect_condition_uses(until, current_index, chains);

                collect_use_def_chains(body, chains, index);
            }

            _ => {}
        }
    }
}
fn collect_condition_uses(condition: &Condition, index: usize, chains: &mut UseDefChains) {
    if !condition.left.is_empty() {
        chains
            .uses
            .entry(condition.left.clone())
            .or_default()
            .push(index);
    }

    if !condition.right.is_empty() {
        chains
            .uses
            .entry(condition.right.clone())
            .or_default()
            .push(index);
    }
}
fn collect_expression_uses(expr: &Expression, index: usize, chains: &mut UseDefChains) {
    match expr {
        Expression::Variable(name) => {
            chains.uses.entry(name.clone()).or_default().push(index);
        }

        Expression::Binary { left, right, .. } => {
            collect_expression_uses(left, index, chains);

            collect_expression_uses(right, index, chains);
        }

        _ => {}
    }
}

pub fn print_use_def_chains(chains: &UseDefChains) {
    println!("");

    println!("=== USE-DEF CHAINS ===");

    let mut names: Vec<&String> = chains.defs.keys().chain(chains.uses.keys()).collect();

    names.sort();

    names.dedup();

    for name in names {
        let definitions = chains.defs.get(name);

        let uses = chains.uses.get(name).cloned().unwrap_or_default();

        println!("{} -> defs: {:?}, uses: {:?}", name, definitions, uses);
    }

    println!("");
}

#[cfg(test)]
mod use_def_tests {

    use super::*;

    #[test]
    fn test_use_def_chains() {
        let program = Program {
            variables: Vec::new(),

            paragraphs: Vec::new(),

            statements: vec![
                Statement::Move {
                    source: Source::Literal(10),

                    target: "A".to_string(),
                },
                Statement::Move {
                    source: Source::Literal(20),

                    target: "B".to_string(),
                },
                Statement::Compute {
                    target: "C".to_string(),

                    expr: Expression::Binary {
                        left: Box::new(Expression::Variable("A".to_string())),

                        operator: "+".to_string(),

                        right: Box::new(Expression::Variable("B".to_string())),
                    },
                },
                Statement::If {
                    condition: Condition {
                        left: "C".to_string(),

                        operator: ">".to_string(),

                        right: "A".to_string(),
                    },

                    then_branch: Vec::new(),

                    else_branch: None,
                },
            ],
        };

        let chains = build_use_def_chains(&program);

        assert_eq!(chains.defs.get("A"), Some(&vec![0]));

        assert_eq!(chains.defs.get("B"), Some(&vec![1]));

        assert_eq!(chains.defs.get("C"), Some(&vec![2]));

        assert_eq!(chains.uses.get("A"), Some(&vec![2, 3]));

        assert_eq!(chains.uses.get("B"), Some(&vec![2]));

        assert_eq!(chains.uses.get("C"), Some(&vec![3]));
    }
}

#[derive(Debug, Clone)]
pub struct PhiCandidate {
    pub variable: String,
    pub block: usize,
}

pub fn find_phi_candidates(program: &Program, cfg: &ControlFlowGraph) -> Vec<PhiCandidate> {
    let chains = build_use_def_chains(program);
    let mut candidates = Vec::new();

    // Map definition statement indices to CFG blocks.
    let mut statement_to_block = HashMap::new();

    let mut statement_index = 0usize;

    for (block_id, block) in cfg.blocks.iter().enumerate() {
        for _statement in &block.statements {
            statement_to_block.insert(statement_index, block_id);
            statement_index += 1;
        }
    }

    for (variable, definitions) in &chains.defs {
        if definitions.len() < 2 {
            continue;
        }

        let mut worklist = Vec::new();
        let mut visited = HashSet::new();

        // Initial definition blocks.
        for definition in definitions {
            if let Some(&block) = statement_to_block.get(definition) {
                if visited.insert(block) {
                    worklist.push(block);
                }
            }
        }

        // Iterated dominance frontier.
        while let Some(definition_block) = worklist.pop() {
            for &frontier_block in &cfg.blocks[definition_block].dominance_frontier {
                let candidate = PhiCandidate {
                    variable: variable.clone(),
                    block: frontier_block,
                };

                if !candidates.iter().any(|existing: &PhiCandidate| {
                    existing.variable == candidate.variable && existing.block == candidate.block
                }) {
                    candidates.push(candidate);
                }

                // A newly discovered frontier block can itself
                // introduce another phi placement through its
                // dominance frontier.
                if visited.insert(frontier_block) {
                    worklist.push(frontier_block);
                }
            }
        }
    }

    candidates.sort_by(|a, b| a.variable.cmp(&b.variable).then(a.block.cmp(&b.block)));

    candidates
}
#[cfg(test)]
mod phi_candidate_tests {
    use super::*;
    use crate::cfg::ControlFlowGraph;
    use crate::ir::{Condition, Program, Source, Statement};

    #[test]
    fn test_phi_candidate_for_nested_branch_definitions() {
        let program = Program {
            variables: Vec::new(),
            paragraphs: Vec::new(),
            statements: vec![
                Statement::If {
                    condition: Condition {
                        left: "A".to_string(),
                        operator: "=".to_string(),
                        right: "1".to_string(),
                    },

                    then_branch: vec![
                        Statement::Move {
                            source: Source::Literal(10),
                            target: "X".to_string(),
                        },
                        Statement::If {
                            condition: Condition {
                                left: "B".to_string(),
                                operator: "=".to_string(),
                                right: "1".to_string(),
                            },

                            then_branch: vec![Statement::Move {
                                source: Source::Literal(20),
                                target: "X".to_string(),
                            }],

                            else_branch: Some(vec![Statement::Move {
                                source: Source::Literal(30),
                                target: "X".to_string(),
                            }]),
                        },
                    ],

                    else_branch: Some(vec![Statement::Move {
                        source: Source::Literal(40),
                        target: "X".to_string(),
                    }]),
                },
                Statement::Move {
                    source: Source::Variable("X".to_string()),
                    target: "Y".to_string(),
                },
            ],
        };

        let cfg = ControlFlowGraph::build(&program);

        println!("");
        println!("=== Nested Phi Candidate Test CFG ===");
        cfg.print();

        let candidates = find_phi_candidates(&program, &cfg);

        println!("");
        println!("=== Phi Candidates ===");

        for candidate in &candidates {
            println!("variable={} block={}", candidate.variable, candidate.block);
        }

        let x_candidates: Vec<&PhiCandidate> = candidates
            .iter()
            .filter(|candidate| candidate.variable == "X")
            .collect();

        assert!(
            !x_candidates.is_empty(),
            "expected at least one phi candidate for X"
        );
    }

    #[test]
    fn test_use_def_chains_include_nested_definitions() {
        let program = Program {
            variables: Vec::new(),
            paragraphs: Vec::new(),
            statements: vec![Statement::If {
                condition: Condition {
                    left: "A".to_string(),
                    operator: "=".to_string(),
                    right: "1".to_string(),
                },

                then_branch: vec![
                    Statement::Move {
                        source: Source::Literal(10),
                        target: "X".to_string(),
                    },
                    Statement::If {
                        condition: Condition {
                            left: "B".to_string(),
                            operator: "=".to_string(),
                            right: "1".to_string(),
                        },

                        then_branch: vec![Statement::Move {
                            source: Source::Literal(20),
                            target: "X".to_string(),
                        }],

                        else_branch: Some(vec![Statement::Move {
                            source: Source::Literal(30),
                            target: "X".to_string(),
                        }]),
                    },
                ],

                else_branch: Some(vec![Statement::Move {
                    source: Source::Literal(40),
                    target: "X".to_string(),
                }]),
            }],
        };

        let chains = build_use_def_chains(&program);

        assert_eq!(chains.defs.get("X"), Some(&vec![1, 3, 4, 5]));
    }
}
