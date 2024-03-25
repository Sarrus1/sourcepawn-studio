use lazy_static::lazy_static;

lazy_static! {
    pub(crate) static ref ERROR_QUERY: tree_sitter::Query =
        tree_sitter::Query::new(&tree_sitter_sourcepawn::language(), "(ERROR) @error")
            .expect("Could not build error query.");
}
