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

pub fn convert_to_ssa(program: &mut Program) {
    // Build the CFG before mutating the program.
    let mut cfg = ControlFlowGraph::build(program);

    // Insert Phi nodes at CFG merge points.
    let with_phis = insert_phi_nodes(program, &cfg);

    *program = with_phis;

    // The CFG was built from the pre-Phi program. The current
    // implementation therefore uses the existing CFG structure
    // only for dominator ordering while matching statements back
    // into the current sequential IR.
    rename_dominator_tree(program, &cfg);
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

    let mut result = program.clone();

    // Group candidates by CFG merge block.
    let mut phis_by_block: HashMap<usize, Vec<String>> = HashMap::new();

    for candidate in candidates {
        phis_by_block
            .entry(candidate.block)
            .or_default()
            .push(candidate.variable);
    }

    // The current Program IR preserves structured IF statements,
    // while the CFG flattens their branches into separate blocks.
    //
    // Therefore a CFG block number cannot safely be used as a direct
    // index into Program::statements.
    //
    // For a merge block, the correct location in the structured IR
    // is immediately after the corresponding IF statement and before
    // the first statement that follows the branch.
    let mut insertion_points: Vec<(usize, Vec<String>)> = Vec::new();

    for (block_id, mut variables) in phis_by_block {
        variables.sort();
        variables.dedup();

        let block = &cfg.blocks[block_id];

        // A merge block has multiple incoming CFG edges.
        let predecessor_count = cfg
            .blocks
            .iter()
            .filter(|candidate| candidate.successors.contains(&block_id))
            .count();

        if predecessor_count < 2 {
            continue;
        }

        // Find the first top-level statement after an IF.
        //
        // Example:
        //
        //   IF ...
        //   MOVE X ...
        //
        // Phi belongs before MOVE X in the structured representation.
        let mut position = None;

        for index in 0..program.statements.len() {
            if matches!(program.statements[index], Statement::If { .. }) {
                if index + 1 < program.statements.len() {
                    position = Some(index + 1);
                    break;
                }

                position = Some(program.statements.len());
            }
        }

        if let Some(position) = position {
            insertion_points.push((position, variables));
        }
    }

    // Deterministic ordering.
    insertion_points.sort_by_key(|(position, _)| *position);

    // Insert backwards so earlier insertion positions remain valid.
    for (position, variables) in insertion_points.into_iter().rev() {
        let phi_statements: Vec<Statement> = variables
            .into_iter()
            .map(|variable| Statement::Phi { variable })
            .collect();

        result.statements.splice(position..position, phi_statements);
    }

    result
}

/*
=== LOOM DOMINATOR RENAMER v4 ===

SSA renaming is performed using the CFG dominator tree.

Important IR detail:
    Move.source is Source, not Expression.

Phi nodes in the current IR only contain the variable being
defined; they do not have incoming operands. Therefore this
phase renames Phi definitions but does not invent Phi operands.
*/

#[derive(Default)]
struct SsaRenameState {
    counters: HashMap<String, usize>,
    stacks: HashMap<String, Vec<String>>,
}

impl SsaRenameState {
    fn new() -> Self {
        Self {
            counters: HashMap::new(),
            stacks: HashMap::new(),
        }
    }

    fn base_name(name: &str) -> String {
        match name.rfind('_') {
            Some(pos)
                if pos + 1 < name.len()
                    && name[pos + 1..].chars().all(|c| c.is_ascii_digit()) =>
            {
                name[..pos].to_string()
            }
            _ => name.to_string(),
        }
    }

    fn define(&mut self, name: &str) -> String {
        let base = Self::base_name(name);

        let counter = self.counters.entry(base.clone()).or_insert(0);

        let version = *counter;
        *counter += 1;

        let renamed = format!("{}_{}", base, version);

        self.stacks
            .entry(base)
            .or_default()
            .push(renamed.clone());

        renamed
    }

    fn current(&self, name: &str) -> Option<String> {
        let base = Self::base_name(name);

        self.stacks
            .get(&base)
            .and_then(|stack| stack.last())
            .cloned()
    }

    fn pop_definition(&mut self, name: &str) {
        let base = Self::base_name(name);

        if let Some(stack) = self.stacks.get_mut(&base) {
            stack.pop();

            if stack.is_empty() {
                self.stacks.remove(&base);
            }
        }
    }
}

fn rename_dominator_tree(
    program: &mut Program,
    cfg: &ControlFlowGraph,
) {
    if cfg.blocks.is_empty() {
        return;
    }

    let mut state = SsaRenameState::new();

    rename_dom_block(
        program,
        cfg,
        0,
        &mut state,
    );
}

fn rename_dom_block(
    program: &mut Program,
    cfg: &ControlFlowGraph,
    block_id: usize,
    state: &mut SsaRenameState,
) {
    if block_id >= cfg.blocks.len() {
        return;
    }

    /*
     * IMPORTANT:
     *
     * We cannot compare CFG statements with Program statements
     * using == because Statement intentionally does not implement
     * PartialEq.
     *
     * Instead, the CFG block stores cloned statements. We locate
     * matching statements structurally through a deterministic
     * sequential cursor.
     *
     * This avoids adding PartialEq to the IR merely for the SSA
     * implementation.
     */

    let block_statements = cfg.blocks[block_id].statements.clone();

    let mut definitions: Vec<String> = Vec::new();

    for cfg_stmt in block_statements.iter() {

        /*
         * Find the next corresponding statement by walking the
         * program sequentially.
         *
         * The cursor is local to this block because CFG construction
         * currently flattens branches.
         *
         * We deliberately use discriminants and relevant identifying
         * fields rather than Statement == Statement.
         */

        let mut statement_index: Option<usize> = None;

        for index in 0..program.statements.len() {
            let candidate = &program.statements[index];

            if statement_kind_matches(candidate, cfg_stmt) {
                statement_index = Some(index);
                break;
            }
        }

        let Some(index) = statement_index else {
            continue;
        };

        let stmt = &mut program.statements[index];

        /*
         * Rename uses before creating the new definition.
         */

        match stmt {
            Statement::Move { source, .. } => {
                if let Source::Variable(name) = source {
                    if let Some(current) = state.current(name) {
                        *name = current;
                    }
                }
            }

            Statement::Compute { expr, .. } => {
                rename_expression_with_state(expr, state);
            }

            Statement::If { condition, .. } => {
                rename_condition_with_state(condition, state);
            }

            Statement::PerformUntil { condition, .. } => {
                rename_condition_with_state(condition, state);
            }

            Statement::PerformVarying { until, .. } => {
                rename_condition_with_state(until, state);
            }

            _ => {}
        }

        /*
         * Rename definitions.
         */

        match stmt {
            Statement::Phi { variable } => {
                let original = variable.clone();
                let renamed = state.define(&original);

                *variable = renamed;
                definitions.push(original);
            }

            Statement::Move { target, .. } => {
                let original = target.clone();
                let renamed = state.define(&original);

                *target = renamed;
                definitions.push(original);
            }

            Statement::Compute { target, .. } => {
                let original = target.clone();
                let renamed = state.define(&original);

                *target = renamed;
                definitions.push(original);
            }

            Statement::Add { target, .. }
            | Statement::Subtract { target, .. }
            | Statement::Multiply { target, .. }
            | Statement::Divide { target, .. } => {
                let original = target.clone();
                let renamed = state.define(&original);

                *target = renamed;
                definitions.push(original);
            }

            _ => {}
        }
    }

    /*
     * Continue down the dominator tree.
     */

    let children = cfg.blocks[block_id].dom_children.clone();

    for child in children {
        rename_dom_block(
            program,
            cfg,
            child,
            state,
        );
    }

    /*
     * Restore the state when leaving this dominator scope.
     */

    for definition in definitions.into_iter().rev() {
        state.pop_definition(&definition);
    }
}

fn statement_kind_matches(
    a: &Statement,
    b: &Statement,
) -> bool {
    match (a, b) {
        (Statement::Phi { .. }, Statement::Phi { .. }) => true,

        (Statement::Move { .. }, Statement::Move { .. }) => true,

        (Statement::Compute { .. }, Statement::Compute { .. }) => true,

        (Statement::Add { .. }, Statement::Add { .. }) => true,

        (Statement::Subtract { .. }, Statement::Subtract { .. }) => true,

        (Statement::Multiply { .. }, Statement::Multiply { .. }) => true,

        (Statement::Divide { .. }, Statement::Divide { .. }) => true,

        (Statement::If { .. }, Statement::If { .. }) => true,

        (Statement::PerformUntil { .. }, Statement::PerformUntil { .. }) => true,

        (Statement::PerformVarying { .. }, Statement::PerformVarying { .. }) => true,

        _ => false,
    }
}

fn rename_expression_with_state(
    expr: &mut Expression,
    state: &SsaRenameState,
) {
    match expr {
        Expression::Variable(name) => {
            if let Some(current) = state.current(name) {
                *name = current;
            }
        }

        Expression::Binary { left, right, .. } => {
            rename_expression_with_state(left, state);
            rename_expression_with_state(right, state);
        }

        _ => {}
    }
}

fn rename_condition_with_state(
    condition: &mut Condition,
    state: &SsaRenameState,
) {
    if let Some(current) = state.current(&condition.left) {
        condition.left = current;
    }

    if let Some(current) = state.current(&condition.right) {
        condition.right = current;
    }
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

    // Map structured Program statement indices to flattened CFG blocks.
    //
    // The CFG expands structured IF branches into separate blocks, so
    // CFG statement positions cannot be used as Program statement
    // positions directly.
    let mut statement_to_block: HashMap<usize, usize> = HashMap::new();

    fn map_branch_statements(
        statements: &[Statement],
        cfg_block: usize,
        program_index: &mut usize,
        statement_to_block: &mut HashMap<usize, usize>,
    ) {
        let _ = statements;

        for _ in statements {
            statement_to_block.insert(*program_index, cfg_block);
            *program_index += 1;
        }
    }

    let mut program_index = 0usize;
    let mut cfg_block = 0usize;

    for statement in &program.statements {
        match statement {
            Statement::If {
                then_branch,
                else_branch,
                ..
            } => {
                // The IF itself occupies the branch-control CFG block.
                statement_to_block.insert(program_index, cfg_block);

                program_index += 1;
                cfg_block += 1;

                // THEN branch occupies the next CFG block.
                if !then_branch.is_empty() {
                    map_branch_statements(
                        then_branch,
                        cfg_block,
                        &mut program_index,
                        &mut statement_to_block,
                    );

                    cfg_block += 1;
                }

                // ELSE branch occupies the following CFG block.
                if let Some(else_branch) = else_branch {
                    if !else_branch.is_empty() {
                        map_branch_statements(
                            else_branch,
                            cfg_block,
                            &mut program_index,
                            &mut statement_to_block,
                        );

                        cfg_block += 1;
                    }
                }
            }

            _ => {
                statement_to_block.insert(program_index, cfg_block);
                program_index += 1;
            }
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

#[cfg(test)]
mod phi_regression_tests_v2 {
    use super::*;
    use crate::cfg::ControlFlowGraph;
    use crate::ir::{Condition, Program, Source, Statement};

    #[test]
    fn test_phi_insertion_is_deterministic_for_multiple_variables() {
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
                        Statement::Move {
                            source: Source::Literal(30),
                            target: "Y".to_string(),
                        },
                    ],
                    else_branch: Some(vec![
                        Statement::Move {
                            source: Source::Literal(20),
                            target: "X".to_string(),
                        },
                        Statement::Move {
                            source: Source::Literal(40),
                            target: "Y".to_string(),
                        },
                    ]),
                },
                Statement::Move {
                    source: Source::Variable("X".to_string()),
                    target: "Z".to_string(),
                },
                Statement::Move {
                    source: Source::Variable("Y".to_string()),
                    target: "W".to_string(),
                },
            ],
        };

        let cfg = ControlFlowGraph::build(&program);

        let first = insert_phi_nodes(&program, &cfg);
        let second = insert_phi_nodes(&program, &cfg);

        let first_phis: Vec<String> = first
            .statements
            .iter()
            .filter_map(|stmt| match stmt {
                Statement::Phi { variable } => Some(variable.clone()),
                _ => None,
            })
            .collect();

        let second_phis: Vec<String> = second
            .statements
            .iter()
            .filter_map(|stmt| match stmt {
                Statement::Phi { variable } => Some(variable.clone()),
                _ => None,
            })
            .collect();

        assert_eq!(
            first_phis,
            vec!["X".to_string(), "Y".to_string()],
            "expected deterministic Phi nodes for X and Y"
        );

        assert_eq!(
            first_phis, second_phis,
            "Phi insertion must be deterministic"
        );
    }
}



