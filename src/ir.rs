use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Add { target: String, value: i64 },
    Move { source: Source, target: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Source {
    Literal(i64),
    Variable(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub body: Vec<Statement>,
}