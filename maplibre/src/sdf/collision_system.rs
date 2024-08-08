use std::borrow::Cow;

use cgmath::{Matrix3, Vector3};

use crate::{
    context::MapContext,
    coords::WorldTileCoords,
    sdf::SymbolLayersDataComponent,
    tcs::system::{System, SystemResult},
};

pub struct CollisionSystem {}

impl CollisionSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl System for CollisionSystem {
    fn name(&self) -> Cow<'static, str> {
        "sdf_populate_world_system".into()
    }

    fn run(
        &mut self,
        MapContext {
            world, view_state, ..
        }: &mut MapContext,
    ) -> SystemResult {
        let coords = WorldTileCoords {
            x: 4193,
            y: 2746,
            z: 13.into(),
        };
        let comp = world.tiles.query::<&SymbolLayersDataComponent>(coords);

        if let Some(component) = comp {
            if let Some(l) = component.layers.get(0) {
                for feature in &l.features.last() {
                    // calculate where tile is
                    let tile = WorldTileCoords::from(coords);
                    let transform = tile.transform_for_zoom(view_state.zoom());

                    let translate = view_state
                        .view_projection()
                        .to_model_view_projection(transform)
                        .get();

                    let zoom_factor = view_state.zoom().scale_to_tile(&coords);

                    let font_scale = 6.0;
                    let scaling = Matrix3::from_cols(
                        Vector3::new(zoom_factor * font_scale, 0.0, 0.0),
                        Vector3::new(0.0, zoom_factor * font_scale, 0.0),
                        Vector3::new(0.0, 0.0, 1.0),
                    );

                    let vec3 =
                        Vector3::new(feature.bbox.max.x as f64, feature.bbox.max.y as f64, 0.0f64);
                    let text_anchor = Vector3::new(
                        feature.text_anchor.x as f64,
                        feature.text_anchor.y as f64,
                        0.0f64,
                    );

                    let shader =
                        translate * (scaling * (vec3 - text_anchor) + text_anchor).extend(1.0);
                    let window = view_state.clip_to_window(&shader);

                    println!("{:?}", window)
                }
            }
        }
        Ok(())
    }
}
