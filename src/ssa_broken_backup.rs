use crate::cfg::ControlFlowGraph;
use crate::ir::*;
use std::collections::{HashMap, HashSet};

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

pub fn insert_phi_nodes(program: &Program, cfg: &ControlFlowGraph) -> Program {
    let candidates = find_phi_candidates(program, cfg);

    if candidates.is_empty() {
        return program.clone();
    }

    let mut phis_by_block: HashMap<usize, Vec<String>> = HashMap::new();

    for candidate in candidates {
        phis_by_block
            .entry(candidate.block)
            .or_default()
            .push(candidate.variable);
    }

    /*
     * The CFG is flattened, but Program::statements is structured.
     *
     * Example:
     *
     *   Program:
     *     0: IF
     *     1: MOVE X -> Y
     *
     *   CFG:
     *     0: IF
     *     1: THEN
     *     2: ELSE
     *     3: MOVE X -> Y
     *
     * A phi candidate for block 3 therefore belongs at structured
     * program index 1, not flattened CFG offset 3.
     *
     * Build a mapping from CFG block IDs to top-level Program
     * statement boundaries.
     */
    let mut block_to_program_index: HashMap<usize, usize> = HashMap::new();

    fn count_blocks(statements: &[Statement]) -> usize {
        let mut blocks = 0usize;
        let mut current_has_statements = false;

        for statement in statements {
            match statement {
                Statement::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    if current_has_statements {
                        blocks += 1;
                        current_has_statements = false;
                    }

                    // IF condition itself is one CFG block.
                    blocks += 1;

                    // Nested THEN / ELSE blocks.
                    blocks += count_blocks(then_branch);

                    if let Some(else_branch) = else_branch {
                        blocks += count_blocks(else_branch);
                    }
                }

                Statement::Perform { .. }
                | Statement::PerformUntil { .. }
                | Statement::PerformVarying { .. }
                | Statement::Call { .. }
                | Statement::StopRun => {
                    blocks += 1;
                    current_has_statements = false;
                }

                _ => {
                    current_has_statements = true;
                }
            }
        }

        if current_has_statements {
            blocks += 1;
        }

        blocks
    }

    /*
     * Walk the structured program and determine the CFG block at which
     * each top-level statement begins.
     *
     * A block boundary is enough for phi insertion because phi nodes
     * are inserted before the first structured statement represented
     * by the CFG block.
     */
    let mut cfg_block = 0usize;

    for (program_index, statement) in program.statements.iter().enumerate() {
        block_to_program_index.insert(cfg_block, program_index);

        match statement {
            Statement::If {
                then_branch,
                else_branch,
                ..
            } => {
                // IF condition block.
                cfg_block += 1;

                // Flattened THEN blocks.
                cfg_block += count_blocks(then_branch);

                // Flattened ELSE blocks.
                if let Some(else_branch) = else_branch {
                    cfg_block += count_blocks(else_branch);
                }
            }

            Statement::Perform { .. }
            | Statement::PerformUntil { .. }
            | Statement::PerformVarying { .. }
            | Statement::Call { .. }
            | Statement::StopRun => {
                cfg_block += 1;
            }

            _ => {
                // Consecutive ordinary statements remain in the same
                // CFG block, so the next top-level statement does not
                // necessarily advance the block number.
                //
                // We still advance only when the current statement
                // actually terminates a CFG block.
            }
        }
    }

    let mut insertions: Vec<(usize, Vec<Statement>)> = Vec::new();

    for (block_id, variables) in phis_by_block {
        let position = match block_to_program_index.get(&block_id) {
            Some(position) => *position,
            None => {
                // If the candidate points beyond a structured boundary,
                // do not corrupt the program.
                continue;
            }
        };

        let mut sorted_variables = variables;
        sorted_variables.sort();
        sorted_variables.dedup();

        let phi_statements = sorted_variables
            .into_iter()
            .map(|variable| Statement::Phi { variable })
            .collect::<Vec<_>>();

        if !phi_statements.is_empty() {
            insertions.push((position, phi_statements));
        }
    }

    // Insert from the end so earlier indexes remain valid.
    insertions.sort_by(|a, b| b.0.cmp(&a.0));

    let mut result = program.clone();

    for (position, phi_statements) in insertions {
        if position <= result.statements.len() {
            result.statements.splice(position..position, phi_statements);
        }
    }

    result
}
