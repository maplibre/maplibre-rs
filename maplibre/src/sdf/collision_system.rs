use std::borrow::Cow;

use crate::{
    context::MapContext,
    coords::{TileCoords, WorldTileCoords, ZOOM_BOUNDS},
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
        projection::globe_camera_for_view,
        shaders::SDFShaderFeatureMetadata,
        tile_view_pattern::WgpuTileViewPattern,
        Renderer,
    },
    sdf::{Feature, SymbolBufferPool, SymbolLayersDataComponent},
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
            style,
            view_state,
            renderer: Renderer { queue, .. },
            ..
        }: &mut MapContext,
    ) -> SystemResult {
        let uses_globe = style.projection.as_ref().is_some_and(|specification| {
            specification
                .projection_type
                .uses_globe_rendering(view_state.zoom().value())
        });
        let globe_camera = if uses_globe {
            Some(globe_camera_for_view(view_state).map_err(|error| {
                tracing::error!(%error, "unable to project globe symbol collisions");
                SystemError::Setup
            })?)
        } else {
            None
        };
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
                    let metadata_count = if layer.features.is_empty() {
                        layer.new_buffer.buffer.vertices.len()
                    } else {
                        layer
                            .features
                            .last()
                            .map(|feature| feature.indices.end)
                            .unwrap_or_default()
                    };
                    let mut feature_metadata =
                        vec![SDFShaderFeatureMetadata { opacity: 1.0 }; metadata_count];

                    for feature in &layer.features {
                        let is_occluded = globe_camera.as_ref().is_some_and(|camera| {
                            canonical_tile(coords).is_none_or(|tile| {
                                camera
                                    .project_tile_coordinates(
                                        f64::from(feature.text_anchor.x),
                                        f64::from(feature.text_anchor.y),
                                        tile,
                                        0.0,
                                    )
                                    .is_occluded
                            })
                        });
                        if is_occluded {
                            set_feature_opacity(feature, layer, &mut feature_metadata, 0.0);
                            continue;
                        }
                        // calculate where tile is

                        let transform = coords.transform_for_zoom(view_state.zoom());

                        let pos_matrix = view_state
                            .view_projection()
                            .to_model_view_projection(transform);

                        let anchor_point =
                            Point2D::new(feature.bbox.min.x as f64, feature.bbox.min.y as f64); // TODO

                        let boxes = vec![CollisionBox {
                            anchor: anchor_point,
                            x1: 0.0,
                            y1: 0.0,
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
                        let (placed_text, _is_offscreen) = collision_index.place_feature(
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
                            None,                                      // avoidEdges
                            Some(|_feature: &IndexedSubfeature| true), // collisionGroupPredicate
                            &mut projected_boxes,                      // output
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

                            set_feature_opacity(feature, layer, &mut feature_metadata, 1.0);
                        } else {
                            set_feature_opacity(feature, layer, &mut feature_metadata, 0.0);

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

fn canonical_tile(coords: WorldTileCoords) -> Option<TileCoords> {
    let tile_count = i32::try_from(ZOOM_BOUNDS[usize::from(u8::from(coords.z))]).ok()?;
    if coords.y < 0 || coords.y >= tile_count {
        return None;
    }
    Some(TileCoords {
        x: coords.x.rem_euclid(tile_count) as u32,
        y: coords.y as u32,
        z: coords.z,
    })
}

fn set_feature_opacity(
    feature: &Feature,
    layer: &crate::sdf::SymbolLayerData,
    metadata: &mut [SDFShaderFeatureMetadata],
    opacity: f32,
) {
    for index in feature.indices.clone() {
        let vertex_index = layer.new_buffer.buffer.indices[index] as usize;
        if let Some(vertex) = metadata.get_mut(vertex_index) {
            vertex.opacity = opacity;
        }
    }
}

#[cfg(test)]
mod tests;
