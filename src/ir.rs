use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Add { target: String, value: Source },
    Move { source: Source, target: String },
    If {
        condition: Condition,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    Perform { name: String },
    While {
        condition: Condition,
        body: Vec<Statement>,
    },
    Display { value: Literal },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Source {
    Literal(i64),
    Variable(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    Int(i64),
    String(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub left: String,
    pub operator: String,
    pub right: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub body: Vec<Statement>,
}