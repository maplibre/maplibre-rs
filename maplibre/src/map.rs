use std::rc::Rc;

use thiserror::Error;

use crate::{
    context::MapContext,
    coords::{LatLon, Zoom},
    ecs::world::World,
    environment::Environment,
    kernel::Kernel,
    plugin::Plugin,
    render::{
        builder::{
            InitializationResult, InitializedRenderer, RendererBuilder, UninitializedRenderer,
        },
        error::RenderError,
        graph::RenderGraphError,
        initialize_default_render_graph, register_default_render_stages,
    },
    schedule::{Schedule, Stage},
    style::Style,
    window::{HeadedMapWindow, MapWindow, MapWindowConfig},
};

#[derive(Error, Debug)]
pub enum MapError {
    /// No need to set renderer again
    #[error("renderer was already set for this map")]
    RendererAlreadySet,
    #[error("initializing render graph failed")]
    RenderGraphInit(RenderGraphError),
    #[error("initializing device failed")]
    DeviceInit(RenderError),
}

pub enum CurrentMapContext {
    Ready(MapContext),
    Pending {
        style: Style,
        renderer_builder: RendererBuilder,
    },
}

pub struct Map<E: Environment> {
    kernel: Rc<Kernel<E>>,
    schedule: Schedule,
    map_context: CurrentMapContext,
    window: <E::MapWindowConfig as MapWindowConfig>::MapWindow,

    plugins: Vec<Box<dyn Plugin<E>>>,
}

impl<E: Environment> Map<E>
where
    <<E as Environment>::MapWindowConfig as MapWindowConfig>::MapWindow: HeadedMapWindow,
{
    pub fn new(
        style: Style,
        mut kernel: Kernel<E>,
        renderer_builder: RendererBuilder,
        plugins: Vec<Box<dyn Plugin<E>>>,
    ) -> Result<Self, MapError> {
        let mut schedule = Schedule::default();
        register_default_render_stages(&mut schedule);

        let kernel = Rc::new(kernel);

        let window = kernel.map_window_config().create();

        let map = Self {
            kernel,
            schedule,
            map_context: CurrentMapContext::Pending {
                style,
                renderer_builder,
            },
            window,
            plugins,
        };
        Ok(map)
    }

    pub async fn initialize_renderer(&mut self) -> Result<(), MapError> {
        match &mut self.map_context {
            CurrentMapContext::Ready(_) => Err(MapError::RendererAlreadySet),
            CurrentMapContext::Pending {
                style,
                renderer_builder,
            } => {
                let init_result = renderer_builder
                    .clone() // Cloning because we want to be able to build multiple times maybe
                    .build()
                    .initialize_renderer::<E::MapWindowConfig>(&self.window)
                    .await
                    .map_err(MapError::DeviceInit)?;

                let window_size = self.window.size();

                let center = style.center.unwrap_or_default();

                let mut world = World::new_at(
                    window_size,
                    LatLon::new(center[0], center[1]),
                    style.zoom.map(Zoom::new).unwrap_or_default(),
                    cgmath::Deg::<f64>(style.pitch.unwrap_or_default()),
                );

                // FIXME tcs: Add initialization logic
                for plugin in &self.plugins {
                    plugin.build(&mut self.schedule, self.kernel.clone(), &mut world);
                }

                match init_result {
                    InitializationResult::Initialized(InitializedRenderer {
                        mut renderer, ..
                    }) => {
                        // FIXME tcs: Move to rendering core
                        initialize_default_render_graph(&mut renderer.render_graph).unwrap();

                        self.map_context = CurrentMapContext::Ready(MapContext {
                            world,
                            style: std::mem::take(style),
                            renderer,
                        });
                    }
                    InitializationResult::Uninizalized(UninitializedRenderer { .. }) => {}
                    _ => panic!("Rendering context gone"),
                };
                Ok(())
            }
        }
    }

    pub fn window_mut(&mut self) -> &mut <E::MapWindowConfig as MapWindowConfig>::MapWindow {
        &mut self.window
    }
    pub fn window(&self) -> &<E::MapWindowConfig as MapWindowConfig>::MapWindow {
        &self.window
    }

    pub fn has_renderer(&self) -> bool {
        match &self.map_context {
            CurrentMapContext::Ready(_) => true,
            CurrentMapContext::Pending { .. } => false,
        }
    }

    #[tracing::instrument(name = "update_and_redraw", skip_all)]
    pub fn run_schedule(&mut self) -> Result<(), MapError> {
        match &mut self.map_context {
            CurrentMapContext::Ready(map_context) => {
                self.schedule.run(map_context);
                Ok(())
            }
            CurrentMapContext::Pending { .. } => Err(MapError::RendererAlreadySet),
        }
    }

    pub fn context(&self) -> Result<&MapContext, MapError> {
        match &self.map_context {
            CurrentMapContext::Ready(map_context) => Ok(map_context),
            CurrentMapContext::Pending { .. } => Err(MapError::RendererAlreadySet),
        }
    }

    pub fn context_mut(&mut self) -> Result<&mut MapContext, MapError> {
        match &mut self.map_context {
            CurrentMapContext::Ready(map_context) => Ok(map_context),
            CurrentMapContext::Pending { .. } => Err(MapError::RendererAlreadySet),
        }
    }

    pub fn kernel(&self) -> &Rc<Kernel<E>> {
        &self.kernel
    }
}
