#[derive(Debug, Clone, PartialEq)]
pub enum PictureCategory {
    Alphabetic,
    Alphanumeric,
    Numeric,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Usage {
    Display,
    Binary,
    Comp,
    Comp3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PictureClause {
    pub signed: bool,
    pub category: PictureCategory,
    pub length: usize,
    pub scale: usize,
    pub usage: Usage,
}
