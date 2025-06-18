//! Like [`std::time::Instant`], but for memory.
//!
//! Measures the total size of all currently allocated objects.
use std::fmt;

#[derive(Copy, Clone)]
pub struct MemoryUsage {
    pub allocated: Bytes,
}

impl fmt::Display for MemoryUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.allocated.fmt(f)
    }
}

impl std::ops::Sub for MemoryUsage {
    type Output = MemoryUsage;
    fn sub(self, rhs: MemoryUsage) -> MemoryUsage {
        MemoryUsage {
            allocated: self.allocated - rhs.allocated,
        }
    }
}

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Bytes(isize);

impl Bytes {
    pub fn new(bytes: isize) -> Bytes {
        Bytes(bytes)
    }
}

impl Bytes {
    pub fn megabytes(self) -> isize {
        self.0 / 1024 / 1024
    }
}

impl fmt::Display for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.0;
        let mut value = bytes;
        let mut suffix = "b";
        if value.abs() > 4096 {
            value /= 1024;
            suffix = "kb";
            if value.abs() > 4096 {
                value /= 1024;
                suffix = "mb";
            }
        }
        f.pad(&format!("{value}{suffix}"))
    }
}

impl std::ops::AddAssign<usize> for Bytes {
    fn add_assign(&mut self, x: usize) {
        self.0 += x as isize;
    }
}

impl std::ops::Sub for Bytes {
    type Output = Bytes;
    fn sub(self, rhs: Bytes) -> Bytes {
        Bytes(self.0 - rhs.0)
    }
}
