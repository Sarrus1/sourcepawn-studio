use std::{ops, process::Command};

pub mod anymap;
pub mod hashable_hash_map;
pub mod macros;
pub mod panic_context;
pub mod process;
pub mod thread;

/// A [`std::process::Child`] wrapper that will kill the child on drop.
#[cfg_attr(not(target_arch = "wasm32"), repr(transparent))]
#[derive(Debug)]
pub struct JodChild(pub std::process::Child);

impl ops::Deref for JodChild {
    type Target = std::process::Child;
    fn deref(&self) -> &std::process::Child {
        &self.0
    }
}

impl ops::DerefMut for JodChild {
    fn deref_mut(&mut self) -> &mut std::process::Child {
        &mut self.0
    }
}

impl Drop for JodChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

impl JodChild {
    pub fn spawn(mut command: Command) -> std::io::Result<Self> {
        command.spawn().map(Self)
    }

    pub fn into_inner(self) -> std::process::Child {
        if cfg!(target_arch = "wasm32") {
            panic!("no processes on wasm");
        }
        // SAFETY: repr transparent, except on WASM
        unsafe { std::mem::transmute::<JodChild, std::process::Child>(self) }
    }
}
