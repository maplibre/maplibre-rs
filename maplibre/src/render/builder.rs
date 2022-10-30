use std::marker::PhantomData;

use crate::{
    environment::Environment,
    error::Error,
    kernel::Kernel,
    render::{
        settings::{RendererSettings, WgpuSettings},
        Renderer,
    },
    style::Style,
    window::{HeadedMapWindow, MapWindow, MapWindowConfig},
};

pub struct RenderBuilder<MWC: MapWindowConfig> {
    wgpu_settings: Option<WgpuSettings>,
    renderer_settings: Option<RendererSettings>,
    phatom_mwc: PhantomData<MWC>,
}

impl<MWC: MapWindowConfig> RenderBuilder<MWC> {
    pub fn new() -> Self {
        Self {
            wgpu_settings: None,
            renderer_settings: None,
            phatom_mwc: Default::default(),
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

    pub fn build(self) -> UninitializedRenderer<MWC> {
        UninitializedRenderer {
            window: None,
            wgpu_settings: self.wgpu_settings.unwrap_or_default(),
            renderer_settings: self.renderer_settings.unwrap_or_default(),
        }
    }
}

pub enum InitializationResult<MWC: MapWindowConfig> {
    Initialized(InitializedRenderer<MWC>),
    Uninizalized(UninitializedRenderer<MWC>),
}

impl<MWC: MapWindowConfig> InitializationResult<MWC> {
    pub fn unwarp(self) -> InitializedRenderer<MWC> {
        match self {
            InitializationResult::Initialized(renderer) => renderer,
            InitializationResult::Uninizalized(_) => panic!("Renderer is not initialized"),
        }
    }
}

pub struct UninitializedRenderer<MWC: MapWindowConfig> {
    window: Option<MWC::MapWindow>,
    wgpu_settings: WgpuSettings,
    renderer_settings: RendererSettings,
}

impl<MWC: MapWindowConfig> UninitializedRenderer<MWC>
where
    MWC::MapWindow: MapWindow + HeadedMapWindow,
{
    /// Initializes the whole rendering pipeline for the given configuration.
    /// Returns the initialized map, ready to be run.
    async fn initialize(self, map_window_config: &MWC) -> Result<InitializationResult<MWC>, Error> {
        let window = map_window_config.create();

        #[cfg(target_os = "android")]
        {
            Ok(InitializationResult::Uninizalized(self))
        }

        #[cfg(not(target_os = "android"))]
        {
            let renderer = Renderer::initialize(
                &window,
                self.wgpu_settings.clone(),
                self.renderer_settings.clone(),
            )
            .await?;
            Ok(InitializationResult::Initialized(InitializedRenderer {
                window,
                renderer,
            }))
        }
    }

    pub async fn initialize_with<E>(
        self,
        kernel: &Kernel<E>,
    ) -> Result<InitializationResult<MWC>, Error>
    where
        E: Environment<MapWindowConfig = MWC>,
    {
        self.initialize(kernel.map_window_config()).await
    }
}

#[cfg(feature = "headless")]
impl<MWC: MapWindowConfig> UninitializedRenderer<MWC> {
    async fn initialize_headless(self, map_window_config: &MWC) -> Result<Renderer, Error> {
        let window = map_window_config.create();

        Ok(Renderer::initialize_headless(
            &window,
            self.wgpu_settings.clone(),
            self.renderer_settings.clone(),
        )
        .await?)
    }

    pub async fn initialize_headless_with<E>(self, kernel: &Kernel<E>) -> Result<Renderer, Error>
    where
        E: Environment<MapWindowConfig = MWC>,
    {
        self.initialize_headless(kernel.map_window_config()).await
    }
}

pub struct InitializedRenderer<MWC: MapWindowConfig> {
    pub window: MWC::MapWindow,
    pub renderer: Renderer,
}
