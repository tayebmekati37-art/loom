
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    Literal(Literal),
    Variable(String),

    Binary {
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub left: String,
    pub operator: String,
    pub right: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    Int(i64),
    String(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Source {
    Literal(i64),
    LiteralString(String),
    Variable(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {

    For {
        variable: String,
        start: Expression,
        step: Expression,
        until: Condition,
        body: Vec<Statement>,
    },

    NoOp,

    Display {
        value: Literal,
    },

    Move {
        source: Source,
        target: String,
    },

    Add {
        value: i64,
        target: String,
    },

    Compute {
        target: String,
        expr: Expression,
    },

    If {
        condition: Condition,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },

    Perform {
        name: Option<String>,
        body: Vec<Statement>,
    },

    PerformUntil {
        condition: Condition,
        body: Vec<Statement>,
    },

    PerformVarying {
        variable: String,
        from: Expression,
        by: Expression,
        until: Condition,
        body: Vec<Statement>,
    },

    Call {
        program: String,
        using_args: Vec<String>,
    },

    StopRun,

    Continue,

    Exit,

    OpenFile {
        name: String,
        mode: FileMode,
    },

    ReadFile {
        name: String,
    },

    WriteFile {
        name: String,
    },

    CloseFile {
        name: String,
    },

    ArrayGet {
        array: String,
        index: usize,
        target: String,
    },

    ArraySet {
        array: String,
        index: usize,
        value: String,
    },

    Accept {
        variable: String,
    },

    Evaluate {
        value: String,
    },

    String {
        value: String,
    },

    Unstring {
        value: String,
    },

    Redefines {
        name: String,
    },

    Occurs {
        name: String,
        times: usize,
    },

    ConditionName {
        name: String,
    },

    Inspect {
        value: String,
    },
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PicType {
    Numeric,
    Decimal,
    PackedDecimal,
    Alpha,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompType {
    Comp,
    Comp3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataField {
    pub level: u32,
    pub name: String,
    pub pic: Option<PicClause>,
    pub occurs: Option<u32>,
    pub redefines: Option<String>,
    pub usage: Option<UsageClause>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PicClause {
    pub raw: String,
    pub category: PicCategory,
    pub length: usize,
    pub scale: usize,
    pub signed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PicCategory {
    Numeric,
    AlphaNumeric,
    Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageClause {
    Display,
    Comp,
    Comp3,
    Binary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDefinition {
    pub name: String,
    pub pic: Option<PicType>,
    pub occurs: Option<usize>,
    pub redefines: Option<String>,
    pub initial_value: Option<Literal>,
    pub comp_type: Option<CompType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    pub name: String,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub variables: Vec<VariableDefinition>,
    pub paragraphs: Vec<Paragraph>,
    pub statements: Vec<Statement>,
}


