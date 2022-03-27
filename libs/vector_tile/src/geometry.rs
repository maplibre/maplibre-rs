#[derive(Debug, Clone)]
pub enum GeometryPoint {
    Point(Point),
    MultiPoint(MultiPoint),
    Unknown,
}

#[derive(Debug, Clone)]
pub struct MultiPoint {
    points: Vec<Point>,
}

#[derive(Debug, Clone)]
pub struct Point {
    x: i32,
    y: i32,
}

/// Contains relative coordinates to which the cursor is moved
#[derive(Debug, Clone)]
pub struct MoveTo {
    pub x: i32,
    pub y: i32,
}

/// Contains relative coordinates to which a line is drawn
#[derive(Debug, Clone)]
pub struct LineTo {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone)]
pub enum Command {
    MoveTo(MoveTo),
    LineTo(LineTo),
    Close,
}

#[derive(Debug, Clone)]
pub struct GeometryLineString {
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone)]
pub struct GeometryPolygon {
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone)]
pub enum Geometry {
    GeometryPoint(GeometryPoint),
    GeometryLineString(GeometryLineString),
    GeometryPolygon(GeometryPolygon),
    Unknown,
}

impl Point {
    pub(crate) fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl MultiPoint {
    pub(crate) fn new(points: Vec<Point>) -> Self {
        Self { points }
    }
}
