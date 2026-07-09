use crate::ir::*;
use super::symbol_table::*;

pub struct TypeChecker{

    pub symbols:SymbolTable,
}

impl TypeChecker{

    pub fn new()->Self{

        Self{

            symbols:SymbolTable::new(),
        }
    }

    pub fn analyze(&mut self,program:&Program){

        for var in &program.variables{

            self.symbols.insert(var);
        }
    }
}
