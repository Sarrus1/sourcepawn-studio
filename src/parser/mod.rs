use lazy_static::lazy_static;
use tree_sitter::Query;

pub mod comment_parser;
pub mod define_parser;
pub mod enum_parser;
pub mod enum_struct_parser;
pub mod function_parser;
pub mod include_parser;
pub mod methodmap_parser;
pub mod property_parser;
pub mod typedef_parser;
pub mod typeset_parser;
pub mod variable_parser;

lazy_static! {
    static ref VARIABLE_QUERY: Query = {
        Query::new(tree_sitter_sourcepawn::language(), "[(variable_declaration_statement) @declaration.variable (old_variable_declaration_statement)  @declaration.variable]").expect("Could not build variable query.")
    };
    pub(crate) static ref ERROR_QUERY: Query =
        Query::new(tree_sitter_sourcepawn::language(), "(ERROR) @error")
            .expect("Could not build error query.");
}
