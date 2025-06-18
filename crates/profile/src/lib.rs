//! A collection of tools for profiling rust-analyzer.

#![warn(
    rust_2018_idioms,
    unused_lifetimes,
    semicolon_in_expressions_from_macros
)]

mod memory_usage;

use std::cell::RefCell;

pub use crate::memory_usage::{Bytes, MemoryUsage};

pub use countme;
/// Include `_c: Count<Self>` field in important structs to count them.
///
/// To view the counts, run with `RA_COUNT=1`. The overhead of disabled count is
/// almost zero.
pub use countme::Count;

thread_local!(static IN_SCOPE: RefCell<bool> = const { RefCell::new(false) });

/// Allows to check if the current code is within some dynamic scope, can be
/// useful during debugging to figure out why a function is called.
pub struct Scope {
    prev: bool,
}

impl Scope {
    #[must_use]
    pub fn enter() -> Scope {
        let prev = IN_SCOPE.with(|slot| std::mem::replace(&mut *slot.borrow_mut(), true));
        Scope { prev }
    }
    pub fn is_active() -> bool {
        IN_SCOPE.with(|slot| *slot.borrow())
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        IN_SCOPE.with(|slot| *slot.borrow_mut() = self.prev);
    }
}
