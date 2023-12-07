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

    fn find_def_inner(
        &self,
        file_id: FileId,
        node: &tree_sitter::Node,
        parent: Option<tree_sitter::Node>,
    ) -> Option<NodePtr> {
        let Some(mut parent) = parent else {
            return None;
        };
        let source = self.db.file_text(file_id);
        let ast_id_map = self.db.ast_id_map(file_id);
        let def_map = self.db.file_def_map(file_id);
        let text = node.utf8_text(source.as_ref().as_bytes()).ok()?;
        while !matches!(TSKind::from(parent), TSKind::sym_block) {
            parent = parent.parent()?;
        }
        // Find the parent function.
        // Compute all the scopes for the function, and the global scope.
        // Each scope is mapped from the block id to the scope.
        // Climb up the tree from the node, and filter the ancestors which can be scopes.
        // For each scope, try to find the def in the scope.
        let ast_id = ast_id_map.ast_id_of(&parent);
        // let (body, source_map) = self.db.body(def);
        // TODO: get function def here
        // let scopes = db.expr_scopes(def);
        // let scope = match offset {
        //     None => scope_for(&scopes, &source_map, node),
        //     Some(offset) => scope_for_offset(db, &scopes, &source_map, node.file_id, offset),
        // };
        // let resolver = resolver_for_scope(db.upcast(), def, scope);
        // let block_id = BlockLoc::new(ast_id, file_id).intern(self.db);

        None
    }

    pub fn find_def(&self, file_id: FileId, node: &tree_sitter::Node) -> Option<NodePtr> {
        let source = self.db.file_text(file_id);
        let ast_id_map = self.db.ast_id_map(file_id);
        let def_map = self.db.file_def_map(file_id);
        let text = node.utf8_text(source.as_ref().as_bytes()).ok()?;
        // TODO: Clean the ? flow up
        let mut parent = node.parent()?;
        while !matches!(TSKind::from(parent), TSKind::sym_function_declaration) {
            if let Some(candidate) = parent.parent() {
                parent = candidate;
            } else {
                return None;
            }
        }
        match TSKind::from(parent) {
            TSKind::sym_function_declaration => {
                let parent_name = parent
                    .child_by_field_name("name")?
                    .utf8_text(source.as_ref().as_bytes())
                    .ok()?;
                let body_node = parent
                    .children(&mut parent.walk())
                    .find(|node| TSKind::from(*node) == TSKind::sym_block)?;
                if let Some(def) = def_map.get(parent_name) {
                    match def {
                        hir_def::FileDefId::FunctionId(id) => {
                            let def = hir_def::DefWithBodyId::FunctionId(id);
                            let offset = node.start_position();
                            let analyzer = SourceAnalyzer::new_for_body(
                                self.db,
                                def,
                                InFile::new(file_id, &body_node),
                                Some(offset),
                            );
                            if let Some(ValueNs::LocalVariable(expr)) =
                                analyzer.resolver.resolve_ident(text)
                            {
                                let (_, source_map) = self.db.body_with_source_map(def);
                                return source_map.expr_source(expr);
                            }
                        }
                        hir_def::FileDefId::VariableId(_) => todo!(),
                    }
                }
            }
            _ => todo!(),
        }
        let item_tree = self.db.file_item_tree(file_id);
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
