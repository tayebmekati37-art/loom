use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Add { target: String, value: i64 },
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
    Evaluate {
        subject: String,
        also_subject: Option<String>,
        when_clauses: Vec<WhenClause>,
    },
<<<<<<< HEAD
=======
    OpenFile { mode: FileMode, name: String },
    ReadFile { file: String, into: Option<String> },
    WriteFile { file: String, from: Option<String> },
    CloseFile { name: String },
>>>>>>> 1660d98 (Add file I/O support (OPEN, READ, WRITE, CLOSE) for COBOL to Python; fix UTF-8 by using ASCII bytes)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhenClause {
    pub condition: WhenCondition,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WhenCondition {
    Literal(Literal),
    Variable(String),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileMode {
    Input,
    Output,
    IO,
}
