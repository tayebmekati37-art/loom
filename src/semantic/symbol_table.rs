use crate::ir::*;
use std::collections::HashMap;

pub type ConditionMap = HashMap<String, Literal>;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,

    pub pic: Option<PicType>,

    pub occurs: Option<usize>,

    pub redefines: Option<String>,

    pub initial_value: Option<Literal>,

    pub condition_values: ConditionMap,
}

#[derive(Default)]
pub struct SymbolTable {
    pub symbols: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    pub fn insert(&mut self, var: &VariableDefinition) {
        self.symbols.insert(
            var.name.clone(),
            Symbol {
                name: var.name.clone(),

                pic: var.pic.clone(),

                occurs: var.occurs,

                redefines: var.redefines.clone(),

                initial_value: var.initial_value.clone(),

                condition_values: HashMap::new(),
            },
        );
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }
}
