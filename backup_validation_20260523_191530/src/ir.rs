use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Add {
        target: String,
        value: i64,
    },
    Move {
        source: Source,
        target: String,
    },
    If {
        condition: Condition,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    Perform {
        name: String,
    },
    While {
        condition: Condition,
        body: Vec<Statement>,
    },
    Display {
        value: Literal,
    },
    Evaluate {
        subject: String,
        also_subject: Option<String>,
        when_clauses: Vec<WhenClause>,
    },
    String {
        sources: Vec<StringSource>,
        into: String,
        pointer: Option<String>,
    },
    Unstring {
        source: String,
        delimited_by: Option<LiteralOrVariable>,
        into: Vec<String>,
        pointer: Option<String>,
    },
    Redefines {
        name: String,
        redefines: String,
    },
    Occurs {
        name: String,
        count: i64,
    },
    ConditionName {
        name: String,
        value: Literal,
    },
    Compute {
        target: String,
        expr: String,
    },
    OpenFile {
        mode: FileMode,
        name: String,
    },
    ReadFile {
        file: String,
        into: Option<String>,
    },
    WriteFile {
        file: String,
        from: Option<String>,
    },
    CloseFile {
        name: String,
    },
    ArrayGet {
        name: String,
        index: i64,
        target: String,
    },
    ArraySet {
        name: String,
        index: i64,
        value: Source,
    },
    // New features
    Accept {
        target: String,
    },
    StopRun,
    Continue,
    Exit,
    Inspect {
        source: String,
        target: String,
        pattern: String,
    },

    PerformUntil {
        condition: Condition,
        body: Vec<Statement>,
    },

    Call {
        program: String,
    },
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
pub struct StringSource {
    pub source: LiteralOrVariable,
    pub delimited_by: Option<LiteralOrVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiteralOrVariable {
    Literal(Literal),
    Variable(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Source {
    Literal(i64),
    LiteralString(String),
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
pub enum PicType {
    Integer,
    SignedInteger,
    Decimal { scale: usize },
    String { length: usize },
    Alpha { length: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompType {
    Comp,
    Comp1,
    Comp2,
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
