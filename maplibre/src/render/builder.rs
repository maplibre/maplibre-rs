use crate::{
    render::{
        error::RenderError,
        settings::{RendererSettings, WgpuSettings},
        Renderer,
    },
    window::{HeadedMapWindow, MapWindowConfig},
};

#[derive(Clone)]
pub struct RendererBuilder {
    wgpu_settings: Option<WgpuSettings>,
    renderer_settings: Option<RendererSettings>,
}

impl RendererBuilder {
    pub fn new() -> Self {
        Self {
            wgpu_settings: None,
            renderer_settings: None,
        }
    }

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

impl Default for RendererBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub enum InitializationResult {
    Initialized(InitializedRenderer),
    Uninitialized(UninitializedRenderer),
    Gone,
}

impl Default for InitializationResult {
    fn default() -> Self {
        Self::Gone
    }
}

impl InitializationResult {
    pub fn unwrap_renderer(self) -> InitializedRenderer {
        match self {
            InitializationResult::Initialized(renderer) => renderer,
            InitializationResult::Uninitialized(_) => panic!("Renderer is not initialized"),
            InitializationResult::Gone => panic!("Initialization context is gone"),
        }
    }

    pub fn into_option(self) -> Option<Renderer> {
        match self {
            InitializationResult::Initialized(InitializedRenderer { renderer, .. }) => {
                Some(renderer)
            }
            InitializationResult::Uninitialized(_) => None,
            InitializationResult::Gone => panic!("Initialization context is gone"),
        }
    }
}

pub struct UninitializedRenderer {
    pub wgpu_settings: WgpuSettings,
    pub renderer_settings: RendererSettings,
}

impl UninitializedRenderer {
    /// Initializes the whole rendering pipeline for the given configuration.
    /// Returns the initialized map, ready to be run.
    pub async fn initialize_renderer<MWC>(
        self,
        existing_window: &MWC::MapWindow,
    ) -> Result<InitializationResult, RenderError>
    where
        MWC: MapWindowConfig,
        <MWC as MapWindowConfig>::MapWindow: HeadedMapWindow,
    {
        let renderer = Renderer::initialize(
            existing_window,
            self.wgpu_settings.clone(),
            self.renderer_settings,
        )
        .await?;
        Ok(InitializationResult::Initialized(InitializedRenderer {
            renderer,
        }))
    }
}

#[cfg(feature = "headless")]
impl UninitializedRenderer {
    pub(crate) async fn initialize_headless<MWC>(
        self,
        existing_window: &MWC::MapWindow,
    ) -> Result<Renderer, RenderError>
    where
        MWC: MapWindowConfig,
    {
        Renderer::initialize_headless(
            existing_window,
            self.wgpu_settings.clone(),
            self.renderer_settings,
        )
        .await
    }
}

pub struct InitializedRenderer {
    pub renderer: Renderer,
}
