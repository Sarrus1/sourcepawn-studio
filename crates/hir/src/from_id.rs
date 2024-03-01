use hir_def::{AttributeId, DefWithBodyId, ExprId, FieldId, FileDefId, PropertyId};

use crate::{Attribute, DefWithBody, Field, FileDef, Property};

macro_rules! from_id {
    ($(($id:path, $ty:path)),* $(,)?) => {$(
        impl From<$id> for $ty {
            fn from(id: $id) -> $ty {
                $ty { id }
            }
        }
        impl From<$ty> for $id {
            fn from(ty: $ty) -> $id {
                ty.id
            }
        }
    )*}
}

from_id![
    (hir_def::FunctionId, crate::Function),
    (hir_def::MacroId, crate::Macro),
    (hir_def::EnumStructId, crate::EnumStruct),
    (hir_def::MethodmapId, crate::Methodmap),
    (hir_def::GlobalId, crate::Global),
    (hir_def::EnumId, crate::Enum),
    (hir_def::VariantId, crate::Variant),
    (hir_def::TypedefId, crate::Typedef),
    (hir_def::TypesetId, crate::Typeset),
    (hir_def::FunctagId, crate::Functag),
];

impl From<(DefWithBodyId, ExprId)> for crate::Local {
    fn from((parent, expr_id): (DefWithBodyId, ExprId)) -> Self {
        crate::Local { parent, expr_id }
    }
}

impl From<Field> for FieldId {
    fn from(def: Field) -> Self {
        FieldId {
            parent: def.parent.into(),
            local_id: def.id,
        }
    }
}

impl From<FieldId> for Field {
    fn from(def: FieldId) -> Self {
        Field {
            parent: def.parent.into(),
            id: def.local_id,
        }
    }
}

impl From<Property> for PropertyId {
    fn from(def: Property) -> Self {
        PropertyId {
            parent: def.parent.into(),
            local_id: def.id,
        }
    }
}

impl From<PropertyId> for Property {
    fn from(def: PropertyId) -> Self {
        Property {
            parent: def.parent.into(),
            id: def.local_id,
        }
    }
}

impl From<AttributeId> for Attribute {
    fn from(id: AttributeId) -> Self {
        match id {
            AttributeId::FieldId(it) => Attribute::Field(it.into()),
            AttributeId::PropertyId(it) => Attribute::Property(it.into()),
        }
    }
}

impl From<DefWithBody> for DefWithBodyId {
    fn from(def: DefWithBody) -> Self {
        match def {
            DefWithBody::Function(it) => DefWithBodyId::FunctionId(it.id),
            DefWithBody::Typedef(it) => DefWithBodyId::TypedefId(it.id),
            DefWithBody::Functag(it) => DefWithBodyId::FunctagId(it.id),
        }
    }
}

impl From<DefWithBodyId> for DefWithBody {
    fn from(def: DefWithBodyId) -> Self {
        match def {
            DefWithBodyId::FunctionId(it) => DefWithBody::Function(it.into()),
            DefWithBodyId::TypedefId(it) => DefWithBody::Typedef(it.into()),
            DefWithBodyId::FunctagId(it) => DefWithBody::Functag(it.into()),
        }
    }
}

impl From<FileDefId> for FileDef {
    fn from(id: FileDefId) -> Self {
        match id {
            FileDefId::FunctionId(it) => FileDef::Function(it.into()),
            FileDefId::MacroId(it) => FileDef::Macro(it.into()),
            FileDefId::EnumStructId(it) => FileDef::EnumStruct(it.into()),
            FileDefId::MethodmapId(it) => FileDef::Methodmap(it.into()),
            FileDefId::GlobalId(it) => FileDef::Global(it.into()),
            FileDefId::EnumId(it) => FileDef::Enum(it.into()),
            FileDefId::VariantId(it) => FileDef::Variant(it.into()),
            FileDefId::TypedefId(it) => FileDef::Typedef(it.into()),
            FileDefId::TypesetId(it) => FileDef::Typeset(it.into()),
            FileDefId::FunctagId(it) => FileDef::Functag(it.into()),
        }
    }
}

impl From<FileDef> for FileDefId {
    fn from(id: FileDef) -> Self {
        match id {
            FileDef::Function(it) => FileDefId::FunctionId(it.into()),
            FileDef::Macro(it) => FileDefId::MacroId(it.into()),
            FileDef::EnumStruct(it) => FileDefId::EnumStructId(it.into()),
            FileDef::Methodmap(it) => FileDefId::MethodmapId(it.into()),
            FileDef::Global(it) => FileDefId::GlobalId(it.into()),
            FileDef::Enum(it) => FileDefId::EnumId(it.into()),
            FileDef::Variant(it) => FileDefId::VariantId(it.into()),
            FileDef::Typedef(it) => FileDefId::TypedefId(it.into()),
            FileDef::Typeset(it) => FileDefId::TypesetId(it.into()),
            FileDef::Functag(it) => FileDefId::FunctagId(it.into()),
        }
    }
}
