use std::marker::PhantomData;

use crate::{render::resource::TrackedRenderPass, tcs::world::World};

/// A draw function which is used to draw a specific [`PhaseItem`].
///
/// They are the the general form of drawing items, whereas [`RenderCommands`](RenderCommand)
/// are more modular.
pub trait Draw<P: PhaseItem>: 'static {
    /// Draws the [`PhaseItem`] by issuing draw calls via the [`TrackedRenderPass`].
    fn draw<'w>(&self, pass: &mut TrackedRenderPass<'w>, wold: &'w World, item: &P);
}

/// An item which will be drawn to the screen. A phase item should be queued up for rendering
/// during the [`RenderStageLabel::Queue`](crate::RenderStageLabel::Queue) stage.
/// Afterwards it will be sorted and rendered automatically  in the
/// [`RenderStageLabel::PhaseSort`](crate::RenderStageLabel::PhaseSort) stage and
/// [`RenderStageLabel::Render`](crate::RenderStageLabel::Render) stage, respectively.
pub trait PhaseItem {
    /// The type used for ordering the items. The smallest values are drawn first.
    type SortKey: Ord;
    /// Determines the order in which the items are drawn during the corresponding [`RenderPhase`](super::RenderPhase).
    fn sort_key(&self) -> Self::SortKey;

    fn draw_function(&self) -> &dyn Draw<Self>;
}

/// [`RenderCommand`] is a trait that runs an ECS query and produces one or more
/// [`TrackedRenderPass`] calls. Types implementing this trait can be composed (as tuples).
///
/// They can be registered as a [`Draw`] function via the
/// [`AddRenderCommand::add_render_command`] method.
///
/// # Example
/// The `DrawPbr` draw function is created from the following render command
/// tuple.  Const generics are used to set specific bind group locations:
///
/// ```ignore
/// pub type DrawPbr = (
///     SetItemPipeline,
///     SetMeshViewBindGroup<0>,
///     SetStandardMaterialBindGroup<1>,
///     SetTransformBindGroup<2>,
///     DrawMesh,
/// );
/// ```
pub trait RenderCommand<P: PhaseItem> {
    /// Renders the [`PhaseItem`] by issuing draw calls via the [`TrackedRenderPass`].
    // TODO: reorder the arguments to match Node and Draw
    fn render<'w>(
        world: &'w World,
        item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult;
}

pub enum RenderCommandResult {
    Success,
    Failure,
}

macro_rules! render_command_tuple_impl {
    ($($name: ident),*) => {
        impl<P: PhaseItem, $($name: RenderCommand<P>),*> RenderCommand<P> for ($($name,)*) {
            #[allow(non_snake_case)]
            fn render<'w>(
                world: &'w World,
                item: &P,
                pass: &mut TrackedRenderPass<'w>,
            ) -> RenderCommandResult{
                $(if let RenderCommandResult::Failure = $name::render(world, item, pass) {
                    return RenderCommandResult::Failure;
                })*
                RenderCommandResult::Success
            }
        }
    };
}

render_command_tuple_impl!(C0);
render_command_tuple_impl!(C0, C1);
render_command_tuple_impl!(C0, C1, C2);
render_command_tuple_impl!(C0, C1, C2, C3);
render_command_tuple_impl!(C0, C1, C2, C3, C4);

pub struct DrawState<C, P> {
    phantom_p: PhantomData<P>,
    phantom_c: PhantomData<C>,
}

impl<C, P> DrawState<C, P> {
    pub(crate) fn new() -> Self {
        DrawState {
            phantom_p: Default::default(),
            phantom_c: Default::default(),
        }
    }
}

impl<P: 'static, C: 'static> Draw<P> for DrawState<P, C>
where
    P: PhaseItem,
    C: RenderCommand<P>,
{
    /// Prepares data for the wrapped [`RenderCommand`] and then renders it.
    fn draw<'w>(&self, pass: &mut TrackedRenderPass<'w>, world: &'w World, item: &P) {
        C::render(world, item, pass);
    }
}
