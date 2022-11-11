use super::source_client::HttpSourceClient;
use crate::io::apc::AsyncProcedureFuture;
use crate::{
    coords::WorldTileCoords, environment::Environment, style::source::TileAddressingScheme,
};

pub trait Source<E>
where
    E: Environment,
{
    fn load(
        &self,
        http_source_client: HttpSourceClient<E::HttpClient>,
        coords: &WorldTileCoords,
    ) -> AsyncProcedureFuture;
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

/*
impl<E> Source<E> for SourceType
where
    E: Environment,
{
    fn load(
        &self,
        http_source_client: HttpSourceClient<E::HttpClient>,
        coords: &WorldTileCoords,
    ) -> AsyncProcedureFuture {
        let client = SourceClient::Http(http_source_client.clone());
        let coords = *coords;
        let source = self.clone();



        /*scheduler
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
        .unwrap();*/
    }
}



impl<E> Source<E> for TessellateSource
where
    E: Environment,
{
    fn load(
        &self,
        http_source_client: HttpSourceClient<E::HttpClient>,
        coords: &WorldTileCoords,
    ) -> AsyncProcedureFuture {
        let source = SourceType::Tessellate(self.clone());

        source.load(http_source_client.clone(), scheduler, coords)
    }
}

impl<E> Source<E> for RasterSource
where
    E: Environment,
{
    fn load(
        &self,
        http_source_client: HttpSourceClient<E::HttpClient>,
        coords: &WorldTileCoords,
    ) -> AsyncProcedureFuture {
        let source = SourceType::Raster(self.clone());

        source.load(http_source_client.clone(), scheduler, coords)
    }
}
*/
