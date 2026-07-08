use crate::ir::*;
use crate::types::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct SemanticAnalyzer {
    pub symbols: HashMap<String, String>,
    pub variable_types: HashMap<String, LoomType>,
    pub errors: Vec<String>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            variable_types: HashMap::new(),
            errors: Vec::new(),
        }
    }

    pub fn analyze_program(&mut self, program: &Program) {
        for var in &program.variables {
            if self.symbols.contains_key(&var.name) {
                self.errors
                    .push(format!("Duplicate variable: {}", var.name));
            } else {
                self.symbols
                    .insert(var.name.clone(), "variable".to_string());
            }
        }

        for stmt in &program.statements {
            self.analyze_statement(stmt);
        }
    }

    fn analyze_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Move { target, .. } => {
                self.ensure_exists(target);
            }

            Statement::Add { target, .. } => {
                self.ensure_exists(target);
            }

            Statement::Compute { target, .. } => {
                self.ensure_exists(target);
            }

            Statement::If {
                then_branch,
                else_branch,
                ..
            } => {
                for stmt in then_branch {
                    self.analyze_statement(stmt);
                }

                if let Some(branch) = else_branch {
                    for stmt in branch {
                        self.analyze_statement(stmt);
                    }
                }
            }

            Statement::PerformUntil { body, .. } => {
                for stmt in body {
                    self.analyze_statement(stmt);
                }
            }

            _ => {}
        }
    }

    fn ensure_exists(&mut self, name: &str) {
        if !self.symbols.contains_key(name) {
            self.errors.push(format!("Undefined variable: {}", name));
        }
    }
}
