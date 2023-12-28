use base_db::Tree;
use db::HirDatabase;
use hir_def::{
    resolver::ValueNs, DefWithBodyId, EnumStructId, ExprId, FunctionId, GlobalId, InFile,
    LocalFieldId,
};
use source_analyzer::SourceAnalyzer;
use std::{collections::HashMap, fmt, ops};
use syntax::TSKind;
use vfs::FileId;

pub mod db;
mod from_id;
mod has_source;
mod semantics;
mod source_analyzer;
mod source_to_def;

pub use crate::has_source::HasSource;
pub use crate::semantics::Semantics;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefResolution {
    Function(Function),
    EnumStruct(EnumStruct),
    Field(Field),
    Global(Global),
    Local(Local),
}

impl<'tree> HasSource<'tree> for DefResolution {
    fn source(
        self,
        db: &dyn HirDatabase,
        tree: &'tree Tree,
    ) -> Option<InFile<tree_sitter::Node<'tree>>> {
        match self {
            DefResolution::Function(func) => func.source(db, tree),
            DefResolution::EnumStruct(enum_struct) => enum_struct.source(db, tree),
            DefResolution::Field(field) => field.source(db, tree),
            DefResolution::Field(field) => field.source(db, tree),
            DefResolution::Global(global) => global.source(db, tree),
            DefResolution::Local(local) => local.source(db, tree)?.source(db, tree),
        }
    }
}

/// The defs which can be visible in the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileDef {
    Function(Function),
    EnumStruct(EnumStruct),
    Global(Global),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Function {
    pub(crate) id: FunctionId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumStruct {
    pub(crate) id: EnumStructId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Global {
    pub(crate) id: GlobalId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Field {
    pub(crate) parent: EnumStruct,
    pub(crate) id: LocalFieldId,
}

/// A single local variable definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Local {
    pub(crate) parent: DefWithBodyId,
    pub(crate) expr_id: ExprId,
}

impl<'tree> Local {
    fn source(self, db: &dyn HirDatabase, tree: &'tree Tree) -> Option<LocalSource<'tree>> {
        let (body, source_map) = db.body_with_source_map(self.parent.into());
        let node_ptr = source_map.expr_source(self.expr_id)?;
        Some(LocalSource {
            local: self,
            source: InFile::new(self.parent.file_id(db.upcast()), node_ptr.to_node(tree)),
        })
    }
}

pub struct LocalSource<'tree> {
    pub local: Local,
    pub source: InFile<tree_sitter::Node<'tree>>,
}
