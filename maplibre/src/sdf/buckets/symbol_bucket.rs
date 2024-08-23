use crate::euclid::Point2D;
use crate::render::view_state::ViewState;
use crate::sdf::geometry_tile_data::GeometryCoordinates;
use crate::sdf::glyph::WritingModeType;
use crate::sdf::image_atlas::ImagePositions;
use crate::sdf::layout::symbol_feature::SymbolGeometryTileFeature;
use crate::sdf::layout::symbol_instance::SymbolInstance;
use crate::sdf::layout::symbol_layout::{LayerProperties, SortKeyRange};
use crate::sdf::style_types::{
    PropertyValue, SymbolLayoutProperties_PossiblyEvaluated, TextWritingModeType,
};
use crate::sdf::{CanonicalTileID, TileSpace};
use geo_types::GeometryCollection;
use std::collections::{BTreeMap, BTreeSet, HashMap};

struct PatternDependency;

struct FillLayoutVertex;
type VertexVector = Vec<FillLayoutVertex>;
type DynamicVertexVector = Vec<FillLayoutVertex>;
type OpacityVertexVector = Vec<FillLayoutVertex>;
#[derive(Default)]
struct SymbolTextAttributes;
#[derive(Default)]
struct SymbolSizeBinder;
#[derive(Default)]
struct FeatureSortOrder;
#[derive(Default)]
struct TriangleIndexVector;
struct UploadPass;
struct SymbolInstanceReferences;
struct RenderLayer;
struct CrossTileSymbolLayerIndex;
struct BucketPlacementData;
struct Placement;
type TransformState = ViewState;
struct RenderTile;
struct Segment<T>(T);

pub struct PlacedSymbol {
    pub anchorPoint: Point2D<f64, TileSpace>,
    pub segment: usize,
    pub lowerSize: f64,
    pub upperSize: f64,
    pub lineOffset: [f64; 2],
    pub writingModes: WritingModeType,
    pub line: GeometryCoordinates,
    pub tileDistances: Vec<f64>,
    pub glyphOffsets: Vec<f64>,
    pub hidden: bool,
    pub vertexStartIndex: usize,
    /// The crossTileID is only filled/used on the foreground for variable text anchors
    pub crossTileID: u32,
    /// The placedOrientation is only used when symbol layer's property is set to
    /// support placement for orientation variants.
    pub placedOrientation: Option<TextWritingModeType>,
    pub angle: f64,
    /// Reference to placed icon, only applicable for text symbols.
    pub placedIconIndex: Option<usize>,
}

type PatternLayerMap = HashMap<String, PatternDependency>;

type SegmentVector<T> = Vec<Segment<T>>;

#[derive(Default)]
pub struct SymbolBucketBuffer {
    pub sharedVertices: Box<VertexVector>,
    pub sharedDynamicVertices: Box<DynamicVertexVector>,
    pub sharedOpacityVertices: Box<OpacityVertexVector>,

    // type TriangleIndexVector = gfx::IndexVector<gfx::Triangles>,
    pub sharedTriangles: Box<TriangleIndexVector>,
    pub triangles: Box<TriangleIndexVector>,
    //TODO triangles: &TriangleIndexVector = *sharedTriangles,
    pub segments: SegmentVector<SymbolTextAttributes>,
    pub placedSymbols: Vec<PlacedSymbol>,
    //    #if MLN_LEGACY_RENDERER
    //            std::optional<VertexBuffer> vertexBuffer,
    //            std::optional<DynamicVertexBuffer> dynamicVertexBuffer,
    //            std::optional<OpacityVertexBuffer> opacityVertexBuffer,
    //            std::optional<gfx::IndexBuffer> indexBuffer,
    //    #endif // MLN_LEGACY_RENDERER
}

struct PaintProperties {
    //    iconBinders: SymbolIconProgram::Binders,
    //    textBinders:  SymbolSDFTextProgram::Binders,
}

//struct CollisionBuffer {
//    Box<CollisionVertexVector> sharedVertices = std::make_shared::<CollisionVertexVector>(),
//    CollisionVertexVector& vertices() { return *sharedVertices, }
//    CollisionVertexVector& vertices()  { return *sharedVertices, }
//
//    Box<CollisionDynamicVertexVector> sharedDynamicVertices = std::make_shared::<CollisionDynamicVertexVector>(),
//    CollisionDynamicVertexVector& dynamicVertices() { return *sharedDynamicVertices, }
//    CollisionDynamicVertexVector& dynamicVertices()  { return *sharedDynamicVertices, }
//
//    SegmentVector<CollisionBoxProgram::AttributeList> segments,
//
//    #if MLN_LEGACY_RENDERER
//    std::optional<gfx::VertexBuffer<gfx::Vertex<CollisionBoxLayoutAttributes>>> vertexBuffer,
//    std::optional<gfx::VertexBuffer<gfx::Vertex<CollisionBoxDynamicAttributes>>> dynamicVertexBuffer,
//    #endif // MLN_LEGACY_RENDERER
//}

//struct CollisionBoxBuffer : public CollisionBuffer {
//    type LineIndexVector = gfx::IndexVector<gfx::Lines>,
//    Box<LineIndexVector> sharedLines = std::make_shared::<LineIndexVector>(),
//    LineIndexVector& lines = *sharedLines,
//    #if MLN_LEGACY_RENDERER
//    std::optional<gfx::IndexBuffer> indexBuffer,
//    #endif // MLN_LEGACY_RENDERER
//}
//struct CollisionCircleBuffer : public CollisionBuffer {
//    //type TriangleIndexVector = gfx::IndexVector<gfx::Triangles>,
//    Box<TriangleIndexVector> sharedTriangles = std::make_shared::<TriangleIndexVector>(),
//    TriangleIndexVector& triangles = *sharedTriangles,
//    #if MLN_LEGACY_RENDERER
//            std::optional<gfx::IndexBuffer> indexBuffer,
//    #endif // MLN_LEGACY_RENDERER
//}

pub struct SymbolBucket {
    layout: SymbolLayoutProperties_PossiblyEvaluated,
    bucketLeaderID: String,
    sortedAngle: f64,

    // Flags
    // TODO what are the initial values?
    iconsNeedLinear: bool,
    sortFeaturesByY: bool,
    staticUploaded: bool,
    placementChangesUploaded: bool,
    dynamicUploaded: bool,
    sortUploaded: bool,
    iconsInText: bool,
    // Set and used by placement.
    pub justReloaded: bool,
    hasVariablePlacement: bool,
    hasUninitializedSymbols: bool,

    pub symbolInstances: Vec<SymbolInstance>,
    pub sortKeyRanges: Vec<SortKeyRange>,

    pub paintProperties: HashMap<String, PaintProperties>,

    pub textSizeBinder: Box<SymbolSizeBinder>,

    //type VertexVector = gfx::VertexVector<SymbolLayoutVertex>,
    //type VertexBuffer = gfx::VertexBuffer<SymbolLayoutVertex>,
    //type DynamicVertexVector = gfx::VertexVector<gfx::Vertex<SymbolDynamicLayoutAttributes>>,
    //type DynamicVertexBuffer = gfx::VertexBuffer<gfx::Vertex<SymbolDynamicLayoutAttributes>>,
    //type OpacityVertexVector = gfx::VertexVector<gfx::Vertex<SymbolOpacityAttributes>>,
    //type OpacityVertexBuffer = gfx::VertexBuffer<gfx::Vertex<SymbolOpacityAttributes>>,
    pub text: SymbolBucketBuffer,

    pub iconSizeBinder: Box<SymbolSizeBinder>,

    pub icon: SymbolBucketBuffer,
    pub sdfIcon: SymbolBucketBuffer,

    //type CollisionVertexVector = gfx::VertexVector<gfx::Vertex<CollisionBoxLayoutAttributes>>,
    //type CollisionDynamicVertexVector = gfx::VertexVector<gfx::Vertex<CollisionBoxDynamicAttributes>>,

    // iconCollisionBox: Box<CollisionBoxBuffer>,
    // textCollisionBox:   Box<CollisionBoxBuffer>,
    // iconCollisionCircle:  Box<CollisionCircleBuffer>,
    // textCollisionCircle: Box<CollisionCircleBuffer> ,
    tilePixelRatio: f64,
    bucketInstanceId: u32,
    allowVerticalPlacement: bool,
    placementModes: Vec<TextWritingModeType>,
    hasFormatSectionOverrides_: Option<bool>,

    featureSortOrder: FeatureSortOrder,

    uploaded: bool,
}

static maxBucketInstanceId: u32 = 0;

impl SymbolBucket {
    pub fn new(
        layout_: SymbolLayoutProperties_PossiblyEvaluated,
        paintProperties_: &BTreeMap<String, LayerProperties>,
        textSize: &PropertyValue<f64>,
        iconSize: &PropertyValue<f64>,
        zoom: f64,
        iconsNeedLinear_: bool,
        sortFeaturesByY_: bool,
        bucketName_: String,
        symbolInstances_: Vec<SymbolInstance>,
        sortKeyRanges_: Vec<SortKeyRange>,
        tilePixelRatio_: f64,
        allowVerticalPlacement_: bool,
        placementModes_: Vec<TextWritingModeType>,
        iconsInText_: bool,
    ) -> Self {
        // TODO maxBucketInstanceId += 1;
        let mut self_ = Self {
            layout: layout_,
            bucketLeaderID: bucketName_,
            sortedAngle: f64::MAX,
            iconsNeedLinear: iconsNeedLinear_ || iconSize.isDataDriven() || !iconSize.isZoomant(),
            sortFeaturesByY: sortFeaturesByY_,
            staticUploaded: false,
            placementChangesUploaded: false,
            dynamicUploaded: false,
            sortUploaded: false,
            iconsInText: false,
            justReloaded: false,
            hasVariablePlacement: false,
            hasUninitializedSymbols: false,
            symbolInstances: symbolInstances_,
            sortKeyRanges: sortKeyRanges_,
            paintProperties: Default::default(),
            textSizeBinder: Default::default(),
            // TODO textSizeBinder: SymbolSizeBinder::create(zoom, textSize, TextSize::defaultValue()),
            text: SymbolBucketBuffer::default(),
            iconSizeBinder: Default::default(),
            // TODO iconSizeBinder: SymbolSizeBinder::create(zoom, iconSize, IconSize::defaultValue()),
            icon: SymbolBucketBuffer::default(),
            sdfIcon: SymbolBucketBuffer::default(),
            tilePixelRatio: tilePixelRatio_,
            bucketInstanceId: maxBucketInstanceId,
            allowVerticalPlacement: allowVerticalPlacement_,
            placementModes: placementModes_,
            hasFormatSectionOverrides_: None,
            featureSortOrder: FeatureSortOrder::default(),
            uploaded: false,
        };

        // TODO
        // for pair in paintProperties_ {
        //      let evaluated = getEvaluated::<SymbolLayerProperties>(pair.second);
        //     self_.paintProperties.emplace(
        //         std::piecewise_ruct,
        //         std::forward_as_tuple(pair.first),
        //         std::forward_as_tuple(PaintProperties{{RenderSymbolLayer::iconPaintProperties(evaluated), zoom},
        //                                               {RenderSymbolLayer::textPaintProperties(evaluated), zoom}}));
        // }
        self_
    }

    // As long as this bucket has a Prepare render pass, this function is
    // getting called. Typically, this only happens once when the bucket is
    // being rendered for the first time.
    pub fn upload(&self, pass: &UploadPass) {
        todo!()
    }
    pub fn hasData(&self) -> bool {
        todo!()
    }

    pub fn hasTextData(&self) -> bool {
        todo!()
    }
    pub fn hasIconData(&self) -> bool {
        todo!()
    }
    pub fn hasSdfIconData(&self) -> bool {
        todo!()
    }
    pub fn hasIconCollisionBoxData(&self) -> bool {
        todo!()
    }
    pub fn hasIconCollisionCircleData(&self) -> bool {
        todo!()
    }
    pub fn hasTextCollisionBoxData(&self) -> bool {
        todo!()
    }
    pub fn hasTextCollisionCircleData(&self) -> bool {
        todo!()
    }
    pub fn hasFormatSectionOverrides(&self) -> bool {
        todo!()
    }

    pub fn sortFeatures(&self, angle: f64) {
        todo!()
    }
    // Returns references to the `symbolInstances` items, sorted by viewport Y.
    pub fn getSortedSymbols(&self, angle: f64) -> SymbolInstanceReferences {
        todo!()
    }
    // Returns references to the `symbolInstances` items, which belong to the
    // `sortKeyRange` range, returns references to all the symbols if
    // |sortKeyRange| is `std::nullopt`.
    pub fn getSymbols(&self, sortKeyRange: &Option<SortKeyRange>) -> SymbolInstanceReferences {
        todo!()
    }

    //    pub fn getOrCreateIconCollisionBox(&self,) -> &CollisionBoxBuffer{
    //        if (!self.iconCollisionBox) {
    //            self.iconCollisionBox = std::make_unique<CollisionBoxBuffer>()
    //        }
    //        return *self.iconCollisionBox
    //    }
    //    pub fn getOrCreateTextCollisionBox(&self,) -> &CollisionBoxBuffer{
    //        if (!self.textCollisionBox) {
    //            self.textCollisionBox = std::make_unique<CollisionBoxBuffer>()
    //        }
    //        return *self.textCollisionBox
    //    }
    //    pub fn    getOrCreateIconCollisionCircleBuffer(&self,)  -> &CollisionCircleBuffer{
    //        if (!self.iconCollisionCircle) {
    //            self.iconCollisionCircle = std::make_unique<CollisionCircleBuffer>()
    //        }
    //        return *self.iconCollisionCircle
    //    }
    //    pub fn  getOrCreateTextCollisionCircleBuffer(&self,) -> &CollisionCircleBuffer {
    //        if (!self.textCollisionCircle) {
    //            self.textCollisionCircle = std::make_unique<CollisionCircleBuffer>(),
    //        }
    //        return *self.textCollisionCircle
    //    }

    pub fn getQueryRadius(&self, layer: RenderLayer) -> f64 {
        return 0.;
    }

    pub fn needsUpload(&self) -> bool {
        return self.hasData() && !self.uploaded;
    }

    // Feature geometries are also used to populate the feature index.
    // Obtaining these is a costly operation, so we do it only once, and
    // pass-by--ref the geometries as a second parameter.
    pub fn addFeature(
        &self,
        geometry_tile_fature: &SymbolGeometryTileFeature,
        geometry_collection: &GeometryCollection,
        image_positions: &ImagePositions,
        patter_layer_map: &PatternLayerMap,
        value: usize,
        canonical: &CanonicalTileID,
    ) {
    }

    // The following methods are implemented by buckets that require cross-tile indexing and placement.

    // Returns a pair, the first element of which is a bucket cross-tile id
    // on success call, `0` otherwise. The second element is `true` if
    // the bucket was originally registered, `false` otherwise.
    pub fn registerAtCrossTileIndex(
        &self,
        cross_tile_index: &CrossTileSymbolLayerIndex,
        render_tile: &RenderTile,
    ) -> (u32, bool) {
        todo!()
    }
    // Places this bucket to the given placement.
    pub fn place(
        &self,
        placement: &Placement,
        bucket_placement_data: &BucketPlacementData,
        values: &BTreeSet<u32>,
    ) {
        todo!()
    }
    pub fn updateVertices(
        placement: &Placement,
        updateOpacities: bool,
        transform_state: &TransformState,
        render_tile: &RenderTile,
        values: &BTreeSet<u32>,
    ) {
        todo!()
    }
}
