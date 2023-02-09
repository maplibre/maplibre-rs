use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub trait Resource: 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> Resource for T
where
    T: 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
pub struct Resources {
    resources: Vec<Box<dyn Resource>>,
    index: HashMap<TypeId, usize>,
}

impl Resources {
    pub fn init<R: Resource + Default>(&mut self) {
        self.insert(R::default());
    }

    pub fn insert<R: Resource>(&mut self, resource: R) {
        let index = self.resources.len();
        self.resources.push(Box::new(resource));
        self.index.insert(TypeId::of::<R>(), index);
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
            let x = self.resources[*index].as_ref().as_any();
            return Some(x.downcast_ref().unwrap()); // FIXME tcs: Unwrap
        }
        return None;
    }

    pub fn get_mut<R: Resource>(&mut self) -> Option<&mut R> {
        if let Some(index) = self.index.get(&TypeId::of::<R>()) {
            let x = self.resources[*index].as_mut().as_any_mut();
            return Some(x.downcast_mut().unwrap()); // FIXME tcs: Unwrap
        }
        return None;
    }

    pub fn query<'t, Q: ResourceQuery>(&'t self) -> Option<Q::Item<'t>> {
        Some(Q::query(self))
    }

    pub fn query_mut<'t, Q: ResourceQuery>(&'t mut self) -> Option<Q::Item<'t>> {
        Some(Q::query_mut(self))
    }
}

pub trait ResourceQuery {
    type Item<'r>;

    fn query<'r>(resources: &'r Resources) -> Self::Item<'r>;
    fn query_mut<'r>(resources: &'r mut Resources) -> Self::Item<'r>;
}

impl<'a, R: Resource> ResourceQuery for &'a R {
    type Item<'r> = &'r R;

    fn query<'r>(resources: &'r Resources) -> Self::Item<'r> {
        resources.get::<R>().unwrap() // FIXME tcs: Unwrap
    }

    fn query_mut<'r>(resources: &'r mut Resources) -> Self::Item<'r> {
        Self::query(resources)
    }
}

impl<'a, R: Resource> ResourceQuery for &'a mut R {
    type Item<'r> = &'r mut R;

    fn query<'r>(resources: &'r Resources) -> Self::Item<'r> {
        unsafe { &mut *(<&R as ResourceQuery>::query(resources) as *const R as *mut R) }
    }

    fn query_mut<'r>(resources: &'r mut Resources) -> Self::Item<'r> {
        resources.get_mut::<R>().unwrap() // FIXME tcs: Unwrap
    }
}

macro_rules! impl_resource_query {
    ($($param: ident),*) => {
        impl<$($param: ResourceQuery),*> ResourceQuery for ($($param,)*) {
            type Item<'r> =  ($($param::Item<'r>,)*);

            fn query<'r>(resources: &'r Resources) -> Self::Item<'r> {
                ($($param::query(resources),)*)
            }

            fn query_mut<'r>(resources: &'r mut Resources) -> Self::Item<'r> {
                ($($param::query(resources),)*)
            }
        }
    };
}

impl_resource_query!(R1);
impl_resource_query!(R1, R2);
impl_resource_query!(R1, R2, R3);
impl_resource_query!(R1, R2, R3, R4);
impl_resource_query!(R1, R2, R3, R4, R5);
impl_resource_query!(R1, R2, R3, R4, R5, R6);
