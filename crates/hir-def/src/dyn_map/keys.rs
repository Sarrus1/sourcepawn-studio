//! keys to be used with `DynMap`

use crate::{FunctionId, NodePtr};

pub type Key<K, V> = crate::dyn_map::Key<K, V>;

pub const FUNCTION: Key<NodePtr, FunctionId> = Key::new();
