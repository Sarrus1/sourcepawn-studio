use std::sync::Arc;

use fxhash::FxHashMap;
use la_arena::{Arena, ArenaMap, Idx};
use syntax::TSKind;
use tracing::field;

use crate::{
    hir::type_ref::TypeRef,
    item_tree::{EnumStructItemId, Name},
    src::{HasChildSource, HasSource},
    DefDatabase, EnumStructId, FunctionId, FunctionLoc, InFile, Intern, ItemTreeId, LocalFieldId,
    Lookup, NodePtr,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionData {
    pub name: Name,
    pub type_ref: Option<TypeRef>,
}

impl FunctionData {
    pub(crate) fn function_data_query(db: &dyn DefDatabase, id: FunctionId) -> Arc<FunctionData> {
        let loc = id.lookup(db).id;
        let item_tree = loc.tree_id().item_tree(db);
        let function = &item_tree[loc.value];
        let function_data = FunctionData {
            name: function.name.clone(),
            type_ref: function.ret_type.clone(),
        };

        Arc::new(function_data)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumStructData {
    pub name: Name,
    pub items: Arc<Arena<EnumStructItemData>>,
    pub items_map: Arc<FxHashMap<Name, Idx<EnumStructItemData>>>,
    // pub visibility: RawVisibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumStructItemData {
    Field(FieldData),
    Method(FunctionId),
}

/// A single field of a struct
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldData {
    pub name: Name,
    pub type_ref: TypeRef,
}

impl EnumStructData {
    pub(crate) fn enum_struct_data_query(
        db: &dyn DefDatabase,
        id: EnumStructId,
    ) -> Arc<EnumStructData> {
        let loc = id.lookup(db).id;
        let item_tree = loc.tree_id().item_tree(db);
        let enum_struct = &item_tree[loc.value];
        let mut items = Arena::new();
        let mut items_map = FxHashMap::default();
        enum_struct.items.iter().for_each(|e| match e {
            EnumStructItemId::Field(field_idx) => {
                let field = &item_tree[*field_idx];
                let field_data = EnumStructItemData::Field(FieldData {
                    name: field.name.clone(),
                    type_ref: field.type_ref.clone(),
                });
                let field_id = items.alloc(field_data);
                items_map.insert(field.name.clone(), field_id);
            }
            EnumStructItemId::Method(method_idx) => {
                let method = &item_tree[*method_idx];
                let fn_id = FunctionLoc {
                    container: id.into(),
                    id: ItemTreeId {
                        tree: loc.tree_id(),
                        value: *method_idx,
                    },
                }
                .intern(db);
                let method_id = items.alloc(EnumStructItemData::Method(fn_id));
                // FIXME: Not sure if we should intern like this...
                items_map.insert(method.name.clone(), method_id);
            } // TODO: Add diagnostic for duplicate enum struct items
        });
        let enum_struct_data = EnumStructData {
            name: enum_struct.name.clone(),
            items: Arc::new(items),
            items_map: Arc::new(items_map),
        };

        Arc::new(enum_struct_data)
    }

    pub fn item(&self, item: Idx<EnumStructItemData>) -> &EnumStructItemData {
        &self.items[item]
    }

    pub fn method(&self, item: Idx<EnumStructItemData>) -> Option<&FunctionId> {
        match &self.items[item] {
            EnumStructItemData::Field(_) => None,
            EnumStructItemData::Method(function_id) => Some(function_id),
        }
    }

    pub fn items(&self, name: &Name) -> Option<Idx<EnumStructItemData>> {
        self.items_map.get(name).cloned()
    }

    pub fn field_type(&self, field: Idx<EnumStructItemData>) -> Option<&TypeRef> {
        match &self.items[field] {
            EnumStructItemData::Field(field_data) => Some(&field_data.type_ref),
            EnumStructItemData::Method(_) => None,
        }
    }
}

impl HasChildSource<LocalFieldId> for EnumStructId {
    type Value = NodePtr;

    fn child_source(&self, db: &dyn DefDatabase) -> InFile<ArenaMap<LocalFieldId, Self::Value>> {
        let loc = self.lookup(db).id;
        let mut map = ArenaMap::default();
        let tree = db.parse(loc.file_id());
        // We use fields to get the Idx of the field, even if they are dropped at the end of the call.
        // The Idx will be the same when we rebuild the EnumStructData.
        // TODO: Is there a better way to do this?
        // FIXME: Why does it feel like we are doing this twice?
        let mut items = Arena::new();
        let enum_struct_node = loc.source(db, &tree).value;
        for child in enum_struct_node.children(&mut enum_struct_node.walk()) {
            match TSKind::from(child) {
                TSKind::enum_struct_field => {
                    let name_node = child.child_by_field_name("name").unwrap();
                    let name = Name::from_node(&name_node, &db.file_text(loc.file_id()));
                    let type_ref_node = child.child_by_field_name("type").unwrap();
                    let type_ref =
                        TypeRef::from_node(&type_ref_node, &db.file_text(loc.file_id())).unwrap();
                    let field = EnumStructItemData::Field(FieldData { name, type_ref });
                    map.insert(items.alloc(field), NodePtr::from(&child));
                }
                // TSKind::enum_struct_method => {
                //     let name_node = child.child_by_field_name("name").unwrap();
                //     let name = Name::from_node(&name_node, &db.file_text(loc.file_id()));
                //     let type_ref =
                //         child
                //             .child_by_field_name("returnType")
                //             .and_then(|type_ref_node| {
                //                 TypeRef::from_node(&type_ref_node, &db.file_text(loc.file_id()))
                //             });
                //     let method = EnumStructItemData::Method(FunctionData { name, type_ref });
                //     map.insert(items.alloc(method), NodePtr::from(&child));
                // }
                _ => (),
            }
        }
        InFile::new(loc.file_id(), map)
    }
}
