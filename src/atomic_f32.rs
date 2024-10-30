use std::sync::atomic::{AtomicU32, Ordering};

/// A thread-safe, atomic wrapper around an `f32` value.
///
/// This struct allows for atomic operations on floating-point numbers
/// by internally representing the `f32` value as a `u32` using bitwise operations.
pub struct AtomicF32 {
    storage: AtomicU32,
}

impl AtomicF32 {
    /// Creates a new `AtomicF32` initialized to zero.
    pub const fn new() -> Self {
        Self {
            storage: AtomicU32::new(0),
        }
    }

    /// Atomically stores an `f32` value.
    ///
    /// # Arguments
    ///
    /// * `value` - The `f32` value to store atomically.
    pub fn store(&self, value: f32) {
        self.storage.store(value.to_bits(), Ordering::Relaxed);
    }

    /// Atomically loads the current `f32` value.
    ///
    /// # Returns
    ///
    /// The current `f32` value stored atomically.
    pub fn load(&self) -> f32 {
        f32::from_bits(self.storage.load(Ordering::Relaxed))
    }

    /// Resets the `AtomicF32` value to zero atomically.
    pub fn reset(&self) {
        self.storage.store(0, Ordering::Relaxed);
    }
}
