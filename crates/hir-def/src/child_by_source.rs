use vfs::FileId;

use crate::{
    dyn_map::{
        keys::{ENUM_STRUCT, FIELD, FUNCTION, GLOBAL},
        DynMap,
    },
    src::HasChildSource,
    DefDatabase, EnumStructId, FieldId, FileDefId, Lookup,
};

pub trait ChildBySource {
    fn child_by_source(&self, db: &dyn DefDatabase, file_id: FileId) -> DynMap {
        let mut res = DynMap::default();
        self.child_by_source_to(db, &mut res, file_id);
        res
    }
    fn child_by_source_to(&self, db: &dyn DefDatabase, map: &mut DynMap, file_id: FileId);
}

impl ChildBySource for FileId {
    fn child_by_source_to(&self, db: &dyn DefDatabase, res: &mut DynMap, file_id: FileId) {
        let item_tree = db.file_item_tree(file_id);
        let ast_id_map = db.ast_id_map(file_id);
        let def_map = db.file_def_map(file_id);
        for id in def_map.declarations() {
            match id {
                FileDefId::FunctionId(id) => {
                    // FIXME: Maybe the lookup can be removed and we can just use the id directly?
                    let item = &item_tree[id.lookup(db)];
                    let node_ptr = ast_id_map.get_raw(item.ast_id);
                    res[FUNCTION].insert(node_ptr, *id);
                }
                FileDefId::VariableId(id) => {
                    let item = &item_tree[id.lookup(db)];
                    let node_ptr = ast_id_map.get_raw(item.ast_id);
                    res[GLOBAL].insert(node_ptr, *id);
                }
                FileDefId::EnumStructId(id) => {
                    let item = &item_tree[id.lookup(db)];
                    let node_ptr = ast_id_map.get_raw(item.ast_id);
                    res[ENUM_STRUCT].insert(node_ptr, *id);
                }
            }
        }
    }
}

impl ChildBySource for EnumStructId {
    fn child_by_source_to(&self, db: &dyn DefDatabase, map: &mut DynMap, _file_id: FileId) {
        let arena_map = self.child_source(db);
        for (local_id, source) in arena_map.value.iter() {
            let field_id = FieldId {
                parent: *self,
                local_id,
            };
            map[FIELD].insert(*source, field_id);
        }
    }
}
