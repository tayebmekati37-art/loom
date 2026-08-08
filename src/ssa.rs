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
    pub defs: HashMap<String, usize>,

    pub uses: HashMap<String, Vec<usize>>,
}

pub fn build_use_def_chains(program: &Program) -> UseDefChains {
    let mut chains = UseDefChains::default();

    for (index, stmt) in program.statements.iter().enumerate() {
        match stmt {
            Statement::Move { source, target } => {
                chains.defs.insert(target.clone(), index);

                if let Source::Variable(name) = source {
                    chains.uses.entry(name.clone()).or_default().push(index);
                }
            }

            Statement::Compute { target, expr } => {
                chains.defs.insert(target.clone(), index);

                collect_expression_uses(expr, index, &mut chains);
            }

            Statement::If { condition, .. } => {
                collect_condition_uses(condition, index, &mut chains);
            }

            Statement::PerformUntil { condition, .. } => {
                collect_condition_uses(condition, index, &mut chains);
            }

            Statement::PerformVarying { until, .. } => {
                collect_condition_uses(until, index, &mut chains);
            }

            Statement::Call { using_args, .. } => {
                for name in using_args {
                    chains.uses.entry(name.clone()).or_default().push(index);
                }
            }

            _ => {}
        }
    }

    chains
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
