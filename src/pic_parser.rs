use crate::ir::*;

pub fn parse_pic(pic: &str) -> Option<PicType> {

    let upper = pic.to_uppercase();

    if upper.contains("X(") || upper.contains("A(") {
        return Some(PicType::Alpha);
    }

    if upper.contains("V") {
        return Some(PicType::Decimal);
    }

    if upper.contains("9") {
        return Some(PicType::Numeric);
    }

    None
}
