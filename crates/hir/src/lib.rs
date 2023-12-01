use base_db::Tree;
use db::HirDatabase;
use hir_def::{FileItem, FunctionId, Lookup, NodePtr};
use std::{fmt, ops};
use vfs::FileId;

pub mod db;

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
        let text = node.utf8_text(source.as_ref().as_bytes()).ok()?;
        let item_tree = self.db.file_item_tree(file_id);
        let def_map = self.db.file_def_map(file_id);
        if let Some(def) = def_map.get(text) {
            match def {
                hir_def::FileDefId::FunctionId(id) => {
                    return Some(ast_id_map[item_tree[id.lookup(self.db).value].ast_id]);
                }
                hir_def::FileDefId::VariableId(id) => {
                    return Some(ast_id_map[item_tree[id.lookup(self.db).value].ast_id]);
                }
            }
        }
        None
    }
}

impl<'db> SemanticsImpl<'db> {
    fn new(db: &'db dyn HirDatabase) -> Self {
        SemanticsImpl { db }
    }
}
