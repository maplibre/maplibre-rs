//! Utils which are used internally

use std::ops::{Deref, DerefMut};

pub use fps_meter::FPSMeter;

mod fps_meter;
pub mod grid;
pub mod label;
pub mod math;

pub trait SignificantlyDifferent<Rhs: ?Sized = Self> {
    type Epsilon;

    /// This method tests for `self` and `other` values to be significantly different
    #[must_use]
    fn ne(&self, other: &Rhs, epsilon: Self::Epsilon) -> bool;
}

pub struct ChangeObserver<T> {
    inner: T,
    reference_value: Option<T>,
}

impl<T> ChangeObserver<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: value,
            reference_value: None,
        }
    }
}

impl<T> ChangeObserver<T>
where
    T: Clone + SignificantlyDifferent,
{
    pub fn update_reference(&mut self) {
        self.reference_value = Some(self.inner.clone());
    }

    pub fn did_change(&self, epsilon: T::Epsilon) -> bool {
        if let Some(reference_value) = &self.reference_value {
            reference_value.ne(&self.inner, epsilon)
        } else {
            true
        }
    }
}

impl<T> Default for ChangeObserver<T>
where
    T: Default,
{
    fn default() -> Self {
        ChangeObserver::new(T::default())
    }
}

impl<T> Deref for ChangeObserver<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for ChangeObserver<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
