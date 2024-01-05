use hir_def::{DefWithBodyId, ExprId, FieldId, FileDefId};

use crate::{DefWithBody, Field, FileDef};

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
    (hir_def::EnumStructId, crate::EnumStruct),
    (hir_def::GlobalId, crate::Global),
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

impl From<DefWithBody> for DefWithBodyId {
    fn from(def: DefWithBody) -> Self {
        match def {
            DefWithBody::Function(it) => DefWithBodyId::FunctionId(it.id),
        }
    }
}

impl From<DefWithBodyId> for DefWithBody {
    fn from(def: DefWithBodyId) -> Self {
        match def {
            DefWithBodyId::FunctionId(it) => DefWithBody::Function(it.into()),
        }
    }
}

impl From<FileDefId> for FileDef {
    fn from(id: FileDefId) -> Self {
        match id {
            FileDefId::FunctionId(it) => FileDef::Function(it.into()),
            FileDefId::EnumStructId(it) => FileDef::EnumStruct(it.into()),
            FileDefId::GlobalId(it) => FileDef::Global(it.into()),
        }
    }
}

impl From<FileDef> for FileDefId {
    fn from(id: FileDef) -> Self {
        match id {
            FileDef::Function(it) => FileDefId::FunctionId(it.into()),
            FileDef::EnumStruct(it) => FileDefId::EnumStructId(it.into()),
            FileDef::Global(it) => FileDefId::GlobalId(it.into()),
        }
    }
}
