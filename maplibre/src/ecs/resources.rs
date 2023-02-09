use std::{any, any::TypeId, collections::HashMap};

use downcast_rs::{impl_downcast, Downcast};

use crate::ecs::{EphemeralQueryState, GlobalQueryState, QueryState};

pub trait Resource: Downcast + 'static {}
impl_downcast!(Resource);

impl<T> Resource for T where T: 'static {}

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

    pub fn get<R: Resource>(&self) -> Option<&R> {
        if let Some(index) = self.index.get(&TypeId::of::<R>()) {
            return Some(self.resources[*index].downcast_ref().unwrap()); // FIXME tcs: Unwrap
        }
        return None;
    }

    pub fn get_mut<R: Resource>(&mut self) -> Option<&mut R> {
        if let Some(index) = self.index.get(&TypeId::of::<R>()) {
            return Some(self.resources[*index].downcast_mut().unwrap()); // FIXME tcs: Unwrap
        }
        return None;
    }

    pub fn query<'r, Q: ResourceQuery>(&'r self) -> Option<Q::Item<'r>> {
        let mut global_state = GlobalQueryState::default();
        let mut state = <Q::State<'_> as QueryState>::create(&mut global_state);
        Q::query(&self, state)
    }

    pub fn query_mut<'r, Q: ResourceQueryMut>(&'r mut self) -> Option<Q::MutItem<'r>> {
        let mut global_state = GlobalQueryState::default();
        let mut state = <Q::State<'_> as QueryState>::create(&mut global_state);
        Q::query_mut(self, state)
    }
}

// ResourceQuery

pub trait ResourceQuery {
    type Item<'r>;

    type State<'s>: QueryState<'s>;

    fn query<'r, 's>(resources: &'r Resources, state: Self::State<'s>) -> Option<Self::Item<'r>>;
}

impl<'a, R: Resource> ResourceQuery for &'a R {
    type Item<'r> = &'r R;
    type State<'s> = EphemeralQueryState<'s>;

    fn query<'r, 's>(resources: &'r Resources, state: Self::State<'s>) -> Option<Self::Item<'r>> {
        resources.get::<R>()
    }
}

// ResourceQueryMut

pub trait ResourceQueryMut {
    type MutItem<'r>;

    type State<'s>: QueryState<'s>;

    fn query_mut<'r, 's>(
        resources: &'r mut Resources,
        state: Self::State<'s>,
    ) -> Option<Self::MutItem<'r>>;
}

impl<'a, R: Resource> ResourceQueryMut for &'a R {
    type MutItem<'r> = &'r R;
    type State<'s> = EphemeralQueryState<'s>;

    fn query_mut<'r, 's>(
        resources: &'r mut Resources,
        state: Self::State<'s>,
    ) -> Option<Self::MutItem<'r>> {
        <&R as ResourceQuery>::query(resources, state)
    }
}

impl<'a, R: Resource> ResourceQueryMut for &'a mut R {
    type MutItem<'r> = &'r mut R;
    type State<'s> = EphemeralQueryState<'s>;

    fn query_mut<'r, 's>(
        resources: &'r mut Resources,
        state: Self::State<'s>,
    ) -> Option<Self::MutItem<'r>> {
        resources.get_mut::<R>()
    }
}

// ResourceQueryUnsafe

pub trait ResourceQueryUnsafe: ResourceQueryMut {
    unsafe fn query_unsafe<'r, 's>(
        resources: &'r Resources,
        state: Self::State<'s>,
    ) -> Option<Self::MutItem<'r>>;
}

impl<'a, R: Resource> ResourceQueryUnsafe for &'a R {
    unsafe fn query_unsafe<'r, 's>(
        resources: &'r Resources,
        state: Self::State<'s>,
    ) -> Option<Self::MutItem<'r>> {
        <&R as ResourceQuery>::query(resources, state)
    }
}

impl<'a, R: Resource> ResourceQueryUnsafe for &'a mut R {
    /// SAFETY: Safe if tiles is borrowed mutably.
    // FIXME: tcs: check if really safe
    unsafe fn query_unsafe<'r, 's>(
        resources: &'r Resources,
        mut state: Self::State<'s>,
    ) -> Option<Self::MutItem<'r>> {
        let id = TypeId::of::<R>();
        let borrowed = &mut state.state.mutably_borrowed;

        if borrowed.contains(&id) {
            panic!(
                "tried to borrow an {} more than once mutably",
                any::type_name::<R>()
            )
        }

        borrowed.insert(id);

        let result = <&R as ResourceQuery>::query(resources, state)?;
        Some(&mut *(result as *const R as *mut R))
    }
}

// Lift to tuples

macro_rules! impl_resource_query {
    ($($param: ident),*) => {
        impl<$($param: ResourceQuery),*> ResourceQuery for ($($param,)*) {
            type Item<'r> = ($($param::Item<'r>,)*);
            type State<'s> = EphemeralQueryState<'s>;

            fn query<'r, 's>(resources: &'r Resources, mut state: Self::State<'s>) -> Option<Self::Item<'r>> {
                Some(
                    (
                        $($param::query(resources, state.clone_to::<$param::State<'_>>())?,)*
                    )
                )
            }
        }

        impl<$($param: ResourceQueryMut + ResourceQueryUnsafe + 'static),*> ResourceQueryMut for ($($param,)*)
        {
            type MutItem<'r> = ($($param::MutItem<'r>,)*);
            type State<'s> = EphemeralQueryState<'s>;

            fn query_mut<'r, 's>(
                resources: &'r mut Resources,
                mut state: Self::State<'s>,
            ) -> Option<Self::MutItem<'r>> {
                unsafe {
                    Some(
                        (
                            $(<$param as ResourceQueryUnsafe>::query_unsafe(resources, state.clone_to::<$param::State<'_>>())?,)*
                        )
                    )
                }
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
