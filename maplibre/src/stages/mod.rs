//! [Stages](Stage) for requesting and preparing data

use std::{marker::PhantomData, rc::Rc};

use geozero::mvt::tile;
use request_stage::RequestStage;

use crate::{
    coords::WorldTileCoords,
    environment::Environment,
    error::Error,
    io::{
        apc::{Context, Message},
        geometry_index::{IndexedGeometry, TileIndex},
        pipeline::PipelineProcessor,
        source_client::HttpClient,
        transferables::{TessellatedLayer, TileTessellated, Transferables, UnavailableLayer},
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
            .send(Message::LayerUnavailable(T::LayerUnavailable::new(
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
            .send(Message::LayerTessellated(T::LayerTessellated::new(
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
        self.context
            .send(Message::LayerIndexed(T::LayerIndexed::from((
                *coords,
                TileIndex::Linear { list: geometries },
            ))))
    }
}
