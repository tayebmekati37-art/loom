#[derive(Debug, Clone)]
pub enum LoomType {
    Integer,

    Decimal,

    String,

    Boolean,

    Array(Box<LoomType>),

    Unknown,
}

impl LoomType {
    pub fn from_pic(pic: &str) -> Self {
        let upper = pic.to_uppercase();

        if upper.contains("9") {
            if upper.contains("V") {
                LoomType::Decimal
            } else {
                LoomType::Integer
            }
        } else if upper.contains("X") {
            LoomType::String
        } else {
            LoomType::Unknown
        }
    }
}
