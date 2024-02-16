use vfs::FileId;

use crate::{
    data::EnumStructItemData,
    dyn_map::{
        keys::{ENUM_STRUCT, FIELD, FUNCTION, GLOBAL, MACRO},
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
                    let item = &item_tree[id.lookup(db).id];
                    let node_ptr = ast_id_map.get_raw(item.ast_id);
                    res[FUNCTION].insert(node_ptr, *id);
                }
                FileDefId::MacroId(id) => {
                    let item = &item_tree[id.lookup(db).id];
                    let node_ptr = ast_id_map.get_raw(item.ast_id);
                    res[MACRO].insert(node_ptr, *id);
                }
                FileDefId::GlobalId(id) => {
                    let item = &item_tree[id.lookup(db)];
                    let node_ptr = ast_id_map.get_raw(item.ast_id);
                    res[GLOBAL].insert(node_ptr, *id);
                }
                FileDefId::EnumStructId(id) => {
                    let item = &item_tree[id.lookup(db).id];
                    let node_ptr = ast_id_map.get_raw(item.ast_id);
                    res[ENUM_STRUCT].insert(node_ptr, *id);
                }
            }
        }
    }
}

impl ChildBySource for EnumStructId {
    fn child_by_source_to(&self, db: &dyn DefDatabase, map: &mut DynMap, file_id: FileId) {
        let arena_map = self.child_source(db);
        let data = db.enum_struct_data(*self);
        let item_tree = db.file_item_tree(file_id);
        let ast_id_map = db.ast_id_map(file_id);
        data.items.iter().for_each(|(idx, item)| match item {
            EnumStructItemData::Field(_) => {
                let field_id = FieldId {
                    parent: *self,
                    local_id: idx,
                };
                map[FIELD].insert(arena_map.value[idx], field_id);
            }
            EnumStructItemData::Method(id) => {
                let item = &item_tree[id.lookup(db).id];
                let node_ptr = ast_id_map.get_raw(item.ast_id);
                map[FUNCTION].insert(node_ptr, *id);
            }
        });
        for (local_id, source) in arena_map.value.iter() {
            let field_id = FieldId {
                parent: *self,
                local_id,
            };
            map[FIELD].insert(*source, field_id);
        }
    }
}
