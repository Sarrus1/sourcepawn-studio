use syntax::TSKind;
use tree_sitter::Node;

use crate::item_tree::Name;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TypeRef {
    /// Reference to a type definition (e.g. enum struct, enum, methodmap, etc.)
    Name(Name),

    /// Old name
    OldName(Name),

    /// int
    Int,

    /// bool
    Bool,

    /// float
    Float,

    /// char
    Char,

    /// void
    Void,

    /// any
    Any,

    /// String
    OldString,

    /// Float
    OldFloat,

    /// Array
    Array((Box<TypeRef>, usize)),
}

impl TypeRef {
    pub fn from_node(node: &Node, source: &str) -> Self {
        match TSKind::from(node) {
            TSKind::anon_int => Self::Int,
            TSKind::anon_bool => Self::Bool,
            TSKind::anon_float => Self::Float,
            TSKind::anon_char => Self::Char,
            TSKind::anon_void => Self::Void,
            TSKind::any_type => Self::Any,
            TSKind::anon_String => Self::OldString,
            TSKind::anon_Float => Self::Float,
            TSKind::r#type => TypeRef::Name(Name::from_node(node, source)),
            TSKind::old_type => {
                let text = node
                    .utf8_text(source.as_bytes())
                    .expect("Failed to get utf8 text")
                    .trim_end_matches(':');
                TypeRef::OldName(Name::from(text))
            }
            _ => TypeRef::Name(Name::from_node(node, source)),
        }
    }

    pub fn from_returntype_node(node: &Node, field_name: &str, source: &str) -> Option<Self> {
        let mut type_ref = None;
        let mut size = 0;
        for child in node.children_by_field_name(field_name, &mut node.walk()) {
            match TSKind::from(child) {
                TSKind::dimension | TSKind::fixed_dimension => {
                    size += 1;
                }
                _ => {
                    type_ref = Some(TypeRef::from_node(&child, source));
                }
            }
        }
        if let Some(type_ref) = type_ref {
            if size > 0 {
                Some(Self::Array((Box::new(type_ref), size)))
            } else {
                Some(type_ref)
            }
        } else {
            None
        }
    }

    pub fn to_lower_dim(&self) -> Self {
        match self {
            TypeRef::Array((type_ref, size)) => {
                if *size > 1 {
                    TypeRef::Array((type_ref.clone(), size - 1))
                } else {
                    self.clone()
                }
            }
            _ => self.clone(),
        }
    }

    pub fn to_str(&self) -> String {
        match self {
            TypeRef::Name(name) => String::from(name.clone()), //TODO: Can we avoid this clone?
            TypeRef::OldName(name) => format!("{}:", name),
            TypeRef::Int => "int".to_string(),
            TypeRef::Bool => "bool".to_string(),
            TypeRef::Float => "float".to_string(),
            TypeRef::Char => "char".to_string(),
            TypeRef::Void => "void".to_string(),
            TypeRef::Any => "any".to_string(),
            TypeRef::OldString => "String".to_string(),
            TypeRef::OldFloat => "Float".to_string(),
            TypeRef::Array((type_ref, size)) => {
                let mut res = type_ref.to_str();
                res.push_str(&"[]".repeat(*size));
                res
            }
        }
    }
}
