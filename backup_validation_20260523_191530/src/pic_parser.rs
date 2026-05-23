use crate::ir::{CompType, PicType};

pub fn parse_pic(pic: &str) -> Option<PicType> {
    let upper = pic.to_uppercase();

    if upper.starts_with("S9") {
        if upper.contains("V") {
            let scale = upper
                .split("V")
                .nth(1)
                .unwrap_or("")
                .chars()
                .filter(|c| *c == '9')
                .count();

            return Some(PicType::Decimal { scale });
        }

        return Some(PicType::SignedInteger);
    }

    if upper.starts_with("9") {
        if upper.contains("V") {
            let scale = upper
                .split("V")
                .nth(1)
                .unwrap_or("")
                .chars()
                .filter(|c| *c == '9')
                .count();

            return Some(PicType::Decimal { scale });
        }

        return Some(PicType::Integer);
    }

    if upper.starts_with("X") {
        return Some(PicType::String {
            length: extract_len(&upper),
        });
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

    None
}

fn extract_len(pic: &str) -> usize {
    if let Some(start) = pic.find("(") {
        if let Some(end) = pic.find(")") {
            return pic[start + 1..end].parse::<usize>().unwrap_or(1);
        }
    }

    1
}
