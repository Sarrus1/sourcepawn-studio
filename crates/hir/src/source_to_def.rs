use fxhash::FxHashMap;
use hir_def::{dyn_map::DynMap, DefWithBodyId, EnumStructId};
use stdx::impl_from;
use vfs::FileId;

use crate::db::HirDatabase;

pub(super) type SourceToDefCache = FxHashMap<(ChildContainer, FileId), DynMap>;

pub(super) struct SourceToDefCtx<'a, 'b> {
    pub(super) db: &'b dyn HirDatabase,
    pub(super) cache: &'a mut SourceToDefCache,
}

impl SourceToDefCtx<'_, '_> {}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum ChildContainer {
    DefWithBodyId(DefWithBodyId),
    EnumStructId(EnumStructId),
}

impl_from! {
    DefWithBodyId,
    EnumStructId
    for ChildContainer
}
