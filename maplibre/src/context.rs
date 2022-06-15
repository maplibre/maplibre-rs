use crate::coords::{Zoom, ZoomLevel, TILE_SIZE, WorldCoords};
use crate::io::tile_repository::TileRepository;
use crate::render::camera::{Camera, Perspective, ViewProjection};
use crate::util::ChangeObserver;
use crate::{Renderer, Style, WindowSize};

/// Stores the camera configuration.
pub struct ViewState {
    pub zoom: ChangeObserver<Zoom>,
    pub camera: ChangeObserver<Camera>,
    pub perspective: Perspective,
}

impl ViewState {
    pub fn new(window_size: &WindowSize, style: &Style) -> Self {
        let zoom = style.zoom.map_or_else(|| ChangeObserver::default(), |zoom| ChangeObserver::new(Zoom::new(zoom)));
        let (lat, lon) = style.center.map_or((0.0, 0.0), |center| (center[0], center[1]));
        let position = WorldCoords::from_lat_lon(lat, lon, zoom.0);
        let camera = Camera::new(
            (position.x, position.y, 150.0),
            cgmath::Deg(-90.0),
            cgmath::Deg(style.pitch.unwrap_or(0.0)),
            window_size.width(),
            window_size.height(),
        );

        let perspective = Perspective::new(
            window_size.width(),
            window_size.height(),
            cgmath::Deg(110.0),
            100.0,
            2000.0,
        );

        Self {
            zoom,
            camera: ChangeObserver::new(camera),
            perspective,
        }
    }

    pub fn view_projection(&self) -> ViewProjection {
        self.camera.calc_view_proj(&self.perspective)
    }

    pub fn visible_level(&self) -> ZoomLevel {
        self.zoom.level()
    }

    pub fn zoom(&self) -> Zoom {
        *self.zoom
    }

    pub fn update_zoom(&mut self, new_zoom: Zoom) {
        *self.zoom = new_zoom;
        log::info!("zoom: {}", new_zoom);
    }
}

/// Stores the context of the map.
pub struct MapContext {
    pub view_state: ViewState,
    pub style: Style,

    pub tile_repository: TileRepository,
    pub renderer: Renderer,
}
