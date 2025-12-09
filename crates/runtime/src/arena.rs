#![forbid(unsafe_code)]

/// Simple bump arena for block/function-scoped allocations.
#[derive(Debug, Clone)]
pub struct Arena {
    buf: Vec<u8>,
    off: usize,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ArenaError {
    #[error("arena out of capacity: requested {requested} remaining {remaining}")]
    OutOfCapacity { requested: usize, remaining: usize },
}

impl Arena {
    /// Create a new arena with a fixed capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            buf: vec![0; cap],
            off: 0,
        }
    }

    /// Total capacity in bytes.
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    /// Remaining free space in bytes.
    pub fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.off)
    }

    /// Allocate `size` bytes and return a mutable view.
    pub fn alloc(&mut self, size: usize) -> Result<&mut [u8], ArenaError> {
        if size > self.remaining() {
            return Err(ArenaError::OutOfCapacity {
                requested: size,
                remaining: self.remaining(),
            });
        }

        let start = self.off;
        let end = start + size;
        self.off = end;
        Ok(&mut self.buf[start..end])
    }

    /// Reset the arena to an empty state; data remains but is considered invalid.
    pub fn reset(&mut self) {
        self.off = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_reset() {
        let mut arena = Arena::with_capacity(16);
        let slice = arena.alloc(8).expect("alloc 8");
        slice.copy_from_slice(&[1u8; 8]);
        assert_eq!(arena.remaining(), 8);

        arena.reset();
        assert_eq!(arena.remaining(), 16);

        let slice2 = arena.alloc(16).expect("alloc after reset");
        assert_eq!(slice2.len(), 16);
    }

    #[test]
    fn overflow_errors() {
        let mut arena = Arena::with_capacity(4);
        let err = arena.alloc(8).expect_err("should overflow");
        assert_eq!(
            err,
            ArenaError::OutOfCapacity {
                requested: 8,
                remaining: 4
            }
        );
    }
}
