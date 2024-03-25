use la_arena::{Idx, RawIdx};
use vfs::FileId;

use crate::{
    data::{EnumStructItemData, MethodmapItemData, PropertyData},
    dyn_map::{keys, DynMap},
    src::HasChildSource,
    DefDatabase, EnumStructId, FieldId, FileDefId, FuncenumId, Lookup, MethodmapId, PropertyItem,
    TypesetId,
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
                    res[keys::FUNCTION].insert(node_ptr, *id);
                }
                FileDefId::MacroId(id) => {
                    let item = &item_tree[id.lookup(db).id];
                    let node_ptr = ast_id_map.get_raw(item.ast_id);
                    res[keys::MACRO].insert(node_ptr, *id);
                }
                FileDefId::GlobalId(id) => {
                    let item = &item_tree[id.lookup(db)];
                    let node_ptr = ast_id_map.get_raw(item.ast_id);
                    res[keys::GLOBAL].insert(node_ptr, *id);
                }
                FileDefId::EnumStructId(id) => {
                    let item = &item_tree[id.lookup(db).id];
                    let node_ptr = ast_id_map.get_raw(item.ast_id);
                    res[keys::ENUM_STRUCT].insert(node_ptr, *id);
                }
                FileDefId::MethodmapId(id) => {
                    let item = &item_tree[id.lookup(db).id];
                    let node_ptr = ast_id_map.get_raw(item.ast_id);
                    res[keys::METHODMAP].insert(node_ptr, *id);
                }
                FileDefId::EnumId(id) => {
                    let item = &item_tree[id.lookup(db).id];
                    let node_ptr = ast_id_map.get_raw(item.ast_id);
                    res[keys::ENUM].insert(node_ptr, *id);
                }
                FileDefId::VariantId(id) => {
                    let item = &item_tree[id.lookup(db).id];
                    let node_ptr = ast_id_map.get_raw(item.ast_id);
                    res[keys::ENUM_VARIANT].insert(node_ptr, *id);
                }
                FileDefId::TypedefId(id) => {
                    let item = &item_tree[id.lookup(db).id];
                    let node_ptr = ast_id_map.get_raw(item.ast_id);
                    res[keys::TYPEDEF].insert(node_ptr, *id);
                }
                FileDefId::TypesetId(id) => {
                    let item = &item_tree[id.lookup(db).id];
                    let node_ptr = ast_id_map.get_raw(item.ast_id);
                    res[keys::TYPESET].insert(node_ptr, *id);
                }
                FileDefId::FunctagId(id) => {
                    let item = &item_tree[id.lookup(db).id];
                    let node_ptr = ast_id_map.get_raw(item.ast_id);
                    res[keys::FUNCTAG].insert(node_ptr, *id);
                }
                FileDefId::FuncenumId(id) => {
                    let item = &item_tree[id.lookup(db).id];
                    let node_ptr = ast_id_map.get_raw(item.ast_id);
                    res[keys::FUNCENUM].insert(node_ptr, *id);
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
        // This is not ideal.
        // When we build the arena_map, we only push properties in it. Therefore, we need to keep track of the property
        // index because some methods may squeeze in between properties.
        // TODO: We should probably change the way we build the arena_map to include methods as well.
        let mut field_idx = 0u32;
        data.items.iter().for_each(|(idx, item)| match item {
            EnumStructItemData::Field(_) => {
                let field_id = FieldId {
                    parent: *self,
                    local_id: idx,
                };
                map[keys::FIELD].insert(
                    arena_map.value[Idx::from_raw(RawIdx::from_u32(field_idx))],
                    field_id,
                );
                field_idx += 1;
            }
            EnumStructItemData::Method(id) => {
                let item = &item_tree[id.lookup(db).id];
                let node_ptr = ast_id_map.get_raw(item.ast_id);
                map[keys::FUNCTION].insert(node_ptr, *id);
            }
        });
        for (local_id, source) in arena_map.value.iter() {
            let field_id = FieldId {
                parent: *self,
                local_id,
            };
            map[keys::FIELD].insert(*source, field_id);
        }
    }
}

impl ChildBySource for MethodmapId {
    fn child_by_source_to(&self, db: &dyn DefDatabase, map: &mut DynMap, file_id: FileId) {
        let data = db.methodmap_data(*self);
        let ast_id_map = db.ast_id_map(file_id);
        data.items.iter().for_each(|(_, item)| match item {
            MethodmapItemData::Property(PropertyData {
                id,
                getters_setters,
            }) => {
                let item_id = id.lookup(db).id;
                let item_tree = db.file_item_tree(item_id.file_id());
                let item = &item_tree[item_id];
                for fn_id in getters_setters.iter().map(PropertyItem::function_id) {
                    let item = &item_tree[fn_id.lookup(db).id];
                    let node_ptr = ast_id_map.get_raw(item.ast_id);
                    map[keys::FUNCTION].insert(node_ptr, fn_id);
                }
                let node_ptr = ast_id_map.get_raw(item.ast_id);
                map[keys::PROPERTY].insert(node_ptr, *id);
            }
            MethodmapItemData::Method(id)
            | MethodmapItemData::Constructor(id)
            | MethodmapItemData::Destructor(id) => {
                let item_id = id.lookup(db).id;
                let item_tree = db.file_item_tree(item_id.file_id());
                let item = &item_tree[id.lookup(db).id];
                let node_ptr = ast_id_map.get_raw(item.ast_id);
                map[keys::FUNCTION].insert(node_ptr, *id);
            }
        });
    }
}

impl ChildBySource for TypesetId {
    fn child_by_source_to(&self, db: &dyn DefDatabase, map: &mut DynMap, file_id: FileId) {
        let data = db.typeset_data(*self);
        let item_tree = db.file_item_tree(file_id);
        let ast_id_map = db.ast_id_map(file_id);
        data.typedefs.iter().for_each(|(_, id)| {
            let item = &item_tree[id.lookup(db).id];
            let node_ptr = ast_id_map.get_raw(item.ast_id);
            map[keys::TYPEDEF].insert(node_ptr, *id);
        });
    }
}

impl ChildBySource for FuncenumId {
    fn child_by_source_to(&self, db: &dyn DefDatabase, map: &mut DynMap, file_id: FileId) {
        let data = db.funcenum_data(*self);
        let item_tree = db.file_item_tree(file_id);
        let ast_id_map = db.ast_id_map(file_id);
        data.functags.iter().for_each(|(_, id)| {
            let item = &item_tree[id.lookup(db).id];
            let node_ptr = ast_id_map.get_raw(item.ast_id);
            map[keys::FUNCTAG].insert(node_ptr, *id);
        });
    }
}
