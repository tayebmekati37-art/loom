use crate::ir::{CompType, PicType};

pub fn parse_pic(pic: &str) -> Option<PicType> {

    let upper = pic.to_uppercase();

    if upper.contains('9') {

        if upper.contains('V') {
            return Some(PicType::Decimal);
        }

        if upper.contains('S') {
            return Some(PicType::SignedNumeric);
        }

        return Some(PicType::Numeric);
    }

    if upper.contains('X') {
        return Some(PicType::AlphaNumeric);
    }

    None
}

pub fn parse_comp(line: &str) -> Option<CompType> {

    let upper = line.to_uppercase();

    if upper.contains("COMP-3") {
        return Some(CompType::Comp3);
    }

    if upper.contains("COMP") {
        return Some(CompType::Comp);
    }

    if upper.contains("BINARY") {
        return Some(CompType::Binary);
    }

    None
}