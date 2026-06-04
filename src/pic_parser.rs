use crate::ir::{CompType, PicType};

pub fn parse_pic(pic: &str) -> Option<PicType> {
    let upper = pic.trim().to_uppercase();

    if upper.contains("COMP-3") {
        return Some(PicType::PackedDecimal);
    }

    if upper.contains('V') {
        return Some(PicType::Decimal);
    }

    if upper.contains('9') {
        return Some(PicType::Numeric);
    }

    if upper.contains('X') || upper.contains('A') {
        return Some(PicType::Alpha);
    }

    None
}

pub fn parse_comp(line: &str) -> Option<CompType> {
    let upper = line.trim().to_uppercase();

    if upper.contains("COMP-3") {
        return Some(CompType::Comp3);
    }

    if upper.contains("COMP") {
        return Some(CompType::Comp);
    }

    None
}

