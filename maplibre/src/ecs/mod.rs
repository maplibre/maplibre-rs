// FIXME: Rename to tcs

use std::ops::{Deref, DerefMut};

mod resources;

pub mod component;
pub mod system;
pub mod tiles;
pub mod world;

pub struct Mut<'t, T> {
    value: &'t mut T,
}

impl<'t, T> Deref for Mut<'t, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'t, T> DerefMut for Mut<'t, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

pub struct Ref<'t, T> {
    value: &'t T,
}

impl<'t, T> Deref for Ref<'t, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}
