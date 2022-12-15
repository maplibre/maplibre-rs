use std::rc::Rc;

use thiserror::Error;

use crate::{
    context::MapContext,
    coords::{LatLon, Zoom},
    environment::Environment,
    kernel::Kernel,
    render::{
        builder::{
            InitializationResult, InitializedRenderer, RendererBuilder, UninitializedRenderer,
        },
        create_default_render_graph,
        error::RenderError,
        graph::RenderGraphError,
        register_default_render_stages,
    },
    schedule::{Schedule, Stage},
    stages::register_stages,
    style::Style,
    window::{HeadedMapWindow, MapWindow, MapWindowConfig},
    world::World,
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

pub enum MapContextState {
    Ready(MapContext),
    Pending {
        style: Style,
        renderer_builder: RendererBuilder,
    },
}

pub struct Map<E: Environment> {
    kernel: Rc<Kernel<E>>,
    schedule: Schedule,
    map_context: MapContextState,
    window: <E::MapWindowConfig as MapWindowConfig>::MapWindow,
}

impl<E: Environment> Map<E>
where
    <<E as Environment>::MapWindowConfig as MapWindowConfig>::MapWindow: HeadedMapWindow,
{
    pub fn new(
        style: Style,
        kernel: Kernel<E>,
        renderer_builder: RendererBuilder,
    ) -> Result<Self, MapError> {
        let mut schedule = Schedule::default();

        let graph = create_default_render_graph().unwrap(); // TODO: Remove unwrap
        register_default_render_stages(graph, &mut schedule);

        let kernel = Rc::new(kernel);

        register_stages::<E>(&mut schedule, kernel.clone());

        let window = kernel.map_window_config().create();

        let map = Self {
            kernel,
            map_context: MapContextState::Pending {
                style,
                renderer_builder,
            },
            schedule,
            window,
        };
        Ok(map)
    }

    pub async fn initialize_renderer(&mut self) -> Result<(), MapError> {
        match &mut self.map_context {
            MapContextState::Ready(_) => Err(MapError::RendererAlreadySet),
            MapContextState::Pending {
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

                let world = World::new_at(
                    window_size,
                    LatLon::new(center[0], center[1]),
                    style.zoom.map(Zoom::new).unwrap_or_default(),
                    cgmath::Deg::<f64>(style.pitch.unwrap_or_default()),
                );

                match init_result {
                    InitializationResult::Initialized(InitializedRenderer { renderer, .. }) => {
                        self.map_context = MapContextState::Ready(MapContext {
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
            MapContextState::Ready(_) => true,
            MapContextState::Pending { .. } => false,
        }
    }

    #[tracing::instrument(name = "update_and_redraw", skip_all)]
    pub fn run_schedule(&mut self) -> Result<(), MapError> {
        match &mut self.map_context {
            MapContextState::Ready(map_context) => {
                self.schedule.run(map_context);
                Ok(())
            }
            MapContextState::Pending { .. } => Err(MapError::RendererAlreadySet),
        }
    }

    pub fn context(&self) -> Result<&MapContext, MapError> {
        match &self.map_context {
            MapContextState::Ready(map_context) => Ok(map_context),
            MapContextState::Pending { .. } => Err(MapError::RendererAlreadySet),
        }
    }

    pub fn context_mut(&mut self) -> Result<&mut MapContext, MapError> {
        match &mut self.map_context {
            MapContextState::Ready(map_context) => Ok(map_context),
            MapContextState::Pending { .. } => Err(MapError::RendererAlreadySet),
        }
    }

    pub fn kernel(&self) -> &Rc<Kernel<E>> {
        &self.kernel
    }
}
