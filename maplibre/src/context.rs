use crate::io::shared_thread_state::SharedThreadState;
use crate::io::source_client::SourceClient;
use crate::io::tile_cache::TileCache;
use crate::map_state::ViewState;
use crate::{HTTPClient, Renderer, ScheduleMethod, Scheduler, Style};

pub struct MapContext {
    pub view_state: ViewState,
    pub style: Style,

    pub tile_cache: TileCache,
    pub renderer: Renderer,
    pub scheduler: Box<dyn ScheduleMethod>,

    pub shared_thread_state: SharedThreadState,
}
