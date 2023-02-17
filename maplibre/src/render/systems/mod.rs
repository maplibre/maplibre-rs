//! Rendering specific systems

use graph_runner_system::GraphRunnerSystem;

use crate::{
    render::{
        systems::{
            cleanup_system::cleanup_system, resource_system::ResourceSystem,
            sort_phase_system::sort_phase_system,
            tile_view_pattern_system::tile_view_pattern_system,
        },
        RenderStageLabel,
    },
    schedule::Schedule,
    tcs::system::{stage::SystemStage, SystemContainer},
};

mod cleanup_system;
mod graph_runner_system;
mod resource_system;
mod sort_phase_system;
mod tile_view_pattern_system;

pub fn register_default_render_stages(schedule: &mut Schedule) {
    schedule.add_stage(RenderStageLabel::Extract, SystemStage::default());
    schedule.add_stage(
        RenderStageLabel::Prepare,
        SystemStage::default().with_system(SystemContainer::new(ResourceSystem)),
    );
    schedule.add_stage(
        RenderStageLabel::Queue,
        SystemStage::default().with_system(tile_view_pattern_system),
    );
    schedule.add_stage(
        RenderStageLabel::PhaseSort,
        SystemStage::default().with_system(sort_phase_system),
    );
    schedule.add_stage(
        RenderStageLabel::Render,
        SystemStage::default().with_system(SystemContainer::new(GraphRunnerSystem)),
    );
    schedule.add_stage(
        RenderStageLabel::Cleanup,
        SystemStage::default().with_system(cleanup_system),
    );
}
