use crate::ir::Function;
use std::fmt::Write;

pub fn translate(_function: &Function) -> String {
    let mut out = String::new();
    writeln!(out, "// Translation not implemented for this language").unwrap();
    out
}

