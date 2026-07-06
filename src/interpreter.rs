
use crate::ir::Function;
use std::collections::HashMap;

pub struct Interpreter {
    vars: HashMap<String, i64>,
    functions: HashMap<String, Function>,
    paragraphs: HashMap<String, Vec<crate::ir::Statement>>,
}

impl Interpreter {

    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            functions: HashMap::new(),
            paragraphs: HashMap::new(),
        }
    }

    pub fn load_program(&mut self, program: &crate::ir::Program) {
        for para in &program.paragraphs {
            self.paragraphs
                .insert(para.name.clone(), para.statements.clone());
        }
    }

    pub fn add_function(&mut self, func: Function) {
        self.functions.insert(func.name.clone(), func);
    }

    fn eval_condition_value(&self, value: &str) -> i64 {

        let value = value.trim();

        if let Ok(v) = value.parse::<i64>() {
            return v;
        }

        *self.vars.get(value).unwrap_or(&0)
    }

    fn evaluate_expression(
        &self,
        expr: &crate::ir::Expression,
    ) -> i64 {

        match expr {

            crate::ir::Expression::Literal(v) => {
                match v {
                    crate::ir::Literal::Int(i) => *i,
                    _ => 0,
                }
            }

            crate::ir::Expression::Variable(name) => {
                *self.vars.get(name).unwrap_or(&0)
            }

            crate::ir::Expression::Binary {
                left,
                operator,
                right,
            } => {

                let l = self.evaluate_expression(left);
                let r = self.evaluate_expression(right);

                match operator.as_str() {

                    "+" => l + r,
                    "-" => l - r,
                    "*" => l * r,

                    "/" => {
                        if r == 0 {
                            0
                        } else {
                            l / r
                        }
                    }

                    _ => 0,
                }
            }
        }
    }

    pub fn run(
        &mut self,
        func_name: &str,
        inputs: HashMap<String, i64>,
    ) -> HashMap<String, i64> {

        for (name, value) in inputs {
            self.vars.insert(name, value);
        }

        if let Some(func) = self.functions.get(func_name) {
            let body = func.body.clone();
            self.execute_block(&body);
        }

        self.vars.clone()
    }

    fn execute_block(
        &mut self,
        statements: &[crate::ir::Statement],
    ) {

        for stmt in statements {
            self.execute_statement(stmt);
        }
    }

    fn execute_statement(
        &mut self,
        stmt: &crate::ir::Statement,
    ) {

        match stmt {

            crate::ir::Statement::NoOp => {}
            crate::ir::Statement::ConditionName {
                name,
                value,
            } => {

                println!(
                    "CONDITION-NAME {} = {}",
                    name,
                    value
                );
            }

            crate::ir::Statement::Inspect {
                source,
                replacing,
                with,
            } => {

                println!(
                    "INSPECT {} replacing {} with {}",
                    source,
                    replacing,
                    with
                );
            }


            crate::ir::Statement::Add {
                target,
                value,
            } => {

                let current =
                    *self.vars.get(target).unwrap_or(&0);

                self.vars
                    .insert(target.clone(), current + value);
            }

            crate::ir::Statement::Move {
                source,
                target,
            } => {

                let src_value = match source {

                    crate::ir::Source::Literal(i) => *i,

                    crate::ir::Source::LiteralString(_) => 0,

                    crate::ir::Source::Variable(v) => {
                        *self.vars.get(v).unwrap_or(&0)
                    }
                };

                self.vars.insert(target.clone(), src_value);
            }

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

            crate::ir::Statement::Perform {
                name,
                body,
            } => {

                if !body.is_empty() {

                    self.execute_block(body);

                } else if let Some(name) = name {

                    if let Some(func) =
                        self.functions.get(name)
                    {
                        let body = func.body.clone();
                        self.execute_block(&body);
                    }
                }
            }

            

            crate::ir::Statement::PerformVarying {
                variable,
                from,
                by,
                until,
                body,
            } => {

                let mut current =
                    self.evaluate_expression(from);

                let step =
                    self.evaluate_expression(by);

                self.vars
                    .insert(variable.clone(), current);

                loop {

                    let left =
                        self.eval_condition_value(&until.left);

                    let right =
                        self.eval_condition_value(&until.right);

                    let done = match until.operator.as_str() {

                        "=" => left == right,
                        "!=" => left != right,
                        ">" => left > right,
                        "<" => left < right,
                        ">=" => left >= right,
                        "<=" => left <= right,

                        _ => false,
                    };

                    if done {
                        break;
                    }

                    self.execute_block(body);

                    current += step;

                    self.vars
                        .insert(variable.clone(), current);
                }
            }

            crate::ir::Statement::Display { value } => {

                match value {

                    crate::ir::Literal::Int(i) => {
                        println!("{}", i)
                    }

                    crate::ir::Literal::String(s) => {
                        println!("{}", s)
                    }
                }
            }

            
crate::ir::Statement::Compute {
    target,
    expr,
} => {

    let value =
        self.evaluate_expression(expr);

    self.vars.insert(target.clone(), value);
}



            crate::ir::Statement::Call {
                program,
                ..
            } => {

                println!("CALL {}", program);
            }

            crate::ir::Statement::Evaluate { .. } => {}
            crate::ir::Statement::String { .. } => {}
            crate::ir::Statement::Unstring { .. } => {}
            crate::ir::Statement::Redefines { .. } => {}
            crate::ir::Statement::Occurs { .. } => {}
            crate::ir::Statement::ConditionName { .. } => {}
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
            crate::ir::Statement::Inspect { .. } => {},

            crate::ir::Statement::For {
                variable,
                start,
                step,
                until,
                body,
            } => {

                let mut current =
                    self.evaluate_expression(start);

                let increment =
                    self.evaluate_expression(step);

                self.vars.insert(
                    variable.clone(),
                    current,
                );

                loop {

                    let left =
                        self.eval_condition_value(
                            &until.left
                        );

                    let right =
                        self.eval_condition_value(
                            &until.right
                        );

                    let done = match until.operator.as_str() {

                        "=" => left == right,
                        "!=" => left != right,
                        ">" => left > right,
                        "<" => left < right,
                        ">=" => left >= right,
                        "<=" => left <= right,

                        _ => false,
                    };

                    if done {
                        break;
                    }

                    self.execute_block(body);

                    current += increment;

                    self.vars.insert(
                        variable.clone(),
                        current,
                    );
                }
            }

            crate::ir::Statement::PerformUntil {
    condition,
    body,
} => {

    while !self.evaluate_condition(condition) {
        self.execute_block(body);
    }
}
        }
    }

    fn evaluate_condition(
        &self,
        cond: &crate::ir::Condition,
    ) -> bool {

        let left_val =
            *self.vars.get(&cond.left).unwrap_or(&0);

        let right_val =
            cond.right.parse::<i64>().unwrap_or(0);

        match cond.operator.as_str() {

            ">" => left_val > right_val,
            "<" => left_val < right_val,
            "=" => left_val == right_val,

            _ => false,
        }
    }
}




