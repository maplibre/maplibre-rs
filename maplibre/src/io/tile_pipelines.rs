use std::collections::HashSet;

use cgmath::InnerSpace;
use geozero::GeozeroDatasource;
use prost::Message;

use crate::{
    io::{
        geometry_index::IndexProcessor,
        pipeline::{DataPipeline, PipelineContext, PipelineEnd, PipelineError, Processable},
        TileRequest,
    },
    render::ShaderVertex,
    tessellation::{zero_tessellator::ZeroTessellator, IndexDataType},
};

#[derive(Default)]
pub struct ParseTile;

impl Processable for ParseTile {
    type Input = (TileRequest, Box<[u8]>);
    type Output = (TileRequest, geozero::mvt::Tile);

    fn process(
        &self,
        (tile_request, data): Self::Input,
        _context: &mut PipelineContext,
    ) -> Result<Self::Output, PipelineError> {
        let tile = geozero::mvt::Tile::decode(data.as_ref()).expect("failed to load tile");
        Ok((tile_request, tile))
    }
}

#[derive(Default)]
pub struct IndexLayer;

impl Processable for IndexLayer {
    type Input = (TileRequest, geozero::mvt::Tile);
    type Output = (TileRequest, geozero::mvt::Tile);

    fn process(
        &self,
        (tile_request, mut tile): Self::Input,
        context: &mut PipelineContext,
    ) -> Result<Self::Output, PipelineError> {
        let mut index = IndexProcessor::new();

        for layer in &mut tile.layers {
            layer.process(&mut index).unwrap();
        }

        for layer in &mut tile.layers {
            layer.process(&mut index).unwrap();
        }

        context
            .processor_mut()
            .layer_indexing_finished(&tile_request.coords, index.get_geometries())?;
        Ok((tile_request, tile))
    }
}

#[derive(Default)]
pub struct TessellateLayer;

impl Processable for TessellateLayer {
    type Input = (TileRequest, geozero::mvt::Tile);
    type Output = (TileRequest, geozero::mvt::Tile);

    fn process(
        &self,
        (tile_request, mut tile): Self::Input,
        context: &mut PipelineContext,
    ) -> Result<Self::Output, PipelineError> {
        let coords = &tile_request.coords;

        for layer in &mut tile.layers {
            let cloned_layer = layer.clone();
            let layer_name: &str = &cloned_layer.name;
            if !tile_request.layers.contains(layer_name) {
                continue;
            }

            tracing::info!("layer {} at {} ready", layer_name, coords);

            let mut tessellator = ZeroTessellator::<IndexDataType>::default();
            if let Err(e) = layer.process(&mut tessellator) {
                context
                    .processor_mut()
                    .layer_unavailable(coords, layer_name)?;

                tracing::error!(
                    "layer {} at {} tesselation failed {:?}",
                    layer_name,
                    &coords,
                    e
                );
            } else {
                let layer_style = tile_request
                    .style
                    .layers
                    .iter()
                    .find(|layer_style| {
                        layer.name == *layer_style.source_layer.as_ref().unwrap_or(&"".to_string())
                    })
                    .unwrap();

                // Extrude all the buildings on the z axis if osm_3d_extrusion is enabled on the layer
                if layer_style.extrusion {
                    // We create a list of all the outer/contour edges. Meaning that these
                    // edges are not inside the 2d mesh, and a "wall" should be instantiated for them.
                    // In order to do that, we create a `HashSet` of every edge that appears only
                    // once in the entire layer.
                    let mut contour_edges: HashSet<(u32, u32)> =
                        HashSet::with_capacity(tessellator.buffer.indices.len());
                    for i in 0..tessellator.buffer.indices.len() {
                        let a = tessellator.buffer.indices[i];
                        let b = tessellator.buffer.indices
                            [if (i + 1) % 3 == 0 { i - 2 } else { i + 1 }];

                        // If the contour edge already exist, it is an inner edge and not a contour edge so we remove it
                        if contour_edges.contains(&(b, a)) {
                            contour_edges.remove(&(b, a));
                        } else {
                            contour_edges.insert((a, b));
                        }
                    }

                    // For each "wall" of the buildings, we create 2 triangles in the clockwise
                    // direction so that their normals are facing outward.
                    let mut extruded_vertices = vec![];
                    let mut side_faces_indices = vec![];
                    for mut edge in contour_edges {
                        let edge_vector = [
                            tessellator.buffer.vertices[edge.1 as usize].position[0]
                                - tessellator.buffer.vertices[edge.0 as usize].position[0],
                            tessellator.buffer.vertices[edge.1 as usize].position[1]
                                - tessellator.buffer.vertices[edge.0 as usize].position[1],
                            0.0,
                        ];
                        let normal_vector =
                            cgmath::Vector3::from([-edge_vector[1], edge_vector[0], 0.0])
                                .normalize()
                                .into();
                        let a_position = tessellator.buffer.vertices[edge.0 as usize].position;
                        let b_position = tessellator.buffer.vertices[edge.1 as usize].position;
                        extruded_vertices.push(ShaderVertex::new(
                            [a_position[0], a_position[1], 0.0],
                            normal_vector,
                        ));
                        let a = (extruded_vertices.len() + tessellator.buffer.vertices.len() - 1)
                            as u32;
                        extruded_vertices.push(ShaderVertex::new(
                            [b_position[0], b_position[1], 0.0],
                            normal_vector,
                        ));
                        let b = (extruded_vertices.len() + tessellator.buffer.vertices.len() - 1)
                            as u32;
                        extruded_vertices.push(ShaderVertex::new(
                            [a_position[0], a_position[1], 40.0],
                            normal_vector,
                        ));
                        let a_extruded = (extruded_vertices.len()
                            + tessellator.buffer.vertices.len()
                            - 1) as u32;
                        extruded_vertices.push(ShaderVertex::new(
                            [b_position[0], b_position[1], 40.0],
                            normal_vector,
                        ));
                        let b_extruded = (extruded_vertices.len()
                            + tessellator.buffer.vertices.len()
                            - 1) as u32;
                        side_faces_indices.push(a);
                        side_faces_indices.push(b_extruded);
                        side_faces_indices.push(a_extruded);
                        side_faces_indices.push(b);
                        side_faces_indices.push(b_extruded);
                        side_faces_indices.push(a);
                    }

                    // We move the vertices to the top, because the bottom will not be visible anyway.
                    for i in 0..tessellator.buffer.vertices.len() {
                        tessellator.buffer.vertices[i] = ShaderVertex::new(
                            [
                                tessellator.buffer.vertices[i].position[0],
                                tessellator.buffer.vertices[i].position[1],
                                40.0,
                            ],
                            tessellator.buffer.vertices[i].normal,
                        );
                    }

                    // We insert the new walls to the buffer.
                    tessellator.buffer.vertices.extend(extruded_vertices.iter());
                    tessellator.buffer.indices.extend(side_faces_indices.iter());
                }

                // We send the tessellated layer to the pipeline.
                context.processor_mut().layer_tesselation_finished(
                    coords,
                    tessellator.buffer.into(),
                    tessellator.feature_indices,
                    cloned_layer,
                )?;
            }
        }

        let available_layers: HashSet<_> = tile
            .layers
            .iter()
            .map(|layer| layer.name.clone())
            .collect::<HashSet<_>>();

        for missing_layer in tile_request.layers.difference(&available_layers) {
            context
                .processor_mut()
                .layer_unavailable(coords, missing_layer)?;

            tracing::info!(
                "requested layer {} at {} not found in tile",
                missing_layer,
                &coords
            );
        }

        tracing::info!("tile tessellated at {} finished", &tile_request.coords);

        context
            .processor_mut()
            .tile_finished(&tile_request.coords)?;

        Ok((tile_request, tile))
    }
}

pub fn build_vector_tile_pipeline() -> impl Processable<Input = <ParseTile as Processable>::Input> {
    DataPipeline::new(
        ParseTile,
        DataPipeline::new(
            TessellateLayer,
            DataPipeline::new(IndexLayer, PipelineEnd::default()),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::build_vector_tile_pipeline;
    use crate::{
        coords::ZoomLevel,
        io::{
            pipeline::{PipelineContext, PipelineProcessor, Processable},
            TileRequest,
        },
    };
    pub struct DummyPipelineProcessor;

    impl PipelineProcessor for DummyPipelineProcessor {}

    #[test]
    #[ignore]
    fn test() {
        let mut context = PipelineContext::new(DummyPipelineProcessor);

        let pipeline = build_vector_tile_pipeline();
        let _output = pipeline.process(
            (
                TileRequest {
                    coords: (0, 0, ZoomLevel::default()).into(),
                    layers: Default::default(),
                },
                Box::new([0]), // TODO: Add proper tile byte array
            ),
            &mut context,
        );
    }
}
