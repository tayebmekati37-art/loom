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
    validate_ssa_structure(program, &cfg);
    validate_ssa_uses(program);
    validate_phi_incoming_edges(program, &cfg);
    validate_phi_placement(program, &cfg);
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

        // Map the CFG block back to the structured Program representation.
        //
        // CFG mapping used by find_phi_candidates():
        //
        //   normal statement -> one CFG block
        //
        //   For:
        //       cfg_block     = loop header
        //       cfg_block + 1 = loop body
        //       cfg_block + 2 = loop exit
        //
        // A loop-carried Phi therefore belongs immediately before
        // the corresponding top-level For statement.
        //
        // For an IF merge, preserve the existing structured placement:
        // immediately after the IF statement.

        let mut position = None;

        let mut current_cfg_block = 0usize;

        for index in 0..program.statements.len() {
            match &program.statements[index] {
                Statement::For { .. } => {
                    // A loop header is exactly the CFG block represented
                    // by this structured For statement.
                    if block_id == current_cfg_block {
                        // Only treat self-dominance-frontier blocks as
                        // loop headers for this milestone.
                        if cfg.blocks[block_id].dominance_frontier.contains(&block_id) {
                            position = Some(index);
                            break;
                        }
                    }

                    // For consumes three CFG blocks:
                    // header, body, exit.
                    current_cfg_block += 3;
                }

                Statement::If { .. } => {
                    if predecessor_count >= 2 {
                        if index + 1 < program.statements.len() {
                            position = Some(index + 1);
                        } else {
                            position = Some(program.statements.len());
                        }

                        break;
                    }

                    current_cfg_block += 1;
                }

                _ => {
                    current_cfg_block += 1;
                }
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
            .map(|variable| Statement::Phi {
                variable,
                incoming: Vec::new(),
            })
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
                if pos + 1 < name.len() && name[pos + 1..].chars().all(|c| c.is_ascii_digit()) =>
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

        self.stacks.entry(base).or_default().push(renamed.clone());

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

fn rename_dominator_tree(program: &mut Program, cfg: &ControlFlowGraph) {
    if cfg.blocks.is_empty() {
        return;
    }

    let mut state = SsaRenameState::new();

    // Shared cursor for sequential CFG -> Program statement matching.
    let mut statement_cursor = 0usize;

    rename_dom_block(program, cfg, 0, &mut state, &mut statement_cursor);
}

fn populate_successor_phi_incomings(
    program: &mut Program,
    cfg: &ControlFlowGraph,
    block_id: usize,
    state: &SsaRenameState,
) {
    if block_id >= cfg.blocks.len() {
        return;
    }

    let successors = cfg.blocks[block_id].successors.clone();

    for successor_id in successors {
        if successor_id >= cfg.blocks.len() {
            continue;
        }

        let successor_statements = cfg.blocks[successor_id].statements.clone();

        if successor_statements.is_empty() {
            continue;
        }

        /*
         * The CFG was created before Phi insertion.
         *
         * Find the first original statement belonging to the
         * successor in the current Program.
         */
        let first_cfg_statement = &successor_statements[0];

        let mut first_program_index: Option<usize> = None;

        for index in 0..program.statements.len() {
            if statement_kind_matches(&program.statements[index], first_cfg_statement) {
                first_program_index = Some(index);
                break;
            }
        }

        let Some(first_index) = first_program_index else {
            continue;
        };

        /*
         * Phi nodes are inserted immediately before the successor's
         * first original statement.
         */
        let mut phi_start = first_index;

        while phi_start > 0 {
            if matches!(&program.statements[phi_start - 1], Statement::Phi { .. }) {
                phi_start -= 1;
            } else {
                break;
            }
        }

        for index in phi_start..first_index {
            let Statement::Phi { variable, incoming } = &mut program.statements[index] else {
                continue;
            };

            let base = SsaRenameState::base_name(variable);

            let Some(current_version) = state.current(&base) else {
                continue;
            };

            if let Some(existing) = incoming
                .iter_mut()
                .find(|(predecessor, _)| *predecessor == block_id)
            {
                existing.1 = current_version;
            } else {
                incoming.push((block_id, current_version));
            }
        }
    }
}
fn rename_dom_block(
    program: &mut Program,
    cfg: &ControlFlowGraph,
    block_id: usize,
    state: &mut SsaRenameState,
    statement_cursor: &mut usize,
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

    /*
     * Phi nodes are inserted after the CFG is built, so they are not
     * present in cfg.blocks[].statements.
     *
     * Find the first original statement belonging to this CFG block.
     * Any Phi nodes immediately before that statement belong to this
     * block and must be defined before normal statements are renamed.
     */
    if !cfg.blocks[block_id].statements.is_empty() {
        let first_cfg_statement = &cfg.blocks[block_id].statements[0];

        let mut first_program_index: Option<usize> = None;

        for index in *statement_cursor..program.statements.len() {
            if statement_kind_matches(&program.statements[index], first_cfg_statement) {
                first_program_index = Some(index);
                break;
            }
        }

        if let Some(first_index) = first_program_index {
            let mut phi_start = first_index;

            while phi_start > 0 {
                if matches!(&program.statements[phi_start - 1], Statement::Phi { .. }) {
                    phi_start -= 1;
                } else {
                    break;
                }
            }

            for index in phi_start..first_index {
                let stmt = &mut program.statements[index];

                if let Statement::Phi { variable, .. } = stmt {
                    let original = variable.clone();
                    let renamed = state.define(&original);

                    *variable = renamed;
                    definitions.push(original);
                }
            }
        }
    }
    for cfg_stmt in block_statements.iter() {
        /*
         * Find the next corresponding statement by walking the
         * program sequentially.
         *
         * The cursor is shared across the dominator-tree traversal.
         *
         * We deliberately use discriminants and relevant identifying
         * fields rather than Statement == Statement.
         */

        let mut statement_index: Option<usize> = None;

        // Continue from the previous matched Program statement.
        for index in *statement_cursor..program.statements.len() {
            let candidate = &program.statements[index];

            if statement_kind_matches(candidate, cfg_stmt) {
                statement_index = Some(index);
                *statement_cursor = index + 1;
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

            Statement::For { until, .. } => {
                rename_condition_with_state(until, state);
            }

            _ => {}
        }

        /*
         * Rename definitions.
         */

        match stmt {
            Statement::Phi { variable, .. } => {
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

            Statement::If {
                then_branch,
                else_branch,
                ..
            } => {
                /*
                 * Rename each branch independently.
                 *
                 * A definition created inside THEN must not become
                 * the starting definition for ELSE. Both branches
                 * eventually converge through the Phi node at the
                 * merge point.
                 */

                let mut then_definitions = Vec::new();

                rename_structured_statements_with_state(then_branch, state, &mut then_definitions);

                for definition in then_definitions.into_iter().rev() {
                    state.pop_definition(&definition);
                }

                if let Some(else_statements) = else_branch {
                    let mut else_definitions = Vec::new();

                    rename_structured_statements_with_state(
                        else_statements,
                        state,
                        &mut else_definitions,
                    );

                    for definition in else_definitions.into_iter().rev() {
                        state.pop_definition(&definition);
                    }
                }
            }

            Statement::For { variable, body, .. } => {
                let original = variable.clone();
                let renamed = state.define(&original);

                *variable = renamed;
                definitions.push(original);

                rename_structured_statements_with_state(body, state, &mut definitions);
            }

            _ => {}
        }
    }

    /*
     * Continue down the dominator tree.
     */

    populate_successor_phi_incomings(program, cfg, block_id, state);

    let children = cfg.blocks[block_id].dom_children.clone();

    for child in children {
        rename_dom_block(program, cfg, child, state, statement_cursor);
    }

    /*
     * Restore the state when leaving this dominator scope.
     */

    for definition in definitions.into_iter().rev() {
        state.pop_definition(&definition);
    }
}

fn rename_structured_statements_with_state(
    statements: &mut Vec<Statement>,
    state: &mut SsaRenameState,
    definitions: &mut Vec<String>,
) {
    for stmt in statements.iter_mut() {
        // Rename uses first.
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

            Statement::For { until, .. } => {
                rename_condition_with_state(until, state);
            }

            _ => {}
        }

        // Rename definitions.
        match stmt {
            Statement::Phi { variable, .. } => {
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

            Statement::For { variable, body, .. } => {
                let original = variable.clone();
                let renamed = state.define(&original);

                *variable = renamed;
                definitions.push(original);

                rename_structured_statements_with_state(body, state, &mut *definitions);
            }

            _ => {}
        }
    }
}
fn statement_kind_matches(a: &Statement, b: &Statement) -> bool {
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
        (Statement::For { .. }, Statement::For { .. }) => true,

        _ => false,
    }
}

fn rename_expression_with_state(expr: &mut Expression, state: &SsaRenameState) {
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

fn rename_condition_with_state(condition: &mut Condition, state: &SsaRenameState) {
    if let Some(current) = state.current(&condition.left) {
        condition.left = current;
    }

    if let Some(current) = state.current(&condition.right) {
        condition.right = current;
    }
}
fn validate_ssa_structure(program: &Program, cfg: &ControlFlowGraph) {
    let mut definitions = HashSet::new();

    fn collect_definitions(statements: &[Statement], definitions: &mut HashSet<String>) {
        for statement in statements {
            match statement {
                Statement::Move { target, .. }
                | Statement::Add { target, .. }
                | Statement::Subtract { target, .. }
                | Statement::Multiply { target, .. }
                | Statement::Divide { target, .. }
                | Statement::Compute { target, .. } => {
                    if target.contains('_') {
                        definitions.insert(target.clone());
                    }
                }

                Statement::Phi { variable, .. } => {
                    if variable.contains('_') {
                        assert!(
                            definitions.insert(variable.clone()),
                            "SSA Phi definition appears more than once: {}",
                            variable
                        );
                    }
                }

                Statement::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    collect_definitions(then_branch, definitions);

                    if let Some(branch) = else_branch {
                        collect_definitions(branch, definitions);
                    }
                }

                Statement::PerformUntil { body, .. }
                | Statement::PerformVarying { body, .. }
                | Statement::For { body, .. } => {
                    collect_definitions(body, definitions);
                }

                _ => {}
            }
        }
    }

    collect_definitions(&program.statements, &mut definitions);

    /*
     * Map top-level Phi nodes to their actual CFG block.
     *
     * Important:
     *
     * Phi nodes do NOT consume CFG blocks.
     *
     * For an IF:
     *
     *     current block = IF control block
     *     +1             = THEN block, if present
     *     +1             = ELSE block, if present
     *     current block  = merge block
     *
     * We deliberately DO NOT advance past the merge block here.
     *
     * This means:
     *
     *     IF
     *     Phi
     *
     * maps the Phi to the merge block.
     *
     * The following normal statement then consumes that merge block.
     *
     * For a FOR:
     *
     *     current block     = loop header
     *     +1                = loop body
     *     +2                = loop exit
     *
     * A Phi immediately before FOR therefore maps to the loop header.
     */
    fn build_phi_cfg_block_map(
        statements: &[Statement],
        cfg: &ControlFlowGraph,
    ) -> HashMap<usize, usize> {
        let mut map = HashMap::new();
        let mut cfg_block = 0usize;

        for (program_index, statement) in statements.iter().enumerate() {
            match statement {
                Statement::Phi { .. } => {
                    assert!(
                        cfg_block < cfg.blocks.len(),
                        "SSA Phi could not be mapped to a CFG block: program index {}",
                        program_index
                    );

                    map.insert(program_index, cfg_block);
                }

                Statement::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    // Current block is the IF control block.
                    assert!(
                        cfg_block < cfg.blocks.len(),
                        "SSA IF could not be mapped to a CFG block"
                    );

                    cfg_block += 1;

                    // THEN branch.
                    if !then_branch.is_empty() {
                        assert!(
                            cfg_block < cfg.blocks.len(),
                            "SSA THEN branch could not be mapped to a CFG block"
                        );

                        cfg_block += 1;
                    }

                    // ELSE branch.
                    if let Some(branch) = else_branch {
                        if !branch.is_empty() {
                            assert!(
                                cfg_block < cfg.blocks.len(),
                                "SSA ELSE branch could not be mapped to a CFG block"
                            );

                            cfg_block += 1;
                        }
                    }

                    /*
                     * IMPORTANT:
                     *
                     * cfg_block is now the merge block.
                     *
                     * Do NOT increment it here.
                     *
                     * A Phi immediately following this IF must map
                     * to this block.
                     */
                }

                Statement::For { .. } => {
                    // Phi before FOR belongs to the loop header.
                    assert!(
                        cfg_block < cfg.blocks.len(),
                        "SSA FOR could not be mapped to a CFG block"
                    );

                    // Header + body + exit.
                    cfg_block += 3;
                }

                _ => {
                    assert!(
                        cfg_block < cfg.blocks.len(),
                        "SSA statement could not be mapped to a CFG block"
                    );

                    cfg_block += 1;
                }
            }
        }

        map
    }

    let phi_cfg_blocks = build_phi_cfg_block_map(&program.statements, cfg);

    fn validate_phi(
        variable: &str,
        incoming: &[(usize, String)],
        cfg: &ControlFlowGraph,
        definitions: &HashSet<String>,
        phi_cfg_block: usize,
    ) {
        assert!(
            variable.contains('_'),
            "SSA Phi definition is not versioned: {}",
            variable
        );

        let mut phi_predecessors = HashSet::new();

        for (predecessor, value) in incoming {
            assert!(
                phi_predecessors.insert(*predecessor),
                "SSA Phi contains duplicate predecessor block: {}",
                predecessor
            );

            assert!(
                *predecessor < cfg.blocks.len(),
                "SSA Phi predecessor block is invalid: {}",
                predecessor
            );

            if value.contains('_') {
                assert!(
                    definitions.contains(value),
                    "SSA Phi incoming value references undefined version: {}",
                    value
                );
            }
        }

        /*
         * Determine the exact CFG predecessor set for the Phi's block.
         */
        let expected_predecessors: HashSet<usize> = cfg
            .blocks
            .iter()
            .enumerate()
            .filter(|(_, candidate)| candidate.successors.contains(&phi_cfg_block))
            .map(|(id, _)| id)
            .collect();

        /*
         * Only a CFG block with multiple predecessors requires
         * predecessor completeness.
         */
        if expected_predecessors.len() < 2 {
            return;
        }

        let actual_predecessors: HashSet<usize> = incoming
            .iter()
            .map(|(predecessor, _)| *predecessor)
            .collect();

        /*
         * Missing predecessor.
         */
        for predecessor in expected_predecessors.difference(&actual_predecessors) {
            panic!("SSA Phi is missing predecessor block: {}", predecessor);
        }

        /*
         * Extra in-range block that is not actually a predecessor.
         */
        for predecessor in actual_predecessors.difference(&expected_predecessors) {
            panic!("SSA Phi contains non-predecessor block: {}", predecessor);
        }
    }

    fn validate_phis(
        statements: &[Statement],
        cfg: &ControlFlowGraph,
        definitions: &HashSet<String>,
        phi_cfg_blocks: &HashMap<usize, usize>,
        top_level: bool,
    ) {
        for (index, statement) in statements.iter().enumerate() {
            match statement {
                Statement::Phi { variable, incoming } => {
                    /*
                     * Current Phi insertion is top-level, so every
                     * top-level Phi must have a concrete CFG mapping.
                     */
                    let phi_cfg_block = if top_level {
                        *phi_cfg_blocks.get(&index).unwrap_or_else(|| {
                            panic!("SSA Phi has no CFG block mapping: program index {}", index)
                        })
                    } else {
                        /*
                         * Nested Phi nodes are not currently inserted
                         * by insert_phi_nodes(). Validate their local
                         * structural invariants, but do not invent a
                         * CFG block mapping.
                         */
                        validate_phi(variable, incoming, cfg, definitions, 0);

                        continue;
                    };

                    validate_phi(variable, incoming, cfg, definitions, phi_cfg_block);
                }

                Statement::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    validate_phis(then_branch, cfg, definitions, phi_cfg_blocks, false);

                    if let Some(branch) = else_branch {
                        validate_phis(branch, cfg, definitions, phi_cfg_blocks, false);
                    }
                }

                Statement::PerformUntil { body, .. }
                | Statement::PerformVarying { body, .. }
                | Statement::For { body, .. } => {
                    validate_phis(body, cfg, definitions, phi_cfg_blocks, false);
                }

                _ => {}
            }
        }
    }

    validate_phis(
        &program.statements,
        cfg,
        &definitions,
        &phi_cfg_blocks,
        true,
    );
}
fn validate_phi_placement(program: &Program, cfg: &ControlFlowGraph) {
    fn validate_statements(
        statements: &[Statement],
        cfg: &ControlFlowGraph,
        cfg_block: &mut usize,
    ) {
        for statement in statements {
            match statement {
                Statement::Phi { variable, .. } => {
                    assert!(
                        *cfg_block < cfg.blocks.len(),
                        "SSA Phi is mapped outside CFG: {}",
                        cfg_block
                    );

                    let predecessor_count = cfg
                        .blocks
                        .iter()
                        .filter(|block| block.successors.contains(cfg_block))
                        .count();

                    assert!(
                        predecessor_count >= 2,
                        "SSA Phi is not placed at a CFG merge point: {} in block {}",
                        variable,
                        cfg_block
                    );
                }

                Statement::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    *cfg_block += 1;

                    validate_statements(then_branch, cfg, cfg_block);

                    if let Some(branch) = else_branch {
                        validate_statements(branch, cfg, cfg_block);
                    }

                    // Leave cfg_block on the merge block so a Phi
                    // immediately following the IF maps to the merge.
                }

                Statement::For { body, .. } => {
                    validate_statements(body, cfg, cfg_block);

                    *cfg_block += 3;
                }

                Statement::PerformUntil { body, .. } | Statement::PerformVarying { body, .. } => {
                    *cfg_block += 1;

                    validate_statements(body, cfg, cfg_block);

                    *cfg_block += 1;
                }

                _ => {
                    *cfg_block += 1;
                }
            }
        }
    }

    let mut cfg_block = 0;

    validate_statements(&program.statements, cfg, &mut cfg_block);
}
fn validate_phi_incoming_edges(program: &Program, cfg: &ControlFlowGraph) {
    let mut definitions: HashMap<String, usize> = HashMap::new();

    fn collect_definitions(
        statements: &[Statement],
        definitions: &mut HashMap<String, usize>,
        index: &mut usize,
    ) {
        for statement in statements {
            let current_index = *index;
            *index += 1;

            match statement {
                Statement::Move { target, .. }
                | Statement::Add { target, .. }
                | Statement::Subtract { target, .. }
                | Statement::Multiply { target, .. }
                | Statement::Divide { target, .. }
                | Statement::Compute { target, .. } => {
                    if target.contains('_') {
                        definitions.insert(target.clone(), current_index);
                    }
                }

                Statement::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    collect_definitions(then_branch, definitions, index);

                    if let Some(branch) = else_branch {
                        collect_definitions(branch, definitions, index);
                    }
                }

                Statement::PerformUntil { body, .. }
                | Statement::PerformVarying { body, .. }
                | Statement::For { body, .. } => {
                    collect_definitions(body, definitions, index);
                }

                Statement::Phi { variable, .. } => {
                    if variable.contains('_') {
                        definitions.insert(variable.clone(), current_index);
                    }
                }

                _ => {}
            }
        }
    }

    collect_definitions(&program.statements, &mut definitions, &mut 0);

    fn statement_index_to_cfg_block(
        statements: &[Statement],
        target_index: usize,
        cfg_block: &mut usize,
        current_index: &mut usize,
    ) -> Option<usize> {
        for statement in statements {
            let statement_index = *current_index;
            *current_index += 1;

            if statement_index == target_index {
                return Some(*cfg_block);
            }

            match statement {
                Statement::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    *cfg_block += 1;

                    if let Some(block) = statement_index_to_cfg_block(
                        then_branch,
                        target_index,
                        cfg_block,
                        current_index,
                    ) {
                        return Some(block);
                    }

                    if let Some(branch) = else_branch {
                        if let Some(block) = statement_index_to_cfg_block(
                            branch,
                            target_index,
                            cfg_block,
                            current_index,
                        ) {
                            return Some(block);
                        }
                    }
                }

                Statement::For { body, .. } => {
                    *cfg_block += 1;

                    if let Some(block) =
                        statement_index_to_cfg_block(body, target_index, cfg_block, current_index)
                    {
                        return Some(block);
                    }

                    *cfg_block += 2;
                }

                Statement::PerformUntil { body, .. } | Statement::PerformVarying { body, .. } => {
                    *cfg_block += 1;

                    if let Some(block) =
                        statement_index_to_cfg_block(body, target_index, cfg_block, current_index)
                    {
                        return Some(block);
                    }

                    *cfg_block += 1;
                }

                Statement::Phi { .. } => {}

                _ => {
                    *cfg_block += 1;
                }
            }
        }

        None
    }

    fn validate_phis(
        statements: &[Statement],
        cfg: &ControlFlowGraph,
        definitions: &HashMap<String, usize>,
        statement_index: &mut usize,
        cfg_block: &mut usize,
        program: &[Statement],
    ) {
        for statement in statements {
            *statement_index += 1;

            match statement {
                Statement::Phi { incoming, .. } => {
                    let phi_cfg_block = *cfg_block;

                    assert!(
                        phi_cfg_block < cfg.blocks.len(),
                        "SSA Phi is mapped outside CFG: {}",
                        phi_cfg_block
                    );

                    let expected_predecessors: HashSet<usize> = cfg
                        .blocks
                        .iter()
                        .enumerate()
                        .filter(|(_, block)| block.successors.contains(&phi_cfg_block))
                        .map(|(id, _)| id)
                        .collect();

                    for (predecessor, value) in incoming {
                        if !value.contains('_') {
                            continue;
                        }

                        let definition_index =
                            definitions.get(value).copied().unwrap_or_else(|| {
                                panic!(
                                    "SSA Phi incoming value references undefined version: {}",
                                    value
                                )
                            });

                        let mut definition_cfg_block = 0;
                        let mut definition_index_cursor = 0;

                        let definition_block = statement_index_to_cfg_block(
                            program,
                            definition_index,
                            &mut definition_cfg_block,
                            &mut definition_index_cursor,
                        )
                        .unwrap_or_else(|| {
                            panic!(
                                "SSA Phi incoming value definition cannot be mapped: {}",
                                value
                            )
                        });

                        // The definition must dominate the predecessor edge.
                        let mut dominated = definition_block == *predecessor;

                        if !dominated && definition_block < cfg.blocks.len() {
                            let mut current = *predecessor;

                            loop {
                                let idom = cfg.blocks[current].idom;

                                match idom {
                                    Some(parent) if parent == definition_block => {
                                        dominated = true;
                                        break;
                                    }

                                    Some(parent) if parent != current => {
                                        current = parent;
                                    }

                                    _ => break,
                                }
                            }
                        }

                        assert!(
                            dominated,
                            "SSA Phi incoming value is not available on predecessor edge: {} from block {}",
                            value,
                            predecessor
                        );

                        if expected_predecessors.len() >= 2 {
                            assert!(
                                expected_predecessors.contains(predecessor),
                                "SSA Phi incoming predecessor is not a CFG predecessor: {}",
                                predecessor
                            );
                        }
                    }
                }

                Statement::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    *cfg_block += 1;

                    validate_phis(
                        then_branch,
                        cfg,
                        definitions,
                        statement_index,
                        cfg_block,
                        program,
                    );

                    if let Some(branch) = else_branch {
                        validate_phis(
                            branch,
                            cfg,
                            definitions,
                            statement_index,
                            cfg_block,
                            program,
                        );
                    }
                }

                Statement::For { body, .. } => {
                    *cfg_block += 1;

                    validate_phis(body, cfg, definitions, statement_index, cfg_block, program);

                    *cfg_block += 2;
                }

                Statement::PerformUntil { body, .. } | Statement::PerformVarying { body, .. } => {
                    *cfg_block += 1;

                    validate_phis(body, cfg, definitions, statement_index, cfg_block, program);

                    *cfg_block += 1;
                }

                _ => {
                    *cfg_block += 1;
                }
            }
        }
    }

    let mut statement_index = 0;
    let mut cfg_block = 0;

    validate_phis(
        &program.statements,
        cfg,
        &definitions,
        &mut statement_index,
        &mut cfg_block,
        &program.statements,
    );
}
fn validate_ssa_uses(program: &Program) {
    let mut definitions = HashSet::new();

    fn collect_definitions(statements: &[Statement], definitions: &mut HashSet<String>) {
        for statement in statements {
            match statement {
                Statement::Move { target, .. }
                | Statement::Add { target, .. }
                | Statement::Subtract { target, .. }
                | Statement::Multiply { target, .. }
                | Statement::Divide { target, .. }
                | Statement::Compute { target, .. } => {
                    if target.contains('_') {
                        definitions.insert(target.clone());
                    }
                }

                Statement::Phi { variable, .. } => {
                    if variable.contains('_') {
                        definitions.insert(variable.clone());
                    }
                }

                Statement::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    collect_definitions(then_branch, definitions);

                    if let Some(branch) = else_branch {
                        collect_definitions(branch, definitions);
                    }
                }

                Statement::PerformUntil { body, .. }
                | Statement::PerformVarying { body, .. }
                | Statement::For { body, .. } => {
                    collect_definitions(body, definitions);
                }

                _ => {}
            }
        }
    }

    fn validate_string_use(value: &str, definitions: &HashSet<String>) {
        if value.contains('_') {
            assert!(
                definitions.contains(value),
                "SSA use references undefined version: {}",
                value
            );
        }
    }

    fn validate_condition_use(condition: &Condition, definitions: &HashSet<String>) {
        validate_string_use(&condition.left, definitions);
        validate_string_use(&condition.right, definitions);
    }

    fn validate_statements(statements: &[Statement], definitions: &HashSet<String>) {
        for statement in statements {
            match statement {
                Statement::Move { source, .. } => {
                    if let Source::Variable(name) = source {
                        validate_string_use(name, definitions);
                    }
                }

                Statement::Subtract { value, .. }
                | Statement::Multiply { value, .. }
                | Statement::Divide { value, .. } => {
                    validate_string_use(value, definitions);
                }

                Statement::Compute { expr, .. } => {
                    fn validate_expression(expr: &Expression, definitions: &HashSet<String>) {
                        match expr {
                            Expression::Variable(name) => {
                                validate_string_use(name, definitions);
                            }

                            Expression::Binary { left, right, .. } => {
                                validate_expression(left, definitions);
                                validate_expression(right, definitions);
                            }

                            _ => {}
                        }
                    }

                    validate_expression(expr, definitions);
                }

                Statement::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    validate_condition_use(condition, definitions);
                    validate_statements(then_branch, definitions);

                    if let Some(branch) = else_branch {
                        validate_statements(branch, definitions);
                    }
                }

                Statement::PerformUntil { condition, body } => {
                    validate_condition_use(condition, definitions);
                    validate_statements(body, definitions);
                }

                Statement::PerformVarying { until, body, .. } => {
                    validate_condition_use(until, definitions);
                    validate_statements(body, definitions);
                }

                Statement::For { until, body, .. } => {
                    validate_condition_use(until, definitions);
                    validate_statements(body, definitions);
                }

                _ => {}
            }
        }
    }

    collect_definitions(&program.statements, &mut definitions);
    validate_statements(&program.statements, &definitions);
}
#[test]
#[should_panic(expected = "SSA Phi is mapped outside CFG")]
fn validator_rejects_phi_outside_merge_block() {
    let program = Program {
        variables: Vec::new(),
        paragraphs: Vec::new(),
        statements: vec![
            Statement::Move {
                source: Source::Literal(1),
                target: "X_0".to_string(),
            },
            Statement::Phi {
                variable: "X_1".to_string(),
                incoming: Vec::new(),
            },
        ],
    };

    let cfg = ControlFlowGraph::build(&program);

    validate_phi_placement(&program, &cfg);
}

#[test]
fn validator_accepts_phi_at_branch_merge_block() {
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
                then_branch: vec![Statement::Move {
                    source: Source::Literal(1),
                    target: "X_0".to_string(),
                }],
                else_branch: Some(vec![Statement::Move {
                    source: Source::Literal(2),
                    target: "X_1".to_string(),
                }]),
            },
            Statement::Phi {
                variable: "X_2".to_string(),
                incoming: vec![(1, "X_0".to_string()), (2, "X_1".to_string())],
            },
        ],
    };

    let cfg = ControlFlowGraph::build(&program);

    validate_phi_placement(&program, &cfg);
}
#[cfg(test)]
mod ssa_validation_tests {
    #[test]
    fn validator_accepts_phi_value_defined_on_matching_predecessor() {
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
                    then_branch: vec![Statement::Move {
                        source: Source::Literal(1),
                        target: "X_0".to_string(),
                    }],
                    else_branch: Some(vec![Statement::Move {
                        source: Source::Literal(2),
                        target: "X_1".to_string(),
                    }]),
                },
                Statement::Phi {
                    variable: "X_2".to_string(),
                    incoming: vec![(1, "X_0".to_string()), (2, "X_1".to_string())],
                },
            ],
        };

        let cfg = ControlFlowGraph::build(&program);

        validate_ssa_structure(&program, &cfg);
        validate_phi_incoming_edges(&program, &cfg);
    }

    #[test]
    #[should_panic(expected = "SSA Phi incoming value is not available on predecessor edge")]
    fn validator_rejects_phi_value_from_wrong_branch() {
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
                    then_branch: vec![Statement::Move {
                        source: Source::Literal(1),
                        target: "X_0".to_string(),
                    }],
                    else_branch: Some(vec![Statement::Move {
                        source: Source::Literal(2),
                        target: "Y_0".to_string(),
                    }]),
                },
                Statement::Phi {
                    variable: "X_1".to_string(),
                    incoming: vec![(1, "Y_0".to_string()), (2, "X_0".to_string())],
                },
            ],
        };

        let cfg = ControlFlowGraph::build(&program);

        validate_ssa_structure(&program, &cfg);
        validate_phi_incoming_edges(&program, &cfg);
    }

    use super::*;

    #[test]
    #[should_panic(expected = "SSA use references undefined version")]
    fn validator_rejects_undefined_versioned_use() {
        let program = Program {
            variables: Vec::new(),
            paragraphs: Vec::new(),
            statements: vec![Statement::Move {
                source: Source::Variable("MISSING_99".to_string()),
                target: "X_1".to_string(),
            }],
        };

        validate_ssa_uses(&program);
    }

    #[test]
    #[should_panic(expected = "SSA Phi incoming value references undefined version")]
    fn validator_rejects_undefined_phi_incoming_version() {
        let program = Program {
            variables: Vec::new(),
            paragraphs: Vec::new(),
            statements: vec![Statement::Phi {
                variable: "X_1".to_string(),
                incoming: vec![(0, "X_99".to_string()), (1, "X_0".to_string())],
            }],
        };

        let cfg = ControlFlowGraph::build(&program);

        validate_ssa_structure(&program, &cfg);
    }
    #[test]
    #[should_panic(expected = "SSA Phi predecessor block is invalid")]
    fn validator_rejects_invalid_phi_predecessor() {
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
                    then_branch: vec![Statement::Move {
                        source: Source::Literal(1),
                        target: "SUM_0".to_string(),
                    }],
                    else_branch: Some(vec![Statement::Move {
                        source: Source::Literal(2),
                        target: "SUM_1".to_string(),
                    }]),
                },
                Statement::Phi {
                    variable: "SUM_2".to_string(),
                    incoming: vec![(999, "SUM_0".to_string()), (1, "SUM_1".to_string())],
                },
            ],
        };

        let cfg = ControlFlowGraph::build(&program);

        validate_ssa_structure(&program, &cfg);
    }
    #[test]
    #[should_panic(expected = "SSA Phi is missing predecessor block")]
    fn validator_rejects_missing_phi_predecessor() {
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
                    then_branch: vec![Statement::Move {
                        source: Source::Literal(1),
                        target: "SUM_0".to_string(),
                    }],
                    else_branch: Some(vec![Statement::Move {
                        source: Source::Literal(2),
                        target: "SUM_1".to_string(),
                    }]),
                },
                Statement::Phi {
                    variable: "SUM_2".to_string(),
                    incoming: vec![(1, "SUM_0".to_string())],
                },
            ],
        };

        let cfg = ControlFlowGraph::build(&program);

        validate_ssa_structure(&program, &cfg);
    }

    #[test]
    fn validator_accepts_plain_phi_incoming_value() {
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
                    then_branch: vec![Statement::Move {
                        source: Source::Literal(1),
                        target: "SUM_0".to_string(),
                    }],
                    else_branch: Some(vec![Statement::Move {
                        source: Source::Literal(2),
                        target: "SUM_1".to_string(),
                    }]),
                },
                Statement::Phi {
                    variable: "SUM_2".to_string(),
                    incoming: vec![(1, "SUM".to_string()), (2, "SUM_0".to_string())],
                },
            ],
        };

        let cfg = ControlFlowGraph::build(&program);

        validate_ssa_structure(&program, &cfg);
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

                // The CFG contains a merge block after the THEN/ELSE
                // branches. Advance past it before mapping the next
                // top-level statement.
                cfg_block += 1;
            }

            Statement::For { body, .. } => {
                // The CFG represents a For statement as:
                //
                //   cfg_block     = loop header
                //   cfg_block + 1 = loop body
                //   cfg_block + 2 = loop exit
                //
                // The For statement itself belongs to the loop header.
                statement_to_block.insert(program_index, cfg_block);
                program_index += 1;

                // Map statements inside the loop body to the loop-body CFG block.
                if !body.is_empty() {
                    map_branch_statements(
                        body,
                        cfg_block + 1,
                        &mut program_index,
                        &mut statement_to_block,
                    );
                }

                // Advance past header, body, and exit blocks.
                cfg_block += 3;
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
                Statement::Phi { variable, .. } => Some(variable.clone()),
                _ => None,
            })
            .collect();

        let second_phis: Vec<String> = second
            .statements
            .iter()
            .filter_map(|stmt| match stmt {
                Statement::Phi { variable, .. } => Some(variable.clone()),
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

#[cfg(test)]
mod v412_regression_tests {
    use super::*;

    #[test]
    fn v412_loop_carried_variable_gets_phi_and_versions() {
        let mut program = Program {
            variables: Vec::new(),
            paragraphs: Vec::new(),
            statements: vec![
                Statement::Move {
                    source: Source::Literal(0),
                    target: "X".to_string(),
                },
                Statement::For {
                    variable: "I".to_string(),
                    start: Expression::Variable("I".to_string()),
                    step: Expression::Variable("I".to_string()),
                    body: vec![Statement::Move {
                        source: Source::Literal(1),
                        target: "X".to_string(),
                    }],
                    until: Condition {
                        left: "I".to_string(),
                        operator: ">=".to_string(),
                        right: "10".to_string(),
                    },
                },
                Statement::Compute {
                    target: "Y".to_string(),
                    expr: Expression::Variable("X".to_string()),
                },
            ],
        };

        convert_to_ssa(&mut program);

        let debug = format!("{:#?}", program);

        assert!(debug.contains("X_0"));
        assert!(debug.contains("X_1"));
        assert!(debug.contains("Y_0"));
    }
}
