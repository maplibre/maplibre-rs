use crate::environment::Environment;
use crate::render::settings::{RendererSettings, WgpuSettings};
use crate::render::Renderer;
use crate::style::Style;
use crate::window::{HeadedMapWindow, MapWindow, MapWindowConfig};

pub struct RenderBuilder {
    wgpu_settings: Option<WgpuSettings>,
    renderer_settings: Option<RendererSettings>,
}

impl RenderBuilder {
    pub fn with_renderer_settings(mut self, renderer_settings: RendererSettings) -> Self {
        self.renderer_settings = Some(renderer_settings);
        self
    }

    pub fn with_wgpu_settings(mut self, wgpu_settings: WgpuSettings) -> Self {
        self.wgpu_settings = Some(wgpu_settings);
        self
    }

    pub fn build(self) -> UninitializedRenderer {
        UninitializedRenderer {
            wgpu_settings: self.wgpu_settings.unwrap_or_default(),
            renderer_settings: self.renderer_settings.unwrap_or_default(),
        }
    }
}

pub struct UninitializedRenderer {
    wgpu_settings: WgpuSettings,
    renderer_settings: RendererSettings,
}

impl UninitializedRenderer {
    /// Initializes the whole rendering pipeline for the given configuration.
    /// Returns the initialized map, ready to be run.
    pub async fn initialize<MWC: MapWindowConfig>(self, map_window_config: MWC) -> Option<Renderer>
    where
        MWC::MapWindow: MapWindow + HeadedMapWindow,
    {
        let window = map_window_config.create();

        #[cfg(target_os = "android")]
        let renderer = None;
        #[cfg(not(target_os = "android"))]
        let renderer = Renderer::initialize(
            &window,
            self.wgpu_settings.clone(),
            self.renderer_settings.clone(),
        )
        .await
        .ok();

        renderer
    }
}

#[cfg(feature = "headless")]
impl UninitializedRenderer {
    pub async fn initialize_headless<MWC: MapWindowConfig>(
        self,
        map_window_config: MWC,
    ) -> Option<Renderer> {
        let window = map_window_config.create();

        Renderer::initialize_headless(
            &window,
            self.wgpu_settings.clone(),
            self.renderer_settings.clone(),
        )
        .await
        .ok()
    }
}
