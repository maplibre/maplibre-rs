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
