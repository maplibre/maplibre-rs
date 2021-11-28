use num_traits::Num;

type Number = i32;

#[derive(Debug)]
pub enum GeometryPoint {
    Point(Point),
    MultiPoint(MultiPoint),
    Unknown,
}

#[derive(Debug)]
pub struct MultiPoint {
    points: Vec<Point>,
}

#[derive(Debug)]
pub struct Point {
    x: Number,
    y: Number,
}

#[derive(Debug)]
pub enum GeometryLineString {
    LineString(LineString),
    MultiLineString(MultiLineString),
    Unknown,
}

#[derive(Debug)]
pub struct MultiLineString {
    lines: Vec<LineString>,
}

#[derive(Debug)]
pub struct LineString {
    points: Vec<Point>,
}

#[derive(Debug)]
pub enum GeometryPolygon {
    Polygon(Polygon),
    MultiLineString(MultiPolygon),
    Unknown,
}

#[derive(Debug)]
pub struct Polygon {
    points: Vec<Point>,
}

#[derive(Debug)]
pub struct MultiPolygon {
    polygons: Vec<Polygon>,
}

#[derive(Debug)]
pub enum Geometry {
    GeometryPoint(GeometryPoint),
    GeometryLineString(GeometryLineString),
    GeometryPolygon(GeometryPolygon),
    Unknown,
}

impl Point {
    pub(crate) fn new(x: Number, y: Number) -> Self {
        Self { x, y }
    }
}

impl MultiPoint {
    pub(crate) fn new(points: Vec<Point>) -> Self {
        Self { points }
    }
}

impl LineString {
    pub(crate) fn new(points: Vec<Point>) -> Self {
        Self { points }
    }
}

impl MultiLineString {
    pub(crate) fn new(lines: Vec<LineString>) -> Self {
        Self { lines }
    }
}

impl Polygon {
    pub(crate) fn new(points: Vec<Point>) -> Self {
        Self { points }
    }
}

impl MultiPolygon {
    pub(crate) fn new(polygons: Vec<Polygon>) -> Self {
        Self { polygons }
    }
}
