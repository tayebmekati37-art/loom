use crate::ir::{Function, Statement, Source, Literal, Condition};
use std::fmt::Write;

pub fn translate(function: &Function) -> String {
    let mut out = String::new();
    writeln!(out, "// Translation not implemented for this language").unwrap();
    out
}