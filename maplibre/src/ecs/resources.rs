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

    pub fn query_mut<'t, Q: ResourceQuery>(&'t mut self) -> Option<Q::Item<'t>> {
        Some(Q::get_resource_mut(self))
    }

    // FIXME tcs
    unsafe fn unsafe_get_mut<R: Resource>(&self) -> &mut R {
        let i = self.index.get(&TypeId::of::<R>()).unwrap();
        let resources = self.resources.as_ptr();

        (&mut *(resources.offset(*i as isize) as *mut Box<dyn Resource>))
            .as_mut()
            .as_any_mut()
            .downcast_mut()
            .unwrap()
    }
}

pub trait ResourceQuery {
    type Item<'a>;

    fn get_resource<'a>(resources: &'a Resources) -> Self::Item<'a>;
    fn get_resource_mut<'a>(resources: &'a mut Resources) -> Self::Item<'a>;

    unsafe fn unsafe_get_mut<'a>(resources: &'a Resources) -> Self::Item<'a>;
}

impl<'r, R: Resource> ResourceQuery for &'r R {
    type Item<'a> = &'a R;

    fn get_resource<'a>(resources: &'a Resources) -> Self::Item<'a> {
        resources.get::<R>().unwrap() // FIXME tcs: Unwrap
    }

    fn get_resource_mut<'a>(resources: &'a mut Resources) -> Self::Item<'a> {
        Self::get_resource(resources)
    }

    unsafe fn unsafe_get_mut<'a>(resources: &'a Resources) -> Self::Item<'a> {
        resources.unsafe_get_mut::<R>()
    }
}

impl<'r, R: Resource> ResourceQuery for &'r mut R {
    type Item<'a> = &'a mut R;

    fn get_resource<'a>(resources: &'a Resources) -> Self::Item<'a> {
        panic!("provide an inmutable World to query inmutable")
    }

    fn get_resource_mut<'a>(resources: &'a mut Resources) -> Self::Item<'a> {
        resources.get_mut::<R>().unwrap() // FIXME tcs: Unwrap
    }

    unsafe fn unsafe_get_mut<'a>(resources: &'a Resources) -> Self::Item<'a> {
        resources.unsafe_get_mut::<R>()
    }
}

impl<RQ1: ResourceQuery> ResourceQuery for (RQ1,) {
    type Item<'a> = (RQ1::Item<'a>,);

    fn get_resource<'a>(resources: &'a Resources) -> Self::Item<'a> {
        (RQ1::get_resource(resources),)
    }

    fn get_resource_mut<'a>(resources: &'a mut Resources) -> Self::Item<'a> {
        (RQ1::get_resource_mut(resources),)
    }

    unsafe fn unsafe_get_mut<'a>(resources: &'a Resources) -> Self::Item<'a> {
        todo!()
    }
}

macro_rules! impl_resource_query {
    ($($param: ident),*) => {
        impl<$($param: ResourceQuery),*> ResourceQuery for ($($param,)*) {
            type Item<'a> =  ($($param::Item<'a>,)*);

            fn get_resource<'a>(resources: &'a Resources) -> Self::Item<'a> {
                ($($param::get_resource(resources),)*)
            }

            fn get_resource_mut<'a>(resources: &'a mut Resources) -> Self::Item<'a> {
                unsafe {
                    ($($param::unsafe_get_mut(resources),)*)
                }
            }
            unsafe fn unsafe_get_mut<'a>(resources: &'a Resources) -> Self::Item<'a> {
                todo!()
            }
        }
    };
}

impl_resource_query!(R1, R2);
impl_resource_query!(R1, R2, R3);
impl_resource_query!(R1, R2, R3, R4);
impl_resource_query!(R1, R2, R3, R4, R5);
impl_resource_query!(R1, R2, R3, R4, R5, R6);
