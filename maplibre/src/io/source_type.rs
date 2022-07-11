use crate::coords::WorldTileCoords;
use crate::error::Error;
use crate::stages::SharedThreadState;

use super::scheduler::ScheduleMethod;
use super::scheduler::Scheduler;
use super::source_client::HttpClient;
use super::source_client::HttpSourceClient;
use super::source_client::SourceClient;
use super::TileRequest;
use super::TileRequestID;

pub trait Source<SM, HC>
where
    SM: ScheduleMethod,
    HC: HttpClient,
{
    fn load(
        http_source_client: HttpSourceClient<HC>,
        scheduler: &Scheduler<SM>,
        state: SharedThreadState,
        coords: &WorldTileCoords,
        request_id: TileRequestID,
    );
}

pub struct TessellateSource;

impl<SM, HC> Source<SM, HC> for TessellateSource
where
    SM: ScheduleMethod,
    HC: HttpClient,
{
    fn load(
        http_source_client: HttpSourceClient<HC>,
        scheduler: &Scheduler<SM>,
        state: SharedThreadState,
        coords: &WorldTileCoords,
        request_id: TileRequestID,
    ) {
        let client = SourceClient::Http(http_source_client.clone());
        let coords = *coords;

        scheduler
            .schedule_method()
            .schedule(Box::new(move || {
                Box::pin(async move {
                    match client.fetch(&coords).await {
                        Ok(data) => state
                            .process_vector_data(request_id, data.into_boxed_slice())
                            .unwrap(),
                        Err(e) => {
                            log::error!("{:?}", &e);
                            state.tile_unavailable(&coords, request_id).unwrap()
                        }
                    }
                })
            }))
            .unwrap();
    }
}

pub struct RasterSource;

impl<SM, HC> Source<SM, HC> for RasterSource
where
    SM: ScheduleMethod,
    HC: HttpClient,
{
    fn load(
        http_source_client: HttpSourceClient<HC>,
        scheduler: &Scheduler<SM>,
        state: SharedThreadState,
        coords: &WorldTileCoords,
        request_id: TileRequestID,
    ) {
        let client = SourceClient::Http(http_source_client.clone());
        let coords = *coords;

        scheduler
            .schedule_method()
            .schedule(Box::new(move || {
                Box::pin(async move {
                    match client.fetch(&coords).await {
                        Ok(data) => state
                            .process_raster_data(request_id, data.into_boxed_slice())
                            .unwrap(),
                        Err(e) => {
                            log::error!("{:?}", &e);
                            state.tile_unavailable(&coords, request_id).unwrap()
                        }
                    }
                })
            }))
            .unwrap();
    }
}
