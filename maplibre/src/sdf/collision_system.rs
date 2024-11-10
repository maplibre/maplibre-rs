use std::borrow::Cow;

use cgmath::{Matrix3, Vector3};

use crate::{
    context::MapContext,
    coords::{EXTENT, TILE_SIZE},
    euclid::Point2D,
    legacy::{
        buckets::symbol_bucket::PlacedSymbol,
        collision_feature::{CollisionBox, CollisionFeature},
        collision_index::CollisionIndex,
        geometry::feature_index::{IndexedSubfeature, RefIndexedSubfeature},
        geometry_tile_data::GeometryCoordinates,
        MapMode,
    },
    render::{
        eventually::{Eventually, Eventually::Initialized},
        shaders::SDFShaderFeatureMetadata,
        tile_view_pattern::WgpuTileViewPattern,
        Renderer,
    },
    sdf::{SymbolBufferPool, SymbolLayersDataComponent},
    tcs::system::{System, SystemError, SystemResult},
};

pub struct CollisionSystem {}

impl Default for CollisionSystem {
    fn default() -> Self {
        Self::new()
    }
}

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
            world,
            view_state,
            renderer: Renderer { device, queue, .. },
            ..
        }: &mut MapContext,
    ) -> SystemResult {
        let Some((Initialized(tile_view_pattern), Initialized(symbol_buffer_pool))) =
            world.resources.query_mut::<(
                &mut Eventually<WgpuTileViewPattern>,
                &mut Eventually<SymbolBufferPool>,
            )>()
        else {
            return Err(SystemError::Dependencies);
        };

        if !view_state.did_camera_change() {
            // TODO
            // return Ok(());
        }

        let mut collision_index = CollisionIndex::new(view_state, MapMode::Continuous);

        for view_tile in tile_view_pattern.iter() {
            let coords = view_tile.coords();
            if let Some(component) = world.tiles.query::<&SymbolLayersDataComponent>(coords) {
                for layer in &component.layers {
                    let mut feature_metadata = vec![
                        SDFShaderFeatureMetadata::default();
                        layer
                            .features
                            .last()
                            .map(|feature| feature.indices.end)
                            .unwrap_or_default()
                    ];

                    for feature in &layer.features {
                        // calculate where tile is

                        let transform = coords.transform_for_zoom(view_state.zoom());

                        let pos_matrix = view_state
                            .view_projection()
                            .to_model_view_projection(transform);

                        let zoom_factor = view_state.zoom().scale_to_tile(&coords);

                        let font_scale = 6.0;
                        let scaling = Matrix3::from_cols(
                            Vector3::new(zoom_factor * font_scale, 0.0, 0.0),
                            Vector3::new(0.0, zoom_factor * font_scale, 0.0),
                            Vector3::new(0.0, 0.0, 1.0),
                        );

                        let vec3 = Vector3::new(
                            feature.bbox.max.x as f64,
                            feature.bbox.max.y as f64,
                            0.0f64,
                        );
                        let text_anchor = Vector3::new(
                            feature.text_anchor.x as f64,
                            feature.text_anchor.y as f64,
                            0.0f64,
                        );

                        let shader = pos_matrix.get()
                            * (scaling * (vec3 - text_anchor) + text_anchor).extend(1.0);
                        let window = view_state.clip_to_window(&shader);

                        //println!("{:?}", window);

                        let anchor_point =
                            Point2D::new(feature.bbox.min.x as f64, feature.bbox.min.y as f64); // TODO

                        let boxes = vec![CollisionBox {
                            anchor: anchor_point,
                            x1: 0.0 * (EXTENT / TILE_SIZE),
                            y1: 0. * (EXTENT / TILE_SIZE),
                            x2: (feature.bbox.max.x - feature.bbox.min.x) as f64, //* (EXTENT / TILE_SIZE),
                            y2: (feature.bbox.max.y - feature.bbox.min.y) as f64, // * (EXTENT / TILE_SIZE),
                            signed_distance_from_anchor: 0.0,
                        }]; // TODO

                        let mut projected_boxes = vec![];
                        let collision_feature = CollisionFeature {
                            boxes,
                            indexed_feature: IndexedSubfeature {
                                ref_: RefIndexedSubfeature {
                                    index: 0,
                                    sort_index: 0,
                                    source_layer_name: "".to_string(),
                                    bucket_leader_id: "".to_string(),
                                    bucket_instance_id: 0,
                                    collision_group_id: 0,
                                },
                                source_layer_name_copy: "".to_string(),
                                bucket_leader_idcopy: "".to_string(),
                            },
                            along_line: false, // false if point, else true
                        };
                        let (placed_text, is_offscreen) = collision_index.place_feature(
                            &collision_feature,
                            Point2D::zero(), // shift
                            &pos_matrix,
                            &pos_matrix.get(), // TODO
                            //TILE_SIZE / EXTENT,
                            1.0,
                            &PlacedSymbol {
                                anchor_point,
                                segment: 0,
                                lower_size: 0.0,
                                upper_size: 0.0,
                                line_offset: [0., 0.],
                                writing_modes: Default::default(),
                                line: GeometryCoordinates(vec![anchor_point.cast()]), // TODO can be linestring or just a single point
                                tile_distances: vec![],                               // TODO
                                glyph_offsets: vec![0., 0.],                          // TODO
                                hidden: false,
                                vertex_start_index: 0,
                                cross_tile_id: 0,
                                placed_orientation: None,
                                angle: 0.0,

                                placed_icon_index: None,
                            },
                            view_state.zoom().scale_to_zoom_level(coords.z),
                            6.0,
                            false,
                            false,
                            false,
                            None,                               // avoidEdges
                            Some(|f: &IndexedSubfeature| true), // collisionGroupPredicate
                            &mut projected_boxes,               // output
                        );
                        if feature.str.starts_with("Ette") {
                            //println!("{}", feature.str);
                            //println!("{:?}", &collision_feature.boxes);
                            //println!("proj {:?}", &projected_boxes.get(0));
                        }

                        if placed_text {
                            collision_index.insert_feature(
                                collision_feature,
                                &projected_boxes,
                                false,
                                55,
                                66,
                            );

                            for index in feature.indices.clone() {
                                let index = layer.buffer.buffer.indices[index] as usize;
                                feature_metadata[index].opacity = 1.0;
                            }
                        } else {
                            for index in feature.indices.clone() {
                                let index = layer.buffer.buffer.indices[index] as usize;
                                feature_metadata[index].opacity = 0.0;
                            }

                            //feature_metadata.extend(iter::repeat(SDFShaderFeatureMetadata { opacity: 0.0 }).take(feature.indices.len()))
                        }
                    }

                    if let Some(layer_at_coords) = symbol_buffer_pool.index().get_layers(coords) {
                        for entry in layer_at_coords {
                            debug_assert_eq!(entry.coords, coords);

                            let source_layer = entry.style_layer.source_layer.as_ref().unwrap();

                            if source_layer != &layer.source_layer {
                                continue;
                            }

                            symbol_buffer_pool.update_feature_metadata(
                                queue,
                                entry,
                                &feature_metadata,
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
