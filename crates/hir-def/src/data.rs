use std::sync::Arc;

use la_arena::{Arena, ArenaMap};
use syntax::TSKind;

use crate::{
    hir::type_ref::TypeRef,
    item_tree::Name,
    src::{HasChildSource, HasSource},
    DefDatabase, EnumStructId, FunctionId, InFile, LocalFieldId, Lookup, NodePtr,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionData {
    pub name: Name,
    pub type_ref: Option<TypeRef>,
}

impl FunctionData {
    pub(crate) fn function_data_query(db: &dyn DefDatabase, id: FunctionId) -> Arc<FunctionData> {
        let loc = id.lookup(db);
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
    pub fields: Arc<Arena<FieldData>>,
    // pub visibility: RawVisibility,
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
        let loc = id.lookup(db);
        let item_tree = loc.tree_id().item_tree(db);
        let enum_struct = &item_tree[loc.value];
        let mut fields = Arena::new();
        // FIXME: Do we need to clone here?
        enum_struct.fields.clone().for_each(|e| {
            let field = &item_tree[e];
            fields.alloc(FieldData {
                name: field.name.clone(),
                type_ref: field.type_ref.clone(),
            });
        });
        let enum_struct_data = EnumStructData {
            name: enum_struct.name.clone(),
            fields: Arc::new(fields),
        };

        Arc::new(enum_struct_data)
    }

    pub fn field(&self, name: &Name) -> Option<LocalFieldId> {
        // FIXME: linear search
        self.fields
            .iter()
            .find_map(|(id, data)| if data.name == *name { Some(id) } else { None })
    }

    pub fn field_type(&self, field: LocalFieldId) -> &TypeRef {
        &self.fields[field].type_ref
    }
}

impl HasChildSource<LocalFieldId> for EnumStructId {
    type Value = NodePtr;

    fn child_source(&self, db: &dyn DefDatabase) -> InFile<ArenaMap<LocalFieldId, Self::Value>> {
        let loc = self.lookup(db);
        let mut map = ArenaMap::default();
        let tree = db.parse(loc.file_id());
        // We use fields to get the Idx of the field, even if they are dropped at the end of the call.
        // The Idx will be the same when we rebuild the EnumStructData.
        // TODO: Is there a better way to do this?
        // FIXME: Why does it feel like we are doing this twice?
        let mut fields = Arena::new();
        let enum_struct_node = loc.source(db, &tree).value;
        for child in enum_struct_node.children(&mut enum_struct_node.walk()) {
            if TSKind::from(child) == TSKind::sym_enum_struct_field {
                let name_node = child.child_by_field_name("name").unwrap();
                let name = Name::from_node(&name_node, &db.file_text(loc.file_id()));
                let type_ref_node = child.child_by_field_name("type").unwrap();
                let type_ref =
                    TypeRef::from_node(&type_ref_node, &db.file_text(loc.file_id())).unwrap();
                let field = FieldData { name, type_ref };
                map.insert(fields.alloc(field), NodePtr::from(&child));
            }
        }
        InFile::new(loc.file_id(), map)
    }
}
