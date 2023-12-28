use std::{cell::RefCell, fmt, ops};

use base_db::Tree;
use fxhash::FxHashMap;
use hir_def::{resolver::ValueNs, FunctionId, InFile, NodePtr};
use syntax::TSKind;
use vfs::FileId;

use crate::{
    db::HirDatabase,
    source_analyzer::SourceAnalyzer,
    source_to_def::{SourceToDefCache, SourceToDefCtx},
    DefResolution, EnumStruct, Function, Global, Local,
};

/// Primary API to get semantic information, like types, from syntax trees.
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

    pub fn find_def(&self, file_id: FileId, node: &tree_sitter::Node) -> Option<DefResolution> {
        let source = self.db.file_text(file_id);
        let def_map = self.db.file_def_map(file_id);
        let text = node.utf8_text(source.as_ref().as_bytes()).ok()?;
        let parent = node.parent()?;
        let src = InFile::new(file_id, NodePtr::from(&parent));
        let res = match TSKind::from(parent) {
            TSKind::sym_function_definition => self
                .fn_to_def(src)
                .map(Function::from)
                .map(DefResolution::Function),
            TSKind::sym_enum_struct => self
                .enum_struct_to_def(src)
                .map(EnumStruct::from)
                .map(DefResolution::EnumStruct),
            TSKind::sym_variable_declaration => {
                if let Some(grand_parent) = parent.parent() {
                    if TSKind::from(&grand_parent) == TSKind::sym_global_variable_declaration {
                        if let Some(sub_node) = parent.child_by_field_name("name") {
                            if sub_node == *node {
                                self.global_to_def(src)
                                    .map(Global::from)
                                    .map(DefResolution::Global)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        };
        if res.is_some() {
            return res;
        }

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

// FIXME: This is a hacky way to implement the `ToDef` trait...
macro_rules! to_def_methods {
    ($(($def:path, $meth:ident)),* ,) => {$(
        pub fn $meth(&self, src: InFile<NodePtr>) -> Option<$def> {
            self.with_ctx(|ctx| ctx.$meth(src))
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
        (crate::GlobalId, global_to_def),
    ];
}
