use crate::ir::{Function, Statement, Source};
use std::collections::HashMap;

pub struct Interpreter {
    vars: HashMap<String, i64>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            vars: HashMap::new(),
        }
    }

    pub fn run(&mut self, func: &Function, inputs: HashMap<String, i64>) -> HashMap<String, i64> {
        // Set initial variables
        for (name, value) in inputs {
            self.vars.insert(name, value);
        }
        // Execute statements
        for stmt in &func.body {
            self.execute_statement(stmt);
        }
        // Return all variable values (or just the ones that changed)
        self.vars.clone()
    }

    fn execute_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Add { target, value } => {
                let current = *self.vars.get(target).unwrap_or(&0);
                self.vars.insert(target.clone(), current + value);
            }
            Statement::Move { source, target } => {
                let src_value = match source {
                    Source::Literal(v) => *v,
                    Source::Variable(v) => *self.vars.get(v).unwrap_or(&0),
                };
                self.vars.insert(target.clone(), src_value);
            }
        }
    }
}