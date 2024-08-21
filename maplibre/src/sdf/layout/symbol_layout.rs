use crate::sdf::bidi::BiDi;
use crate::sdf::buckets::symbol_bucket::PlacedSymbol;
use crate::sdf::geometry_tile_data::GeometryCoordinates;
use crate::sdf::glyph::{GlyphMap, WritingModeType};
use crate::sdf::glyph_atlas::GlyphPositions;
use crate::sdf::image::ImageMap;
use crate::sdf::image_atlas::ImagePositions;
use crate::sdf::layout::symbol_feature::SymbolFeature;
use crate::sdf::layout::symbol_instance::{ShapedTextOrientations, SymbolContent, SymbolInstance};
use crate::sdf::quads::{SymbolQuad, SymbolQuads};
use crate::sdf::shaping::PositionedIcon;
use crate::sdf::style_types::{SymbolAnchorType, TextWritingModeType};
use crate::sdf::text::Anchor;
use crate::sdf::MapMode;
use std::collections::{BTreeMap, HashMap};
use std::ops::Range;
use widestring::U16String;

// bucket
struct SymbolBucket;
struct SymbolBucket_Buffer;

// index
struct FeatureIndex;

// render
struct SortKeyRange;
struct LayerRenderData; // = Bucket + LayerProperties

// tile
struct GeometryTileLayer;
struct CanonicalTileID;

// props
struct LayerProperties;
struct TextSize_UnevaluatedType;
struct IconSize_UnevaluatedType;
struct TextRadialOffset_UnevaluatedType;
struct SymbolLayoutProperties_PossiblyEvaluated;

struct SymbolLayout {
    pub layerPaintProperties: BTreeMap<String, LayerProperties>,
    pub bucketLeaderID: String,
    pub symbolInstances: Vec<SymbolInstance>,
    pub sortKeyRanges: Vec<SortKeyRange>,

    // Stores the layer so that we can hold on to GeometryTileFeature instances
    // in SymbolFeature, which may reference data from this object.
    sourceLayer: Box<GeometryTileLayer>,
    overscaling: f64,
    zoom: f64,
    canonicalID: CanonicalTileID,
    mode: MapMode,
    pixelRatio: f64,

    tileSize: u32,
    tilePixelRatio: f64,

    iconsNeedLinear: bool,
    sortFeaturesByY: bool,
    sortFeaturesByKey: bool,
    allowVerticalPlacement: bool,
    iconsInText: bool,
    placementModes: Vec<TextWritingModeType>,

    textSize: TextSize_UnevaluatedType,
    iconSize: IconSize_UnevaluatedType,
    textRadialOffset: TextRadialOffset_UnevaluatedType,
    layout: SymbolLayoutProperties_PossiblyEvaluated,
    features: Vec<SymbolFeature>,

    bidi: BiDi, // Consider moving this up to geometry tile worker to reduce
    // reinstantiation costs, use of BiDi/ubiditransform object must
    // be rained to one thread
    compareText: BTreeMap<U16String, Vec<Anchor>>,
}

impl SymbolLayout {
    fn prepareSymbols(
        &self,
        glyphMap: &GlyphMap,
        glyphPositions: &GlyphPositions,
        imageMap: &ImageMap,
        imagePositions: &ImagePositions,
    ) {
        todo!()
    }

    fn createBucket(
        &self,
        imagePositions: ImagePositions,
        feature_index: Box<FeatureIndex>,
        renderData: &HashMap<String, LayerRenderData>,
        firstLoad: bool,
        showCollisionBoxes: bool,
        canonical: &CanonicalTileID,
    ) {
        todo!()
    }

    fn hasSymbolInstances(&self) -> bool {
        todo!()
    }
    fn hasDependencies(&self) -> bool {
        todo!()
    }

    pub const INVALID_OFFSET_VALUE: f64 = f64::MAX;
    /**
     * @brief Calculates variable text offset.
     *
     * @param anchor text anchor
     * @param textOffset Either `text-offset` or [ `text-radial-offset`,
     * INVALID_OFFSET_VALUE ]
     * @return std::array<f64, 2> offset along x- and y- axis correspondingly.
     */
    pub fn evaluateVariableOffset(anchor: SymbolAnchorType, textOffset: [f64; 2]) -> [f64; 2] {
        todo!()
    }

    pub fn calculateTileDistances(line: &GeometryCoordinates, anchor: &Anchor) -> Vec<f64> {
        todo!()
    }
}

impl SymbolLayout {
    fn addFeature(
        layoutFeatureIndex: usize,
        feature: &SymbolFeature,
        shapedTextOrientations: &ShapedTextOrientations,
        shapedIcon: Option<PositionedIcon>,
        imageMap: &ImageMap,
        textOffset: [f64; 2],
        layoutTextSize: f64,
        layoutIconSize: f64,
        iconType: SymbolContent,
    ) {
        todo!()
    }

    fn anchorIsTooClose(&self, text: &U16String, repeatDistance: f64, anchor: &Anchor) -> bool {
        todo!()
    }

    fn addToDebugBuffers(&self, bucket: &SymbolBucket) {
        todo!()
    }

    // Adds placed items to the buffer.
    fn addSymbol(
        &self,
        buffer: &SymbolBucket_Buffer,
        sizeData: Range<f64>,
        symbol: &SymbolQuad,
        labelAnchor: &Anchor,
        placedSymbol: &PlacedSymbol,
        sortKey: f64,
    ) -> usize {
        todo!()
    }
    fn addSymbols(
        &self,
        buffer: &SymbolBucket_Buffer,
        sizeData: Range<f64>,
        symbols: &SymbolQuads,
        labelAnchor: &Anchor,
        placedSymbol: &PlacedSymbol,
        sortKey: f64,
    ) -> usize {
        todo!()
    }

    // Adds symbol quads to bucket and returns formatted section index of last
    // added quad.
    fn addSymbolGlyphQuads(
        &self,
        buffer: &SymbolBucket_Buffer,
        symbolInstance: &SymbolInstance,
        feature: &SymbolFeature,
        writingMode: WritingModeType,
        placedIndex: Option<usize>,
        glyphQuads: &SymbolQuads,
        canonical: &CanonicalTileID,
        lastAddedSection: Option<usize>,
    ) -> usize {
        todo!()
    }

    fn updatePaintPropertiesForSection(
        &self,
        buffer: &SymbolBucket_Buffer,
        feature: &SymbolFeature,
        sectionIndex: usize,
        canonical: &CanonicalTileID,
    ) -> usize {
        todo!()
    }
}
