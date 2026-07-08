use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AstStatement {
    Move {
        source: String,
        target: String,
    },

    Add {
        value: String,
        target: String,
    },

    Display {
        value: String,
    },

    If {
        condition: AstCondition,
        then_branch: Vec<AstStatement>,
        else_branch: Option<Vec<AstStatement>>,
    },

    PerformUntil {
        condition: AstCondition,
        body: Vec<AstStatement>,
    },

    PerformVarying {
        variable: String,
        from: String,
        by: String,
        until: AstCondition,
        body: Vec<AstStatement>,
    },

    Evaluate {
        value: String,
        whens: Vec<AstWhen>,
    },

    String {
        sources: Vec<String>,
        target: String,
    },

    Unstring {
        source: String,
        delimiter: String,
        targets: Vec<String>,
    },

    Inspect {
        source: String,
        replacing_from: String,
        replacing_to: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstCondition {
    pub left: String,
    pub operator: String,
    pub right: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstWhen {
    pub condition: String,
    pub body: Vec<AstStatement>,
}
