use crate::ir::{Function, Statement, Source, Literal, Condition};
use std::collections::HashMap;

pub struct Interpreter {
    vars: HashMap<String, i64>,
    functions: HashMap<String, Function>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            vars: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn add_function(&mut self, func: Function) {
        self.functions.insert(func.name.clone(), func);
    }

    pub fn run(&mut self, func_name: &str, inputs: HashMap<String, i64>) -> HashMap<String, i64> {
        for (name, value) in inputs {
            self.vars.insert(name, value);
        }
        if let Some(func) = self.functions.get(func_name) {
            let body = func.body.clone();
            self.execute_block(&body);
        }
        self.vars.clone()
    }

    fn execute_block(&mut self, statements: &[Statement]) {
        for stmt in statements {
            self.execute_statement(stmt);
        }
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
            Statement::If { condition, then_branch, else_branch } => {
                if self.evaluate_condition(condition) {
                    self.execute_block(then_branch);
                } else if let Some(else_branch) = else_branch {
                    self.execute_block(else_branch);
                }
            }
            Statement::Perform { name } => {
                if let Some(func) = self.functions.get(name) {
                    let body = func.body.clone();
                    self.execute_block(&body);
                } else {
                    eprintln!("Undefined function: {}", name);
                }
            }
            Statement::While { condition, body } => {
                while self.evaluate_condition(condition) {
                    self.execute_block(body);
                }
            }
            Statement::Display { value } => {
                match value {
                    Literal::Int(i) => println!("{}", i),
                    Literal::String(s) => println!("{}", s),
                }
            }
        }
    }

    fn evaluate_condition(&self, cond: &Condition) -> bool {
        let left_val = *self.vars.get(&cond.left).unwrap_or(&0);
        match cond.operator.as_str() {
            ">" => left_val > cond.right,
            "<" => left_val < cond.right,
            "=" => left_val == cond.right,
            _ => false,
        }
    }
}