use crate::euclid::Point2D;
use crate::sdf::geometry_tile_data::GeometryCoordinates;
use crate::sdf::TileSpace;

pub struct PlacedSymbol {
    pub anchorPoint: Point2D<f64, TileSpace>,
    pub segment: usize,
    // pub lowerSize: f64,
    // pub upperSize: f64,
    pub lineOffset: [f64; 2],
    // pub writingModes: WritingModeType,
    pub line: GeometryCoordinates,
    pub tileDistances: Vec<f64>,
    pub glyphOffsets: Vec<f64>,
    // pub hidden: bool,
    // pub vertexStartIndex: usize,
    // /// The crossTileID is only filled/used on the foreground for variable text anchors
    // pub crossTileID: u32,
    // /// The placedOrientation is only used when symbol layer's property is set to
    // /// support placement for orientation variants.
    // pub placedOrientation:  Option<TextWritingModeType>,
    // pub angle: f64,
    // /// Reference to placed icon, only applicable for text symbols.
    // pub placedIconIndex: Option<usize>,
}
