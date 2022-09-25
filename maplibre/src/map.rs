/// The [`Map`] defines the public interface of the map renderer.
// DO NOT IMPLEMENT INTERNALS ON THIS STRUCT.
pub struct Map<E: Environment> {
    // FIXME (wasm-executor): Avoid RefCell, change ownership model!
    map_schedule: Rc<RefCell<InteractiveMapSchedule<E>>>,
    window: RefCell<Option<<E::MapWindowConfig as MapWindowConfig>::MapWindow>>,
}

impl<E: Environment> Map<E>
where
    <E::MapWindowConfig as MapWindowConfig>::MapWindow: EventLoop<E>,
{
    /// Starts the [`crate::map_schedule::MapState`] Runnable with the configured event loop.
    pub fn run(&self) {
        self.run_with_optionally_max_frames(None);
    }

    /// Starts the [`crate::map_schedule::MapState`] Runnable with the configured event loop.
    ///
    /// # Arguments
    ///
    /// * `max_frames` - Maximum number of frames per second.
    pub fn run_with_max_frames(&self, max_frames: u64) {
        self.run_with_optionally_max_frames(Some(max_frames));
    }

    /// Starts the MapState Runnable with the configured event loop.
    ///
    /// # Arguments
    ///
    /// * `max_frames` - Optional maximum number of frames per second.
    pub fn run_with_optionally_max_frames(&self, max_frames: Option<u64>) {
        self.window
            .borrow_mut()
            .take()
            .unwrap() // FIXME (wasm-executor): Remove unwrap
            .run(self.map_schedule.clone(), max_frames);
    }

    pub fn map_schedule(&self) -> Rc<RefCell<InteractiveMapSchedule<E>>> {
        self.map_schedule.clone()
    }

    /*    pub fn map_schedule_mut(&mut self) -> &mut InteractiveMapSchedule<E> {
        &mut self.map_schedule
    }*/
}
