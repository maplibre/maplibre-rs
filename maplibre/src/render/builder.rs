use crate::environment::Kernel;
use crate::error::Error;
use crate::{
    environment::Environment,
    render::{
        settings::{RendererSettings, WgpuSettings},
        Renderer,
    },
    style::Style,
    window::{HeadedMapWindow, MapWindow, MapWindowConfig},
};

pub struct RenderBuilder {
    wgpu_settings: Option<WgpuSettings>,
    renderer_settings: Option<RendererSettings>,
}

impl RenderBuilder {
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

pub enum InitializationResult {
    Initialized(Renderer),
    Uninizalized(UninitializedRenderer),
}

impl InitializationResult {
    pub fn unwarp(self) -> Renderer {
        match self {
            InitializationResult::Initialized(renderer) => renderer,
            InitializationResult::Uninizalized(_) => panic!("Renderer is not initialized"),
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
    async fn initialize<MWC: MapWindowConfig>(
        self,
        map_window_config: &MWC,
    ) -> Result<InitializationResult, Error>
    where
        MWC::MapWindow: MapWindow + HeadedMapWindow,
    {
        let window = map_window_config.create();

        #[cfg(target_os = "android")]
        let renderer = Ok(InitializationResult::Uninizalized(self));
        #[cfg(not(target_os = "android"))]
        let renderer = Ok(InitializationResult::Initialized(
            Renderer::initialize(
                &window,
                self.wgpu_settings.clone(),
                self.renderer_settings.clone(),
            )
            .await?,
        ));

        renderer
    }

    pub async fn initialize_with<E: Environment>(
        self,
        kernel: &Kernel<E>,
    ) -> Result<InitializationResult, Error>
    where
        <E::MapWindowConfig as MapWindowConfig>::MapWindow: MapWindow + HeadedMapWindow,
    {
        self.initialize(&kernel.map_window_config).await
    }
}

#[cfg(feature = "headless")]
impl UninitializedRenderer {
    async fn initialize_headless<MWC: MapWindowConfig>(
        self,
        map_window_config: MWC,
    ) -> Result<Renderer, Error> {
        let window = map_window_config.create();

        Ok(Renderer::initialize_headless(
            &window,
            self.wgpu_settings.clone(),
            self.renderer_settings.clone(),
        )
        .await?)
    }

    pub async fn initialize_headless_with<E: Environment>(
        self,
        kernel: &Kernel<E>,
    ) -> Result<InitializationResult, Error>
    where
        <E::MapWindowConfig as MapWindowConfig>::MapWindow: MapWindow + HeadedMapWindow,
    {
        self.initialize(&kernel.map_window_config).await
    }
}
