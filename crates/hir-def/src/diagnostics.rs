use crate::{ast_id_map::AstId, Name};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DefDiagnostic {
    UnresolvedInherit {
        methodmap_ast_id: AstId,
        inherit_name: Name,
        exists: bool,
    },
}
