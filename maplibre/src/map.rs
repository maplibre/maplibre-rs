use std::rc::Rc;

use crate::{
    context::MapContext,
    coords::{LatLon, WorldCoords, Zoom, TILE_SIZE},
    environment::Environment,
    error::Error,
    headless::environment::HeadlessEnvironment,
    kernel::Kernel,
    render::{create_default_render_graph, register_default_render_stages, Renderer},
    schedule::{Schedule, Stage},
    stages::register_stages,
    style::Style,
    world::World,
};

pub struct Map<E: Environment> {
    kernel: Rc<Kernel<E>>,
    schedule: Schedule,
    map_context: MapContext,
}

impl<E: Environment> Map<E> {
    pub fn new(style: Style, kernel: Kernel<E>, renderer: Renderer) -> Result<Self, Error> {
        let window_size = renderer.state().surface().size();

        let center = style.center.unwrap_or_default();
        let world = World::new_at(
            window_size,
            LatLon::new(center[0], center[1]),
            style.zoom.map(|zoom| Zoom::new(zoom)).unwrap_or_default(),
            cgmath::Deg::<f64>(style.pitch.unwrap_or_default()),
        );

        let mut schedule = Schedule::default();

        let graph = create_default_render_graph().unwrap(); // TODO: Remove unwrap
        register_default_render_stages(graph, &mut schedule);

        let kernel = Rc::new(kernel);

        register_stages::<E>(&mut schedule, kernel.clone());

        Ok(Self {
            kernel,
            map_context: MapContext {
                style,
                world,
                renderer,
            },
            schedule,
        })
    }

    #[tracing::instrument(name = "update_and_redraw", skip_all)]
    pub fn run_schedule(&mut self) -> Result<(), Error> {
        self.schedule.run(&mut self.map_context);
        Ok(())
    }
}
