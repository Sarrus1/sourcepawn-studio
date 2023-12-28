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
mod source_analyzer;
mod source_to_def;

pub use crate::has_source::HasSource;

/// Primary API to get semantic information, like types, from syntax trees.
pub struct Semantics<'db, DB> {
    pub db: &'db DB,
    imp: SemanticsImpl<'db>,
}

pub struct SemanticsImpl<'db> {
    pub db: &'db dyn HirDatabase,
    // s2d_cache: RefCell<SourceToDefCache>,
    // Rootnode to HirFileId cache
    // cache: RefCell<FxHashMap<SyntaxNode, HirFileId>>,
}

impl<DB> fmt::Debug for Semantics<'_, DB> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Semantics {{ ... }}")
    }
}

impl<'db, DB> ops::Deref for Semantics<'db, DB> {
    type Target = SemanticsImpl<'db>;

    fn deref(&self) -> &Self::Target {
        &self.imp
    }
}

impl<'db, DB: HirDatabase> Semantics<'db, DB> {
    pub fn new(db: &DB) -> Semantics<'_, DB> {
        let impl_ = SemanticsImpl::new(db);
        Semantics { db, imp: impl_ }
    }

    pub fn parse(&self, file_id: FileId) -> Tree {
        self.db.parse(file_id)
    }

    pub fn find_def(&self, file_id: FileId, node: &tree_sitter::Node) -> Option<DefResolution> {
        let source = self.db.file_text(file_id);
        let def_map = self.db.file_def_map(file_id);
        let text = node.utf8_text(source.as_ref().as_bytes()).ok()?;
        let parent = node.parent()?;
        // match TSKind::from(parent) {
        //     TSKind::sym_function_declaration => Some(DefResolution::Function(Function::from(
        //         FunctionId::from(def_map.get_from_str(text)?),
        //     ))),
        // };

        let mut container = node.parent()?;
        // If the node does not have a parent we are at the root, nothing to resolve.
        while !matches!(TSKind::from(container), TSKind::sym_function_definition) {
            if let Some(candidate) = container.parent() {
                container = candidate;
            } else {
                break;
            }
        }
        match TSKind::from(container) {
            TSKind::sym_function_definition => {
                let parent_name = container
                    .child_by_field_name("name")?
                    .utf8_text(source.as_ref().as_bytes())
                    .ok()?;
                let body_node = container.child_by_field_name("body")?;
                match TSKind::from(body_node) {
                    TSKind::sym_block => match def_map.get_from_str(parent_name)? {
                        hir_def::FileDefId::FunctionId(id) => {
                            let def = hir_def::DefWithBodyId::FunctionId(id);
                            let offset = node.start_position();
                            if TSKind::sym_field_access == TSKind::from(parent) {
                                let analyzer = SourceAnalyzer::new_for_body(
                                    self.db,
                                    def,
                                    InFile::new(file_id, &body_node),
                                    Some(offset),
                                );
                                let field = analyzer.resolve_field(self.db, &parent)?;
                                return Some(DefResolution::Field(field));
                            }

                            // TODO: The part below seems hacky...
                            let analyzer = SourceAnalyzer::new_for_body(
                                self.db,
                                def,
                                InFile::new(file_id, &body_node),
                                Some(offset),
                            );
                            let value_ns = analyzer.resolver.resolve_ident(text).or_else(|| {
                                let analyzer = SourceAnalyzer::new_for_body(
                                    self.db,
                                    def,
                                    InFile::new(file_id, &body_node),
                                    None,
                                );
                                analyzer.resolver.resolve_ident(text)
                            });

                            match value_ns? {
                                // TODO: Maybe hide the match logic in a function/macro?
                                ValueNs::LocalId(expr) => {
                                    return Some(DefResolution::Local(Local::from(expr)));
                                }
                                ValueNs::FunctionId(id) => {
                                    return Some(DefResolution::Function(Function::from(id.value)));
                                }
                                ValueNs::GlobalId(id) => {
                                    return Some(DefResolution::Global(Global::from(id.value)));
                                }
                                ValueNs::EnumStructId(id) => {
                                    return Some(DefResolution::EnumStruct(EnumStruct::from(
                                        id.value,
                                    )));
                                }
                            }
                        }
                        _ => unreachable!("Expected a function"),
                    },
                    _ => todo!("Handle non block body"),
                }
            }
            TSKind::sym_source_file => {
                if let Some(def) = def_map.get_from_str(text) {
                    match def {
                        hir_def::FileDefId::FunctionId(id) => {
                            return Some(DefResolution::Function(Function::from(id)));
                        }
                        hir_def::FileDefId::VariableId(id) => {
                            return Some(DefResolution::Global(Global::from(id)));
                        }
                        hir_def::FileDefId::EnumStructId(id) => {
                            return Some(DefResolution::EnumStruct(EnumStruct::from(id)));
                        }
                    }
                }
            }
            _ => todo!(),
        }
        None
    }
}

impl<'db> SemanticsImpl<'db> {
    fn new(db: &'db dyn HirDatabase) -> Self {
        SemanticsImpl { db }
    }
}

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
