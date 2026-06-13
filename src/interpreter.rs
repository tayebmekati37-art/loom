use crate::ir::Function;
use std::collections::HashMap;
use std::process::Command;

pub trait LegacyRunner {
    fn run(&self, code: &str, inputs: HashMap<String, i64>)
        -> anyhow::Result<HashMap<String, i64>>;
}

pub struct CommandRunner {
    pub command: String,
    pub args: Vec<String>,
    pub tempfile_extension: String,
}

impl CommandRunner {
    pub fn new(command: &str, args: Vec<String>, ext: &str) -> Self {
        Self {
            command: command.to_string(),
            args,
            tempfile_extension: ext.to_string(),
        }
    }
}

impl LegacyRunner for CommandRunner {
    fn run(
        &self,
        code: &str,
        inputs: HashMap<String, i64>,
    ) -> anyhow::Result<HashMap<String, i64>> {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path().with_extension(&self.tempfile_extension);

        std::fs::write(&path, code)?;

        let mut cmd = Command::new(&self.command);

        cmd.args(&self.args).arg(&path);

        for (name, value) in inputs {
            cmd.env(name, value.to_string());
        }

        let output = cmd.output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Legacy execution failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let stdout = String::from_utf8(output.stdout)?;

        parse_output(&stdout)
    }
}

fn parse_output(s: &str) -> anyhow::Result<HashMap<String, i64>> {
    let mut map = HashMap::new();

    for line in s.lines() {
        if let Some((key, val)) = line.split_once('=') {
            map.insert(key.trim().to_string(), val.trim().parse::<i64>()?);
        }
    }

    Ok(map)
}

pub struct Interpreter {
    vars: HashMap<String, i64>,
    functions: HashMap<String, Function>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
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

    fn execute_block(&mut self, statements: &[crate::ir::Statement]) {
        for stmt in statements {
            self.execute_statement(stmt);
        }
    }

    fn execute_statement(&mut self, stmt: &crate::ir::Statement) {
        match stmt {
            crate::ir::Statement::NoOp => {}

            crate::ir::Statement::Add { target, value } => {
                let current = *self.vars.get(target).unwrap_or(&0);
                self.vars.insert(target.clone(), current + value);
            }

            crate::ir::Statement::Move { source, target } => {
                let src_value = match source {
                    crate::ir::Source::Literal(i) => *i,
                    crate::ir::Source::LiteralString(_) => 0,
                    crate::ir::Source::Variable(v) => *self.vars.get(v).unwrap_or(&0),
                };

                self.vars.insert(target.clone(), src_value);
            }

            // rest of your existing arms...
            crate::ir::Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.evaluate_condition(condition) {
                    self.execute_block(then_branch);
                } else if let Some(else_branch) = else_branch {
                    self.execute_block(else_branch);
                }
            }

            crate::ir::Statement::Perform { name } => {
                if let Some(func) = self.functions.get(name) {
                    let body = func.body.clone();
                    self.execute_block(&body);
                }
            }

            crate::ir::Statement::While { condition, body } => {
                while self.evaluate_condition(condition) {
                    self.execute_block(body);
                }
            }

            crate::ir::Statement::Display { value } => match value {
                crate::ir::Literal::Int(i) => println!("{}", i),
                crate::ir::Literal::String(s) => println!("{}", s),
            },

            crate::ir::Statement::Evaluate { .. } => {}
            crate::ir::Statement::String { .. } => {}
            crate::ir::Statement::Unstring { .. } => {}
            crate::ir::Statement::Redefines { .. } => {}
            crate::ir::Statement::Occurs { .. } => {}
            crate::ir::Statement::ConditionName { .. } => {}
            crate::ir::Statement::Compute { .. } => {}
            crate::ir::Statement::OpenFile { .. } => {}
            crate::ir::Statement::ReadFile { .. } => {}
            crate::ir::Statement::WriteFile { .. } => {}
            crate::ir::Statement::CloseFile { .. } => {}
            crate::ir::Statement::ArrayGet { .. } => {}
            crate::ir::Statement::ArraySet { .. } => {}
            crate::ir::Statement::Accept { .. } => {}
            crate::ir::Statement::StopRun => {}
            crate::ir::Statement::Continue => {}
            crate::ir::Statement::Exit => {}
            crate::ir::Statement::Inspect { .. } => {}

            crate::ir::Statement::PerformUntil { condition, body } => {
                while self.evaluate_condition(condition) {
                    self.execute_block(body);
                }
            }

            crate::ir::Statement::Call { program, .. } => {
                println!("CALL {}", program);
            }
        }
    }

    fn evaluate_condition(&self, cond: &crate::ir::Condition) -> bool {
        let left_val = *self.vars.get(&cond.left).unwrap_or(&0);
        let right_val = cond.right.parse::<i64>().unwrap_or(0);

      match cond.operator.as_str() {
      ">" => left_val > right_val,
      "<" => left_val < right_val,
      "=" => left_val == right_val,
      _ => false,
      } 
    }
}
