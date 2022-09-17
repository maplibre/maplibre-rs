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
    environment::DefaultTransferables,
    error::Error,
    io::{
        apc::{AsyncProcedureCall, Context, Message},
        geometry_index::{GeometryIndex, IndexedGeometry, TileIndex},
        pipeline::{PipelineContext, PipelineProcessor, Processable},
        source_client::HttpSourceClient,
        tile_pipelines::build_vector_tile_pipeline,
        transferables::{
            DefaultTessellatedLayer, DefaultTileTessellated, DefaultUnavailableLayer,
            TessellatedLayer, TileTessellated, Transferables, UnavailableLayer,
        },
        TileRequest,
    },
    render::ShaderVertex,
    schedule::Schedule,
    stages::populate_tile_store_stage::PopulateTileStore,
    tessellation::{IndexDataType, OverAlignedVertexBuffer},
    Environment, HttpClient, Scheduler,
};

mod populate_tile_store_stage;
mod request_stage;

/// Register stages required for requesting and preparing new tiles.
pub fn register_stages<E: Environment>(
    schedule: &mut Schedule,
    http_source_client: HttpSourceClient<E::HttpClient>,
    apc: Rc<RefCell<E::AsyncProcedureCall>>,
) {
    schedule.add_stage(
        "request",
        RequestStage::<E>::new(http_source_client, apc.clone()),
    );
    schedule.add_stage("populate_tile_store", PopulateTileStore::<E>::new(apc));
}

pub struct HeadedPipelineProcessor<T: Transferables, HC: HttpClient, C: Context<T, HC>> {
    context: C,
    phantom_t: PhantomData<T>,
    phantom_hc: PhantomData<HC>,
}

impl<'c, T: Transferables, HC: HttpClient, C: Context<T, HC>> PipelineProcessor
    for HeadedPipelineProcessor<T, HC, C>
{
    fn tile_finished(&mut self, coords: &WorldTileCoords) {
        self.context
            .send(Message::TileTessellated(T::TileTessellated::new(*coords)))
    }

    fn layer_unavailable(&mut self, coords: &WorldTileCoords, layer_name: &str) {
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
    ) {
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
    ) {
        // TODO: Readd
        /*        if let Ok(mut geometry_index) = self.state.geometry_index.lock() {
            geometry_index.index_tile(coords, TileIndex::Linear { list: geometries })
        }*/
    }
}

// FIXME: clean this up
///// Stores and provides access to the thread safe data shared between the schedulers.
//[derive(Clone)]
//pub struct SharedThreadState {
/*    pub tile_request_state: Arc<Mutex<TileRequestState>>,
pub message_sender: mpsc::Sender<TessellateMessage>,*/
// TODO: Readd
//pub geometry_index: Arc<Mutex<GeometryIndex>>,
//}

//impl SharedThreadState {
/* fn get_tile_request(&self, request_id: TileRequestID) -> Option<TileRequest> {
    self.tile_request_state
        .lock()
        .ok()
        .and_then(|tile_request_state| tile_request_state.get_tile_request(request_id).cloned())
}

#[tracing::instrument(skip_all)]
pub fn process_tile(&self, request_id: TileRequestID, data: Box<[u8]>) -> Result<(), Error> {
    if let Some(tile_request) = self.get_tile_request(request_id) {
        let mut pipeline_context = PipelineContext::new(HeadedPipelineProcessor {
            state: self.clone(),
        });
        let pipeline = build_vector_tile_pipeline();
        pipeline.process((tile_request, request_id, data), &mut pipeline_context);
    }

    Ok(())
}

pub fn tile_unavailable(
    &self,
    coords: &WorldTileCoords,
    request_id: TileRequestID,
) -> Result<(), Error> {
    if let Some(tile_request) = self.get_tile_request(request_id) {
        for to_load in &tile_request.layers {
            tracing::warn!("layer {} at {} unavailable", to_load, coords);
            self.message_sender.send(TessellateMessage::Layer(
                LayerTessellateMessage::UnavailableLayer {
                    coords: tile_request.coords,
                    layer_name: to_load.to_string(),
                },
            ))?;
        }
    }

    Ok(())
}*/

// TODO: Readd
/*    #[tracing::instrument(skip_all)]
pub fn query_point(
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
