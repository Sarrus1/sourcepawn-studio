//! Defines the [`Semantics`](Semantics) struct.

use std::{cell::RefCell, fmt, ops, sync::Arc};

use base_db::{is_name_node, Tree};
use fxhash::FxHashMap;
use hir_def::{resolver::ValueNs, InFile, NodePtr};
use syntax::TSKind;
use vfs::FileId;

use crate::{
    db::HirDatabase,
    source_analyzer::SourceAnalyzer,
    source_to_def::{SourceToDefCache, SourceToDefCtx},
    DefResolution, EnumStruct, Field, Function, Global, Local,
};

/// Primary API to get semantic information, like types, from syntax trees.
///
/// For now, it only allows to get from a node in a tree-sitter CST, to a definition.
pub struct Semantics<'db, DB> {
    pub db: &'db DB,
    imp: SemanticsImpl<'db>,
}

pub struct SemanticsImpl<'db> {
    pub db: &'db dyn HirDatabase,
    s2d_cache: RefCell<SourceToDefCache>,
    cache: RefCell<FxHashMap<NodePtr, FileId>>, //FIXME: What is this?
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

    fn find_name_def(&self, file_id: FileId, node: &tree_sitter::Node) -> Option<DefResolution> {
        if !is_name_node(node) {
            return None;
        }
        let parent = node.parent()?;
        let src = InFile::new(file_id, NodePtr::from(&parent));

        match TSKind::from(parent) {
            TSKind::function_definition => self
                .fn_to_def(src)
                .map(Function::from)
                .map(DefResolution::Function),
            TSKind::enum_struct => self
                .enum_struct_to_def(src)
                .map(EnumStruct::from)
                .map(DefResolution::EnumStruct),
            TSKind::enum_struct_field => self
                .field_to_def(src)
                .map(Field::from)
                .map(DefResolution::Field),
            TSKind::parameter_declaration => self
                .local_to_def(src)
                .map(Local::from)
                .map(DefResolution::Local),
            TSKind::variable_declaration => {
                let grand_parent = parent.parent()?;
                match TSKind::from(&grand_parent) {
                    TSKind::global_variable_declaration => self
                        .global_to_def(src)
                        .map(Global::from)
                        .map(DefResolution::Global),
                    TSKind::variable_declaration_statement => self
                        .local_to_def(src)
                        .map(Local::from)
                        .map(DefResolution::Local),
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }

    pub fn find_def(&self, file_id: FileId, node: &tree_sitter::Node) -> Option<DefResolution> {
        let source = self.db.file_text(file_id);
        let parent = node.parent()?;
        if let Some(res) = self.find_name_def(file_id, node) {
            return res.into();
        }

        let mut container = node.parent()?;
        // If the node does not have a parent we are at the root, nothing to resolve.
        while !matches!(TSKind::from(container), TSKind::function_definition) {
            if let Some(candidate) = container.parent() {
                container = candidate;
            } else {
                break;
            }
        }
        match TSKind::from(container) {
            TSKind::function_definition => {
                self.function_node_to_def(file_id, container, parent, *node, source)
            }
            TSKind::source_file => self.source_node_to_def(file_id, *node, source),
            _ => todo!(),
        }
    }

    fn source_node_to_def(
        &self,
        file_id: FileId,
        node: tree_sitter::Node,
        source: Arc<str>,
    ) -> Option<DefResolution> {
        let def_map = self.db.file_def_map(file_id);
        let text = node.utf8_text(source.as_ref().as_bytes()).ok()?;
        match def_map.get_from_str(text)? {
            hir_def::FileDefId::FunctionId(id) => {
                DefResolution::Function(Function::from(id)).into()
            }
            hir_def::FileDefId::VariableId(id) => DefResolution::Global(Global::from(id)).into(),
            hir_def::FileDefId::EnumStructId(id) => {
                DefResolution::EnumStruct(EnumStruct::from(id)).into()
            }
        }
    }

    fn function_node_to_def(
        &self,
        file_id: FileId,
        container: tree_sitter::Node,
        parent: tree_sitter::Node,
        node: tree_sitter::Node,
        source: Arc<str>,
    ) -> Option<DefResolution> {
        let def_map = self.db.file_def_map(file_id);
        let text = node.utf8_text(source.as_ref().as_bytes()).ok()?;

        let parent_name = container
            .child_by_field_name("name")?
            .utf8_text(source.as_ref().as_bytes())
            .ok()?;
        let body_node = container.child_by_field_name("body")?;
        match TSKind::from(body_node) {
            TSKind::block => match def_map.get_from_str(parent_name)? {
                hir_def::FileDefId::FunctionId(id) => {
                    let def = hir_def::DefWithBodyId::FunctionId(id);
                    let offset = node.start_position();
                    if TSKind::field_access == TSKind::from(parent) {
                        let analyzer = SourceAnalyzer::new_for_body(
                            self.db,
                            def,
                            InFile::new(file_id, &body_node),
                            Some(offset),
                        );
                        let field = analyzer.resolve_field(self.db, &parent)?;
                        return Some(DefResolution::Field(field));
                    }

                    let analyzer = SourceAnalyzer::new_for_body_no_infer(
                        self.db,
                        def,
                        InFile::new(file_id, &body_node),
                        Some(offset),
                    );
                    let value_ns = analyzer.resolver.resolve_ident(text);
                    match value_ns? {
                        // TODO: Maybe hide the match logic in a function/macro?
                        ValueNs::LocalId(expr) => DefResolution::Local(Local::from(expr)).into(),
                        ValueNs::FunctionId(id) => {
                            DefResolution::Function(Function::from(id.value)).into()
                        }
                        ValueNs::GlobalId(id) => {
                            DefResolution::Global(Global::from(id.value)).into()
                        }
                        ValueNs::EnumStructId(id) => {
                            DefResolution::EnumStruct(EnumStruct::from(id.value)).into()
                        }
                    }
                }
                _ => unreachable!("Expected a function"),
            },
            _ => todo!("Handle non block body"),
        }
    }
}

// FIXME: This is a hacky way to implement the `ToDef` trait...
macro_rules! to_def_methods {
    ($(($def:path, $meth:ident)),* ,) => {$(
        pub fn $meth(&self, src: InFile<NodePtr>) -> Option<$def> {
            self.with_ctx(|ctx| ctx.$meth(src)).map(<$def>::from)
        }
    )*}
}

impl<'db> SemanticsImpl<'db> {
    fn new(db: &'db dyn HirDatabase) -> Self {
        SemanticsImpl {
            db,
            s2d_cache: Default::default(),
            cache: Default::default(),
        }
    }

    fn with_ctx<F: FnOnce(&mut SourceToDefCtx<'_, '_>) -> T, T>(&self, f: F) -> T {
        let mut cache = self.s2d_cache.borrow_mut();
        let mut ctx = SourceToDefCtx {
            db: self.db,
            cache: &mut cache,
        };
        f(&mut ctx)
    }

    to_def_methods![
        (crate::FunctionId, fn_to_def),
        (crate::EnumStructId, enum_struct_to_def),
        (hir_def::FieldId, field_to_def),
        (crate::GlobalId, global_to_def),
        (crate::Local, local_to_def),
    ];
}
