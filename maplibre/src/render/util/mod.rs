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

/// Wrapper around a resource which can be initialized or uninitialized.
/// Uninitialized resourced can be initialized by calling [`Eventually::initialize()`].
pub enum Eventually<T> {
    Initialized(T),
    Uninitialized,
}

pub trait HasChanged {
    type Criteria: Eq;

    fn has_changed(&self, criteria: &Self::Criteria) -> bool;
}

impl<T> HasChanged for Option<T>
where
    T: HasChanged,
{
    type Criteria = T::Criteria;

    fn has_changed(&self, criteria: &Self::Criteria) -> bool {
        match self {
            None => true,
            Some(value) => value.has_changed(criteria),
        }
    }
}

impl<'a, T> Eventually<T>
where
    T: HasChanged,
{
    #[tracing::instrument(name = "reinitialize", skip_all)]
    pub fn reinitialize(&mut self, f: impl FnOnce() -> T, criteria: &T::Criteria) {
        let should_replace = match &self {
            Eventually::Initialized(current) => {
                if current.has_changed(criteria) {
                    true
                } else {
                    false
                }
            }
            Eventually::Uninitialized => true,
        };

        if should_replace {
            mem::replace(self, Eventually::Initialized(f()));
        }
    }
}
impl<T> Eventually<T> {
    #[tracing::instrument(name = "initialize", skip_all)]
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
