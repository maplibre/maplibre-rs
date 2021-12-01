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

/// Contains relative coordinates to which the cursor is moved
#[derive(Debug)]
pub struct MoveTo {
    pub x: Number,
    pub y: Number,
}

/// Contains relative coordinates to which a line is drawn
#[derive(Debug)]
pub struct LineTo {
    pub x: Number,
    pub y: Number,
}

#[derive(Debug)]
pub enum Command {
    MoveTo(MoveTo),
    LineTo(LineTo),
    Close,
}

#[derive(Debug)]
pub struct GeometryLineString {
    pub commands: Vec<Command>,
}

#[derive(Debug)]
pub struct GeometryPolygon {
    pub commands: Vec<Command>,
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
