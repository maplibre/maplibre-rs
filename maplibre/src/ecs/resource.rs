use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use downcast_rs::Downcast;

pub trait Resource: 'static {}

impl<T> Resource for T where T: 'static {}

#[derive(Default)]
pub struct Resources {
    resources: Vec<Box<dyn Resource>>,
    index: HashMap<TypeId, usize>,
}

impl Resources {
    pub fn insert<R: Resource>(&mut self, resource: R) {
        self.resources.push(Box::new(resource))
    }

    pub fn remove<R: Resource>(&mut self) {
        if let Some(index) = self.index.get(&TypeId::of::<R>()) {
            self.resources.swap_remove(*index);
            let moved = &self.resources[*index];
            self.index.insert(moved.type_id(), *index);
        }
    }

    pub fn get<R: Resource>(&self) -> Option<&R> {
        if let Some(index) = self.index.get(&TypeId::of::<R>()) {
            return Some(self.resources[*index].as_any().downcast_ref().unwrap());
        }
        return None;
    }

    pub fn get_mut<R: Resource>(&mut self) -> Option<&mut R> {
        if let Some(index) = self.index.get(&TypeId::of::<R>()) {
            return Some(self.resources[*index].as_any_mut().downcast_mut().unwrap());
        }
        return None;
    }
}
