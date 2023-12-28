use std::{any::Any, sync::Arc};

use la_arena::Arena;

use crate::{
    hir::type_ref::TypeRef, item_tree::Name, DefDatabase, EnumStructId, FunctionId, LocalFieldId,
    Lookup,
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
    pub tree_id: la_arena::Idx<crate::item_tree::Field>, //FIXME: This is a hack
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
                tree_id: e,
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
