use std::{any::type_name, borrow::Cow};

use crate::{context::MapContext, tcs::system::System};

/// Conversion trait to turn something into a [`System`].
///
/// Use this to get a system from a function. Also note that every system implements this trait as
/// well.
pub trait IntoSystem: Sized {
    type System: System;
    /// Turns this value into its corresponding [`System`].
    fn into_system(self) -> Self::System;
}

pub struct FunctionSystem<F> {
    func: F,
}

impl<F> System for FunctionSystem<F>
where
    F: FnMut(&mut MapContext) + 'static,
{
    fn name(&self) -> Cow<'static, str> {
        type_name::<F>().into()
    }

    fn run(&mut self, context: &mut MapContext) {
        (self.func)(context)
    }
}

impl<F> IntoSystem for F
where
    F: FnMut(&mut MapContext) + 'static,
{
    type System = FunctionSystem<F>;

    fn into_system(self) -> Self::System {
        FunctionSystem { func: self }
    }
}
