//! keys to be used with `DynMap`

use crate::{EnumStructId, FunctionId, GlobalId, NodePtr};

pub type Key<K, V> = crate::dyn_map::Key<K, V>;

pub const FUNCTION: Key<NodePtr, FunctionId> = Key::new();
pub const GLOBAL: Key<NodePtr, GlobalId> = Key::new();
pub const ENUM_STRUCT: Key<NodePtr, EnumStructId> = Key::new();
