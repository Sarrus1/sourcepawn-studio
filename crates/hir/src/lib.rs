use base_db::Tree;
use db::HirDatabase;
use hir_def::{
    resolver::ValueNs, BlockLoc, DefWithBodyId, ExprId, FileItem, FunctionId, InFile, Lookup,
    NodePtr, VariableId,
};
use source_analyzer::SourceAnalyzer;
use std::{fmt, ops};
use syntax::TSKind;
use vfs::FileId;

pub mod db;
mod source_analyzer;

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

    pub fn find_def(&self, file_id: FileId, node: &tree_sitter::Node) -> Option<NodePtr> {
        let source = self.db.file_text(file_id);
        let ast_id_map = self.db.ast_id_map(file_id);
        let def_map = self.db.file_def_map(file_id);
        let text = node.utf8_text(source.as_ref().as_bytes()).ok()?;

        let mut parent = node.parent()?;
        // If the node does not have a parent we are at the root, nothing to resolve.

        while !matches!(TSKind::from(parent), TSKind::sym_function_definition) {
            if let Some(candidate) = parent.parent() {
                parent = candidate;
            } else {
                break;
            }
        }
        match TSKind::from(parent) {
            TSKind::sym_function_definition => {
                let parent_name = parent
                    .child_by_field_name("name")?
                    .utf8_text(source.as_ref().as_bytes())
                    .ok()?;
                let body_node = parent.child_by_field_name("body")?;
                match TSKind::from(body_node) {
                    TSKind::sym_block => match def_map.get_from_str(parent_name)? {
                        hir_def::FileDefId::FunctionId(id) => {
                            let def = hir_def::DefWithBodyId::FunctionId(id);
                            let offset = node.start_position();
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
                                ValueNs::LocalVariable(expr) => {
                                    let (_, source_map) = self.db.body_with_source_map(def);
                                    return source_map.expr_source(expr);
                                }
                                ValueNs::FunctionId(id) => {
                                    let item_tree = self.db.file_item_tree(file_id);
                                    return Some(
                                        ast_id_map
                                            [item_tree[id.value.lookup(self.db).value].ast_id],
                                    );
                                }
                                ValueNs::GlobalVariable(id) => {
                                    let item_tree = self.db.file_item_tree(file_id);
                                    return Some(
                                        ast_id_map
                                            [item_tree[id.value.lookup(self.db).value].ast_id],
                                    );
                                }
                                ValueNs::EnumStructId(id) => {
                                    let item_tree = self.db.file_item_tree(file_id);
                                    return Some(
                                        ast_id_map
                                            [item_tree[id.value.lookup(self.db).value].ast_id],
                                    );
                                }
                            }
                        }
                        _ => unreachable!("Expected a function"),
                    },
                    _ => todo!("Handle non block body"),
                }
            }
            TSKind::sym_source_file => {
                let item_tree = self.db.file_item_tree(file_id);
                if let Some(def) = def_map.get_from_str(text) {
                    match def {
                        hir_def::FileDefId::FunctionId(id) => {
                            return Some(ast_id_map[item_tree[id.lookup(self.db).value].ast_id]);
                        }
                        hir_def::FileDefId::VariableId(id) => {
                            return Some(ast_id_map[item_tree[id.lookup(self.db).value].ast_id]);
                        }
                        hir_def::FileDefId::EnumStructId(id) => {
                            return Some(ast_id_map[item_tree[id.lookup(self.db).value].ast_id]);
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
pub enum PathResolution {
    /// An item
    Def(ModuleDef),
    /// A local binding (only value namespace)
    Local(Local),
}

/// The defs which can be visible in the module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleDef {
    Function(FunctionId),
    Variable(VariableId),
}

/// A single local definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Local {
    pub(crate) parent: DefWithBodyId,
    pub(crate) expr_id: ExprId,
}
