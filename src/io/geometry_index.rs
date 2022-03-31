use std::collections::HashMap;

use cgmath::num_traits::Signed;
use cgmath::Bounded;
use geo::prelude::*;
use geo_types::{CoordFloat, Coordinate, Geometry, LineString, Point, Polygon};
use geozero::error::GeozeroError;
use geozero::geo_types::GeoWriter;
use geozero::{ColumnValue, FeatureProcessor, GeomProcessor, PropertyProcessor};
use rstar::{Envelope, PointDistance, RTree, RTreeObject, AABB};

use crate::coords::InnerCoords;
use crate::util::math::bounds_from_points;

pub enum TileIndex {
    Spatial { tree: RTree<IndexGeometry<f64>> },
    Linear { list: Vec<IndexGeometry<f64>> },
}

impl TileIndex {
    pub fn point_query(&self, inner_coords: InnerCoords) -> Vec<&IndexGeometry<f64>> {
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

#[derive(Debug)]
pub struct IndexGeometry<T>
where
    T: CoordFloat + Bounded + Signed,
{
    pub bounds: AABB<Point<T>>,
    pub exact: ExactGeometry<T>,
    pub properties: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ExactGeometry<T>
where
    T: CoordFloat + Bounded + Signed,
{
    Polygon(Polygon<T>),
    LineString(LineString<T>),
}

impl<T> IndexGeometry<T>
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

impl<T> RTreeObject for IndexGeometry<T>
where
    T: CoordFloat + Bounded + Signed + PartialOrd,
{
    type Envelope = AABB<Point<T>>;

    fn envelope(&self) -> Self::Envelope {
        self.bounds
    }
}

impl<T> PointDistance for IndexGeometry<T>
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
    geometries: Vec<IndexGeometry<f64>>,
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

    pub fn build_tree(self) -> RTree<IndexGeometry<f64>> {
        RTree::bulk_load(self.geometries)
    }

    pub fn get_geometries(self) -> Vec<IndexGeometry<f64>> {
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
        let geometry = self.geo_writer.geometry().clone();

        match geometry {
            Geometry::Polygon(polygon) => self.geometries.push(
                IndexGeometry::from_polygon(polygon, self.properties.take().unwrap()).unwrap(),
            ),
            Geometry::LineString(linestring) => self.geometries.push(
                IndexGeometry::from_linestring(linestring, self.properties.take().unwrap())
                    .unwrap(),
            ),
            _ => {}
        };

        Ok(())
    }
}
