use super::{
    scheduler,
    scheduler::{ScheduleMethod, Scheduler},
    source_client::{HttpClient, HttpSourceClient, SourceClient},
    TileRequest, TileRequestID,
};
use crate::{
    coords::WorldTileCoords, error::Error, stages::SharedThreadState,
    style::source::TileAddressingScheme,
};

pub trait Source<SM, HC>
where
    SM: ScheduleMethod,
    HC: HttpClient,
{
    fn load(
        &self,
        http_source_client: HttpSourceClient<HC>,
        scheduler: &Scheduler<SM>,
        state: SharedThreadState,
        coords: &WorldTileCoords,
        request_id: TileRequestID,
    );
}

#[derive(Clone)]
pub struct TessellateSource {
    pub url: String,
    pub filetype: String,
}

impl TessellateSource {
    pub fn new(url: &str, filetype: &str) -> Self {
        Self {
            url: url.to_string(),
            filetype: filetype.to_string(),
        }
    }

    pub fn format(&self, coords: &WorldTileCoords) -> String {
        let tile_coords = coords.into_tile(TileAddressingScheme::TMS).unwrap();
        format!(
            "{url}/{z}/{x}/{y}.{filetype}",
            url = self.url,
            z = tile_coords.z,
            x = tile_coords.x,
            y = tile_coords.y,
            filetype = self.filetype,
        )
    }
}

impl Default for TessellateSource {
    fn default() -> Self {
        Self::new("https://maps.tuerantuer.org/europe_germany", "pbf")
    }
}

#[derive(Clone)]
pub struct RasterSource {
    pub url: String,
    pub filetype: String,
    pub key: String,
}

impl RasterSource {
    pub fn new(url: &str, filetype: &str, key: &str) -> Self {
        Self {
            url: url.to_string(),
            filetype: filetype.to_string(),
            key: key.to_string(),
        }
    }

    pub fn format(&self, coords: &WorldTileCoords) -> String {
        let tile_coords = coords.into_tile(TileAddressingScheme::TMS).unwrap();
        format!(
            "{url}/{z}/{x}/{y}.{filetype}?key={key}",
            url = self.url,
            z = tile_coords.z,
            x = tile_coords.x,
            y = tile_coords.y,
            filetype = self.filetype,
            key = self.key,
        )
    }
}

impl Default for RasterSource {
    fn default() -> Self {
        Self::new(
            "https://api.maptiler.com/tiles/satellite-v2",
            "jpg",
            "qnePkfbGpMsLCi3KFBs3",
        )
    }
}

#[derive(Clone)]
pub enum SourceType {
    Raster(RasterSource),
    Tessellate(TessellateSource),
}

impl SourceType {
    pub fn format(&self, coords: &WorldTileCoords) -> String {
        match self {
            SourceType::Raster(raster_source) => raster_source.format(coords),
            SourceType::Tessellate(tessellate_source) => tessellate_source.format(coords),
        }
    }
}

impl<SM, HC> Source<SM, HC> for SourceType
where
    SM: ScheduleMethod,
    HC: HttpClient,
{
    fn load(
        &self,
        http_source_client: HttpSourceClient<HC>,
        scheduler: &Scheduler<SM>,
        state: SharedThreadState,
        coords: &WorldTileCoords,
        request_id: TileRequestID,
    ) {
        let client = SourceClient::Http(http_source_client.clone());
        let coords = *coords;
        let source = self.clone();

        scheduler
            .schedule_method()
            .schedule(Box::new(move || {
                Box::pin(async move {
                    match client.fetch(&coords, &source).await {
                        Ok(data) => match source {
                            SourceType::Raster(raster_source) => {
                                state
                                    .process_raster_data(request_id, data.into_boxed_slice())
                                    .unwrap();
                            }
                            SourceType::Tessellate(tessellate_source) => {
                                state
                                    .process_vector_data(request_id, data.into_boxed_slice())
                                    .unwrap();
                            }
                        },
                        Err(e) => {
                            log::error!("{:?}", e);
                            state.tile_unavailable(&coords, request_id).unwrap();
                        }
                    }
                })
            }))
            .unwrap();
    }
}

impl<SM, HC> Source<SM, HC> for TessellateSource
where
    SM: ScheduleMethod,
    HC: HttpClient,
{
    fn load(
        &self,
        http_source_client: HttpSourceClient<HC>,
        scheduler: &Scheduler<SM>,
        state: SharedThreadState,
        coords: &WorldTileCoords,
        request_id: TileRequestID,
    ) {
        let source = SourceType::Tessellate(self.clone());

        source.load(
            http_source_client.clone(),
            scheduler,
            state,
            coords,
            request_id,
        );
    }
}

impl<SM, HC> Source<SM, HC> for RasterSource
where
    SM: ScheduleMethod,
    HC: HttpClient,
{
    fn load(
        &self,
        http_source_client: HttpSourceClient<HC>,
        scheduler: &Scheduler<SM>,
        state: SharedThreadState,
        coords: &WorldTileCoords,
        request_id: TileRequestID,
    ) {
        let source = SourceType::Raster(self.clone());

        source.load(
            http_source_client.clone(),
            scheduler,
            state,
            coords,
            request_id,
        );
    }
}
