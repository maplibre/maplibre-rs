use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::ecs::{Mut, Ref};

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
            return Some(x.downcast_ref().unwrap());
        }
        return None;
    }

    pub fn get_mut<R: Resource>(&mut self) -> Option<&mut R> {
        if let Some(index) = self.index.get(&TypeId::of::<R>()) {
            let x = self.resources[*index].as_mut().as_any_mut();
            return Some(x.downcast_mut().unwrap());
        }
        return None;
    }

    pub unsafe fn unsafe_get_mut<R: Resource>(&mut self) -> &mut R {
        let i = self.index.get(&TypeId::of::<R>()).unwrap();
        let resources = self.resources.as_mut_ptr();
        return (&mut *resources.offset(*i as isize))
            .as_mut()
            .as_any_mut()
            .downcast_mut()
            .unwrap();
    }

    // FIXME: Do this properly
    pub fn collect_mut3<R1: Resource, R2: Resource, R3: Resource>(
        &mut self,
    ) -> Option<(&mut R1, &mut R2, &mut R3)> {
        let i1 = self.index.get(&TypeId::of::<R1>())?;
        let i2 = self.index.get(&TypeId::of::<R2>())?;
        let i3 = self.index.get(&TypeId::of::<R3>())?;

        unsafe {
            let resources = self.resources.as_mut_ptr();

            Some((
                (&mut *resources.offset(*i1 as isize))
                    .as_mut()
                    .as_any_mut()
                    .downcast_mut()
                    .unwrap(),
                (&mut *resources.offset(*i2 as isize))
                    .as_mut()
                    .as_any_mut()
                    .downcast_mut()
                    .unwrap(),
                (&mut *resources.offset(*i3 as isize))
                    .as_mut()
                    .as_any_mut()
                    .downcast_mut()
                    .unwrap(),
            ))
        }
    }

    // FIXME: Do this properly
    pub fn collect_mut6<
        R1: Resource,
        R2: Resource,
        R3: Resource,
        R4: Resource,
        R5: Resource,
        R6: Resource,
    >(
        &mut self,
    ) -> Option<(&mut R1, &mut R2, &mut R3, &mut R4, &mut R5, &mut R6)> {
        let i1 = self.index.get(&TypeId::of::<R1>())?;
        let i2 = self.index.get(&TypeId::of::<R2>())?;
        let i3 = self.index.get(&TypeId::of::<R3>())?;
        let i4 = self.index.get(&TypeId::of::<R4>())?;
        let i5 = self.index.get(&TypeId::of::<R5>())?;
        let i6 = self.index.get(&TypeId::of::<R6>())?;

        unsafe {
            let resources = self.resources.as_mut_ptr();
            Some((
                (&mut *resources.offset(*i1 as isize))
                    .as_mut()
                    .as_any_mut()
                    .downcast_mut()
                    .unwrap(),
                (&mut *resources.offset(*i2 as isize))
                    .as_mut()
                    .as_any_mut()
                    .downcast_mut()
                    .unwrap(),
                (&mut *resources.offset(*i3 as isize))
                    .as_mut()
                    .as_any_mut()
                    .downcast_mut()
                    .unwrap(),
                (&mut *resources.offset(*i4 as isize))
                    .as_mut()
                    .as_any_mut()
                    .downcast_mut()
                    .unwrap(),
                (&mut *resources.offset(*i5 as isize))
                    .as_mut()
                    .as_any_mut()
                    .downcast_mut()
                    .unwrap(),
                (&mut *resources.offset(*i6 as isize))
                    .as_mut()
                    .as_any_mut()
                    .downcast_mut()
                    .unwrap(),
            ))
        }
    }

    pub fn collect_mut4<R1: Resource, R2: Resource, R3: Resource, R4: Resource>(
        &mut self,
    ) -> Option<(&mut R1, &mut R2, &mut R3, &mut R4)> {
        let i1 = self.index.get(&TypeId::of::<R1>())?;
        let i2 = self.index.get(&TypeId::of::<R2>())?;
        let i3 = self.index.get(&TypeId::of::<R3>())?;
        let i4 = self.index.get(&TypeId::of::<R4>())?;

        unsafe {
            let resources = self.resources.as_mut_ptr();
            Some((
                (&mut *resources.offset(*i1 as isize))
                    .as_mut()
                    .as_any_mut()
                    .downcast_mut()
                    .unwrap(),
                (&mut *resources.offset(*i2 as isize))
                    .as_mut()
                    .as_any_mut()
                    .downcast_mut()
                    .unwrap(),
                (&mut *resources.offset(*i3 as isize))
                    .as_mut()
                    .as_any_mut()
                    .downcast_mut()
                    .unwrap(),
                (&mut *resources.offset(*i4 as isize))
                    .as_mut()
                    .as_any_mut()
                    .downcast_mut()
                    .unwrap(),
            ))
        }
    }
    pub fn collect_mut2<R1: Resource, R2: Resource>(&mut self) -> Option<(&mut R1, &mut R2)> {
        let i1 = self.index.get(&TypeId::of::<R1>())?;
        let i2 = self.index.get(&TypeId::of::<R2>())?;

        unsafe {
            let resources = self.resources.as_mut_ptr();
            Some((
                (&mut *resources.offset(*i1 as isize))
                    .as_mut()
                    .as_any_mut()
                    .downcast_mut()
                    .unwrap(),
                (&mut *resources.offset(*i2 as isize))
                    .as_mut()
                    .as_any_mut()
                    .downcast_mut()
                    .unwrap(),
            ))
        }
    }
}

trait ResourceQuery {
    type Item<'a>;

    fn get_resource<'a>(resources: &'a Resources) -> Self::Item<'a>;
    fn get_resource_mut<'a>(resources: &'a mut Resources) -> Self::Item<'a>;
}

impl<'r, R: Resource> ResourceQuery for Ref<'r, R> {
    type Item<'a> = Ref<'a, R>;

    fn get_resource<'a>(resources: &'a Resources) -> Self::Item<'a> {
        Ref {
            value: resources.get::<R>().unwrap(),
        }
    }

    fn get_resource_mut<'a>(resources: &'a mut Resources) -> Self::Item<'a> {
        Self::get_resource(resources)
    }
}

impl<'r, R: Resource> ResourceQuery for Mut<'r, R> {
    type Item<'a> = Mut<'a, R>;

    fn get_resource<'a>(resources: &'a Resources) -> Self::Item<'a> {
        panic!("provide an inmutable World to query inmutable")
    }

    fn get_resource_mut<'a>(resources: &'a mut Resources) -> Self::Item<'a> {
        Mut {
            value: unsafe { resources.unsafe_get_mut::<R>() },
        }
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
}

macro_rules! impl_resource_query {
    ($($param: ident),*) => {
        impl<$($param: ResourceQuery),*> ResourceQuery for ($($param,)*) {
            type Item<'a> =  ($($param::Item<'a>,)*);

            fn get_resource<'a>(world: &'a World) -> Self::Item<'a> {
                ($($param::get_resource(world),)*)
            }

            fn get_resource_mut<'a>(world: &'a mut World) -> Self::Item<'a> {
                ($($param::get_resource_mut(world),)*)
            }
        }
    };
}

/*impl_system_function!(R1, R2);
impl_system_function!(R1, R2, R3);
impl_system_function!(R1, R2, R3, R4);
impl_system_function!(R1, R2, R3, R4, R5);
impl_system_function!(R1, R2, R3, R4, R5, R6);*/
