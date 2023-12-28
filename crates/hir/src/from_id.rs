use hir_def::{DefWithBodyId, ExprId, FieldId};

use crate::Field;

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
