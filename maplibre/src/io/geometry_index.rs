use std::collections::{BTreeMap, HashMap};

use cgmath::num_traits::Signed;
use cgmath::Bounded;
use geo::prelude::*;
use geo_types::{CoordFloat, Coordinate, Geometry, LineString, Point, Polygon};
use geozero::error::GeozeroError;
use geozero::geo_types::GeoWriter;
use geozero::{ColumnValue, FeatureProcessor, GeomProcessor, PropertyProcessor};
use rstar::{Envelope, PointDistance, RTree, RTreeObject, AABB};

use crate::coords::{InnerCoords, Quadkey, WorldCoords, WorldTileCoords, Zoom, EXTENT, TILE_SIZE};
use crate::util::math::bounds_from_points;

pub struct GeometryIndex {
    index: BTreeMap<Quadkey, TileIndex>,
}

impl GeometryIndex {
    pub fn new() -> Self {
        Self {
            index: Default::default(),
        }
    }

    pub fn index_tile(&mut self, coords: &WorldTileCoords, tile_index: TileIndex) {
        coords
            .build_quad_key()
            .and_then(|key| self.index.insert(key, tile_index));
    }

    pub fn query_point(
        &self,
        world_coords: &WorldCoords,
        z: u8,
        zoom: Zoom,
    ) -> Option<Vec<&IndexedGeometry<f64>>> {
        let world_tile_coords = world_coords.into_world_tile(z, zoom);

        if let Some(index) = world_tile_coords
            .build_quad_key()
            .and_then(|key| self.index.get(&key))
        {
            let scale = zoom.scale_delta(&Zoom::new(z as f64)); // FIXME: can be wrong, if tiles of different z are visible

            let delta_x = world_coords.x / TILE_SIZE * scale - world_tile_coords.x as f64;
            let delta_y = world_coords.y / TILE_SIZE * scale - world_tile_coords.y as f64;

            let x = delta_x * EXTENT;
            let y = delta_y * EXTENT;
            Some(index.point_query(InnerCoords { x, y }))
        } else {
            None
        }
    }
}

pub enum TileIndex {
    Spatial { tree: RTree<IndexedGeometry<f64>> },
    Linear { list: Vec<IndexedGeometry<f64>> },
}

impl TileIndex {
    pub fn point_query(&self, inner_coords: InnerCoords) -> Vec<&IndexedGeometry<f64>> {
        let point = geo_types::Point::new(inner_coords.x, inner_coords.y);
        let coordinate: Coordinate<_> = point.into();

        // FIXME: Respect layer order of style
        match self {
            TileIndex::Spatial { tree } => tree
                .nearest_neighbor_iter(&point)
                .filter(|geometry| match &geometry.exact {
                    ExactGeometry::Polygon(exact) => exact.contains(&coordinate),
                    ExactGeometry::LineString(exact) => exact.distance_2(&point) <= 64.0,
                })
                .collect::<Vec<_>>(),
            TileIndex::Linear { list } => list
                .iter()
                .filter(|geometry| match &geometry.exact {
                    ExactGeometry::Polygon(exact) => exact.contains(&coordinate),
                    ExactGeometry::LineString(exact) => exact.distance_2(&point) <= 64.0,
                })
                .collect::<Vec<_>>(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexedGeometry<T>
where
    T: CoordFloat + Bounded + Signed,
{
    pub bounds: AABB<Point<T>>,
    pub exact: ExactGeometry<T>,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum ExactGeometry<T>
where
    T: CoordFloat + Bounded + Signed,
{
    Polygon(Polygon<T>),
    LineString(LineString<T>),
}

impl<T> IndexedGeometry<T>
where
    T: CoordFloat + Bounded + Signed + PartialOrd,
{
    fn from_polygon(polygon: Polygon<T>, properties: HashMap<String, String>) -> Option<Self> {
        let (min, max) = bounds_from_points(polygon.exterior().points())?;

        Some(Self {
            exact: ExactGeometry::Polygon(polygon),
            bounds: AABB::from_corners(Point::from(min), Point::from(max)),
            properties,
        })
    }
    fn from_linestring(
        linestring: LineString<T>,
        properties: HashMap<String, String>,
    ) -> Option<Self> {
        let bounds = linestring.envelope();

        Some(Self {
            exact: ExactGeometry::LineString(linestring),
            bounds,
            properties,
        })
    }
}

impl<T> RTreeObject for IndexedGeometry<T>
where
    T: CoordFloat + Bounded + Signed + PartialOrd,
{
    type Envelope = AABB<Point<T>>;

    fn envelope(&self) -> Self::Envelope {
        self.bounds
    }
}

impl<T> PointDistance for IndexedGeometry<T>
where
    T: CoordFloat + Bounded + Signed + PartialOrd,
{
    fn distance_2(
        &self,
        point: &<Self::Envelope as Envelope>::Point,
    ) -> <<Self::Envelope as Envelope>::Point as rstar::Point>::Scalar {
        self.bounds.center().distance_2(point)
    }

    fn contains_point(&self, point: &<Self::Envelope as Envelope>::Point) -> bool {
        self.bounds.contains_point(point)
    }
}

pub struct IndexProcessor {
    geo_writer: GeoWriter,
    geometries: Vec<IndexedGeometry<f64>>,
    properties: Option<HashMap<String, String>>,
}

impl IndexProcessor {
    pub fn new() -> Self {
        Self {
            geo_writer: GeoWriter::new(),
            geometries: Vec::new(),
            properties: None,
        }
    }

    pub fn build_tree(self) -> RTree<IndexedGeometry<f64>> {
        RTree::bulk_load(self.geometries)
    }

    pub fn get_geometries(self) -> Vec<IndexedGeometry<f64>> {
        self.geometries
    }
}

impl GeomProcessor for IndexProcessor {
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<(), GeozeroError> {
        self.geo_writer.xy(x, y, idx)
    }
    fn point_begin(&mut self, idx: usize) -> Result<(), GeozeroError> {
        self.geo_writer.point_begin(idx)
    }
    fn multipoint_begin(&mut self, size: usize, idx: usize) -> Result<(), GeozeroError> {
        self.geo_writer.multipoint_begin(size, idx)
    }
    fn linestring_begin(
        &mut self,
        tagged: bool,
        size: usize,
        idx: usize,
    ) -> Result<(), GeozeroError> {
        self.geo_writer.linestring_begin(tagged, size, idx)
    }
    fn linestring_end(&mut self, tagged: bool, idx: usize) -> Result<(), GeozeroError> {
        self.geo_writer.linestring_end(tagged, idx)
    }
    fn multilinestring_begin(&mut self, size: usize, idx: usize) -> Result<(), GeozeroError> {
        self.geo_writer.multilinestring_begin(size, idx)
    }
    fn multilinestring_end(&mut self, idx: usize) -> Result<(), GeozeroError> {
        self.geo_writer.multilinestring_end(idx)
    }
    fn polygon_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<(), GeozeroError> {
        self.geo_writer.polygon_begin(tagged, size, idx)
    }
    fn polygon_end(&mut self, tagged: bool, idx: usize) -> Result<(), GeozeroError> {
        self.geo_writer.polygon_end(tagged, idx)
    }
    fn multipolygon_begin(&mut self, size: usize, idx: usize) -> Result<(), GeozeroError> {
        self.geo_writer.multipolygon_begin(size, idx)
    }
}

impl PropertyProcessor for IndexProcessor {
    fn property(
        &mut self,
        _idx: usize,
        name: &str,
        value: &ColumnValue,
    ) -> Result<bool, GeozeroError> {
        self.properties
            .as_mut()
            .unwrap()
            .insert(name.to_string(), value.to_string());
        Ok(true)
    }
}

impl FeatureProcessor for IndexProcessor {
    /// Begin of dataset processing
    fn dataset_begin(&mut self, _name: Option<&str>) -> Result<(), GeozeroError> {
        Ok(())
    }
    /// End of dataset processing
    fn dataset_end(&mut self) -> Result<(), GeozeroError> {
        Ok(())
    }
    /// Begin of feature processing
    fn feature_begin(&mut self, _idx: u64) -> Result<(), GeozeroError> {
        Ok(())
    }
    /// End of feature processing
    fn feature_end(&mut self, _idx: u64) -> Result<(), GeozeroError> {
        Ok(())
    }
    /// Begin of feature property processing
    fn properties_begin(&mut self) -> Result<(), GeozeroError> {
        self.properties = Some(HashMap::new());
        Ok(())
    }
    /// End of feature property processing
    fn properties_end(&mut self) -> Result<(), GeozeroError> {
        Ok(())
    }
    /// Begin of feature geometry processing
    fn geometry_begin(&mut self) -> Result<(), GeozeroError> {
        Ok(())
    }
    /// End of feature geometry processing
    fn geometry_end(&mut self) -> Result<(), GeozeroError> {
        let geometry = self.geo_writer.geometry().cloned().unwrap();

        match geometry {
            Geometry::Polygon(polygon) => self.geometries.push(
                IndexedGeometry::from_polygon(polygon, self.properties.take().unwrap()).unwrap(),
            ),
            Geometry::LineString(linestring) => self.geometries.push(
                IndexedGeometry::from_linestring(linestring, self.properties.take().unwrap())
                    .unwrap(),
            ),
            _ => {}
        };

        Ok(())
    }
}
