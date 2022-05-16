use std::cmp::Ordering;
use std::mem;

/// A wrapper type that enables ordering floats. This is a work around for the famous "rust float
/// ordering" problem. By using it, you acknowledge that sorting NaN is undefined according to spec.
/// This implementation treats NaN as the "smallest" float.
#[derive(Debug, Copy, Clone, PartialOrd)]
pub struct FloatOrd(pub f32);

impl PartialEq for FloatOrd {
    fn eq(&self, other: &Self) -> bool {
        if self.0.is_nan() && other.0.is_nan() {
            true
        } else {
            self.0 == other.0
        }
    }
}

impl Eq for FloatOrd {}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for FloatOrd {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or_else(|| {
            if self.0.is_nan() && !other.0.is_nan() {
                Ordering::Less
            } else if !self.0.is_nan() && other.0.is_nan() {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        })
    }
}

pub enum Eventually<T> {
    Initialized(T),
    Uninitialized,
}

impl<T> Eventually<T> {
    pub fn initialize(&mut self, f: impl FnOnce() -> T) {
        if let Eventually::Uninitialized = self {
            mem::replace(self, Eventually::Initialized(f()));
        }
    }

    pub fn take(&mut self) -> Eventually<T> {
        mem::replace(self, Eventually::Uninitialized)
    }
}

impl<T> Default for Eventually<T> {
    fn default() -> Self {
        Eventually::Uninitialized
    }
}
