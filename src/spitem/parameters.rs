use crate::providers::hover::description::Description;

#[derive(Debug, Clone)]
pub struct Parameter {
    pub type_: Option<Type>,
    pub name: String,
    pub is_const: bool,
    pub description: Description,
}

#[derive(Debug, Clone)]
pub struct Type {
    pub is_pointer: bool,
    pub name: String,
    pub dimension: Vec<String>,
}
