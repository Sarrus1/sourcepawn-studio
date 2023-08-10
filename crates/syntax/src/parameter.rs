use crate::description::Description;

#[derive(Debug, Clone)]
pub struct Parameter {
    pub type_: Option<Type>,
    pub name: String,
    pub is_const: bool,
    pub description: Description,
    pub dimensions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Type {
    pub is_pointer: bool,
    pub name: String,
    pub dimensions: Vec<String>,
}
