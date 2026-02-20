use crate::{
    background::resource_system::BackgroundRenderPipeline,
    render::{
        eventually::Eventually::{self, Initialized},
        render_phase::{PhaseItem, RenderCommand, RenderCommandResult},
        resource::TrackedRenderPass,
    },
    tcs::world::World,
};

pub struct SetBackgroundPipeline;
impl<P: PhaseItem> RenderCommand<P> for SetBackgroundPipeline {
    fn render<'w>(
        world: &'w World,
        _item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(Initialized(BackgroundRenderPipeline(pipeline))) = world
            .resources
            .get::<Eventually<BackgroundRenderPipeline>>()
        else {
            return RenderCommandResult::Failure;
        };

        pass.set_render_pipeline(pipeline);
        RenderCommandResult::Success
    }
}

pub struct DrawBackgroundQuad;
impl<P: PhaseItem> RenderCommand<P> for DrawBackgroundQuad {
    fn render<'w>(
        world: &'w World,
        item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(buf) = world
            .resources
            .get::<crate::background::queue_system::BackgroundBuffers>()
        {
            pass.set_vertex_buffer(0, buf.metadata_buffer.slice(..));

            // Simplified drawing for now, assuming a single background layer or that the first one is sufficient for the test.
            pass.draw(0..6, 0..1);
            return RenderCommandResult::Success;
        }
        RenderCommandResult::Failure
    }
}

pub struct DrawBackground;
impl<P: PhaseItem> RenderCommand<P> for DrawBackground {
    fn render<'w>(
        world: &'w World,
        item: &P,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mut result = SetBackgroundPipeline::render(world, item, pass);
        if let RenderCommandResult::Success = result {
            result = DrawBackgroundQuad::render(world, item, pass);
        }
        result
    }
}
