/*    pub fn resume(&mut self, window: &<E::MapWindowConfig as MapWindowConfig>::MapWindow) {
    if let EventuallyMapContext::Full(map_context) = &mut self.map_context {
        let renderer = &mut map_context.renderer;
        renderer.state.recreate_surface(window, &renderer.instance);
    }
}*/

/*    pub async fn late_init(&mut self) -> bool {
    match &self.map_context {
        EventuallyMapContext::Full(_) => false,
        EventuallyMapContext::Uninizalized(PrematureMapContext {
            wgpu_settings,
            renderer_settings,
            ..
        }) => {
            let window = self.map_window_config.create();
            let renderer =
                Renderer::initialize(&window, wgpu_settings.clone(), renderer_settings.clone())
                    .await
                    .unwrap(); // TODO: Remove unwrap
            self.map_context.make_full(renderer);
            true
        }
        EventuallyMapContext::_Uninitialized => false,
    }
}*/
