//! [Stages](Stage) for requesting and preparing data

use std::{
    cell::RefCell,
    marker::PhantomData,
    rc::Rc,
    sync::{mpsc, Arc, Mutex},
};

use geozero::{mvt::tile, GeozeroDatasource};
use request_stage::RequestStage;

use crate::{
    coords::{WorldCoords, WorldTileCoords, Zoom, ZoomLevel},
    environment::Environment,
    error::Error,
    io::{
        apc::{AsyncProcedureCall, Context, Message},
        geometry_index::{GeometryIndex, IndexedGeometry, TileIndex},
        pipeline::{PipelineContext, PipelineProcessor, Processable},
        source_client::{HttpClient, HttpSourceClient},
        tile_pipelines::build_vector_tile_pipeline,
        transferables::{
            DefaultTessellatedLayer, DefaultTileTessellated, DefaultTransferables,
            DefaultUnavailableLayer, TessellatedLayer, TileTessellated, Transferables,
            UnavailableLayer,
        },
        TileRequest,
    },
    kernel::Kernel,
    render::ShaderVertex,
    schedule::Schedule,
    stages::populate_tile_store_stage::PopulateTileStore,
    tessellation::{IndexDataType, OverAlignedVertexBuffer},
};

mod populate_tile_store_stage;
mod request_stage;

/// Register stages required for requesting and preparing new tiles.
pub fn register_stages<E: Environment>(schedule: &mut Schedule, kernel: Rc<Kernel<E>>) {
    schedule.add_stage("request", RequestStage::<E>::new(kernel.clone()));
    schedule.add_stage("populate_tile_store", PopulateTileStore::<E>::new(kernel));
}

pub struct HeadedPipelineProcessor<T: Transferables, HC: HttpClient, C: Context<T, HC>> {
    context: C,
    phantom_t: PhantomData<T>,
    phantom_hc: PhantomData<HC>,
}

impl<'c, T: Transferables, HC: HttpClient, C: Context<T, HC>> PipelineProcessor
    for HeadedPipelineProcessor<T, HC, C>
{
    fn tile_finished(&mut self, coords: &WorldTileCoords) -> Result<(), Error> {
        self.context
            .send(Message::TileTessellated(T::TileTessellated::new(*coords)))
    }

    fn layer_unavailable(
        &mut self,
        coords: &WorldTileCoords,
        layer_name: &str,
    ) -> Result<(), Error> {
        self.context
            .send(Message::UnavailableLayer(T::UnavailableLayer::new(
                *coords,
                layer_name.to_owned(),
            )))
    }

    fn layer_tesselation_finished(
        &mut self,
        coords: &WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        feature_indices: Vec<u32>,
        layer_data: tile::Layer,
    ) -> Result<(), Error> {
        self.context
            .send(Message::TessellatedLayer(T::TessellatedLayer::new(
                *coords,
                buffer,
                feature_indices,
                layer_data,
            )))
    }

    fn layer_indexing_finished(
        &mut self,
        coords: &WorldTileCoords,
        geometries: Vec<IndexedGeometry<f64>>,
    ) -> Result<(), Error> {
        // FIXME (wasm-executor): Readd
        /*        if let Ok(mut geometry_index) = self.state.geometry_index.lock() {
            geometry_index.index_tile(coords, TileIndex::Linear { list: geometries })
        }*/
        Ok(())
    }
}

// FIXME (wasm-executor): Readd
/*pub fn query_point(
    &self,
    world_coords: &WorldCoords,
    z: ZoomLevel,
    zoom: Zoom,
) -> Option<Vec<IndexedGeometry<f64>>> {
    if let Ok(geometry_index) = self.geometry_index.lock() {
        geometry_index
            .query_point(world_coords, z, zoom)
            .map(|geometries| {
                geometries
                    .iter()
                    .cloned()
                    .cloned()
                    .collect::<Vec<IndexedGeometry<f64>>>()
            })
    } else {
        unimplemented!()
    }
}*/
//}
