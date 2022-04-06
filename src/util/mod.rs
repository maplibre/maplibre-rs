//! Utils which are used internally

mod fps_meter;
pub mod math;

use crate::coords::WorldTileCoords;
pub use fps_meter::FPSMeter;
use std::ops::{Deref, DerefMut};

struct MinMaxBoundingBox {
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
    initialized: bool,
}

impl MinMaxBoundingBox {
    fn new() -> Self {
        Self {
            min_x: i32::MAX,
            min_y: i32::MAX,
            max_x: i32::MIN,
            max_y: i32::MIN,
            initialized: false,
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn update(&mut self, world_coords: &WorldTileCoords) {
        self.initialized = true;

        if world_coords.x < self.min_x {
            self.min_x = world_coords.x;
        }

        if world_coords.y < self.min_y {
            self.min_y = world_coords.y;
        }

        if world_coords.x > self.max_x {
            self.max_x = world_coords.x;
        }

        if world_coords.y > self.max_y {
            self.max_y = world_coords.y;
        }
    }
}

pub struct ChangeObserver<T> {
    inner: T,
    last_value: Option<T>,
}

impl<T> ChangeObserver<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: value,
            last_value: None,
        }
    }
}

impl<T> ChangeObserver<T>
where
    T: Clone + Eq,
{
    pub fn finished_observing(&mut self) {
        self.last_value = Some(self.inner.clone());
    }

    pub fn did_change(&self) -> bool {
        if let Some(last_value) = &self.last_value {
            if !last_value.eq(&self.inner) {
                true
            } else {
                false
            }
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
