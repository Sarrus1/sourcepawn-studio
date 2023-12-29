use fxhash::FxHashMap;
use hir_def::{
    child_by_source::ChildBySource,
    dyn_map::{keys, DynMap, Key},
    DefWithBodyId, EnumStructId, ExprId, FieldId, FunctionId, GlobalId, InFile, Lookup, NodePtr,
};
use stdx::impl_from;
use syntax::TSKind;
use vfs::FileId;

use crate::db::HirDatabase;

pub(super) type SourceToDefCache = FxHashMap<(ChildContainer, FileId), DynMap>;

pub(super) struct SourceToDefCtx<'a, 'b> {
    pub(super) db: &'b dyn HirDatabase,
    pub(super) cache: &'a mut SourceToDefCache,
}

impl SourceToDefCtx<'_, '_> {
    pub(super) fn fn_to_def(&mut self, src: InFile<NodePtr>) -> Option<FunctionId> {
        self.to_def(src, keys::FUNCTION)
    }
    pub(super) fn enum_struct_to_def(&mut self, src: InFile<NodePtr>) -> Option<EnumStructId> {
        self.to_def(src, keys::ENUM_STRUCT)
    }
    pub(super) fn field_to_def(&mut self, src: InFile<NodePtr>) -> Option<FieldId> {
        self.to_def(src, keys::FIELD)
    }
    pub(super) fn global_to_def(&mut self, src: InFile<NodePtr>) -> Option<GlobalId> {
        self.to_def(src, keys::GLOBAL)
    }

    pub(super) fn local_to_def(&mut self, src: InFile<NodePtr>) -> Option<(DefWithBodyId, ExprId)> {
        let container = self.find_container(src.as_ref())?;
        match container {
            ChildContainer::DefWithBodyId(def) => {
                let (_, source_map) = self.db.body_with_source_map(def);
                source_map.node_ptr_expr(src.value).map(|expr| (def, expr))
            }
            _ => todo!(),
        }
    }

    fn to_def<ID: Copy + 'static>(
        &mut self,
        src: InFile<NodePtr>,
        key: Key<NodePtr, ID>,
    ) -> Option<ID> {
        self.dyn_map(src.as_ref())?[key].get(&src.value).copied()
    }

    fn dyn_map(&mut self, src: InFile<&NodePtr>) -> Option<&DynMap> {
        let container = self.find_container(src)?;
        Some(self.cache_for(container, src.file_id))
    }

    fn cache_for(&mut self, container: ChildContainer, file_id: FileId) -> &DynMap {
        let db = self.db;
        self.cache
            .entry((container, file_id))
            .or_insert_with(|| container.child_by_source(db, file_id))
    }

    pub(super) fn find_container(&mut self, src: InFile<&NodePtr>) -> Option<ChildContainer> {
        let tree = self.db.parse(src.file_id);
        let node = src.value.to_node(&tree);
        let mut container = node.parent()?;
        loop {
            match TSKind::from(container) {
                TSKind::sym_source_file => return Some(ChildContainer::FileId(src.file_id)),
                TSKind::sym_function_definition => {
                    let func =
                        self.fn_to_def(InFile::new(src.file_id, NodePtr::from(&container)))?;
                    return Some(ChildContainer::DefWithBodyId(DefWithBodyId::from(func)));
                }
                TSKind::sym_enum_struct => {
                    let enum_struct = self
                        .enum_struct_to_def(InFile::new(src.file_id, NodePtr::from(&container)))?;
                    return Some(ChildContainer::EnumStructId(enum_struct));
                }
                _ => {
                    if let Some(candidate) = container.parent() {
                        container = candidate;
                    } else {
                        return None;
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum ChildContainer {
    DefWithBodyId(DefWithBodyId),
    FileId(FileId),
    EnumStructId(EnumStructId),
}

impl_from! {
    DefWithBodyId,
    EnumStructId,
    FileId
    for ChildContainer
}

impl ChildContainer {
    fn child_by_source(self, db: &dyn HirDatabase, file_id: FileId) -> DynMap {
        let db = db.upcast();
        match self {
            ChildContainer::FileId(id) => id.child_by_source(db, file_id),
            ChildContainer::EnumStructId(id) => id.child_by_source(db, file_id),
            _ => todo!(),
        }
    }
}
