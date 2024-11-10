//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/renderer/buckets/symbol_bucket.cpp

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    marker::PhantomData,
    ops::Range,
};

use geo_types::GeometryCollection;

use crate::{
    euclid::Point2D,
    legacy::{
        geometry_tile_data::GeometryCoordinates,
        glyph::WritingModeType,
        image_atlas::ImagePositions,
        layout::{
            symbol_feature::SymbolGeometryTileFeature,
            symbol_instance::SymbolInstance,
            symbol_layout::{LayerProperties, SortKeyRange},
        },
        style_types::{
            PropertyValue, SymbolLayoutProperties_PossiblyEvaluated, TextWritingModeType,
        },
        CanonicalTileID, TileSpace,
    },
    render::view_state::ViewState,
};

/// maplibre/maplibre-native#4add9ea original name: PatternDependency
struct PatternDependency;

/// maplibre/maplibre-native#4add9ea original name: SymbolVertex
#[derive(Clone, Debug)]
pub struct SymbolVertex {
    pub labelAnchor: Point2D<f64, TileSpace>,
    pub o: Point2D<f64, TileSpace>,
    pub glyphOffsetY: f64,
    pub tx: u16,
    pub ty: u16,
    pub sizeData: Range<f64>,
    pub isSDF: bool,
    pub pixelOffset: Point2D<f64, TileSpace>,
    pub minFontScale: Point2D<f64, TileSpace>,
}

impl SymbolVertex {
    /// maplibre/maplibre-native#4add9ea original name: new
    pub fn new(
        labelAnchor: Point2D<f64, TileSpace>,
        o: Point2D<f64, TileSpace>,
        glyphOffsetY: f64,
        tx: u16,
        ty: u16,
        sizeData: Range<f64>,
        isSDF: bool,
        pixelOffset: Point2D<f64, TileSpace>,
        minFontScale: Point2D<f64, TileSpace>,
    ) -> SymbolVertex {
        Self {
            labelAnchor,
            o,
            glyphOffsetY,
            tx,
            ty,
            sizeData,
            isSDF,
            pixelOffset,
            minFontScale,
        }
    }
}

/// maplibre/maplibre-native#4add9ea original name: DynamicVertex
#[derive(Copy, Clone, Debug)]
pub struct DynamicVertex {
    anchorPoint: Point2D<f64, TileSpace>,
    labelAngle: f64,
}
/// maplibre/maplibre-native#4add9ea original name: OpacityVertex
#[derive(Copy, Clone, Debug)]
pub struct OpacityVertex {
    placed: bool,
    opacity: f64,
}

impl DynamicVertex {
    /// maplibre/maplibre-native#4add9ea original name: new
    pub fn new(anchorPoint: Point2D<f64, TileSpace>, labelAngle: f64) -> Self {
        Self {
            anchorPoint,
            labelAngle,
        }
    }
}

impl OpacityVertex {
    /// maplibre/maplibre-native#4add9ea original name: new
    pub fn new(placed: bool, opacity: f64) -> Self {
        Self { placed, opacity }
    }
}

/// maplibre/maplibre-native#4add9ea original name: VertexVector
pub type VertexVector = Vec<SymbolVertex>;
/// maplibre/maplibre-native#4add9ea original name: DynamicVertexVector
pub type DynamicVertexVector = Vec<DynamicVertex>;
/// maplibre/maplibre-native#4add9ea original name: OpacityVertexVector
pub type OpacityVertexVector = Vec<OpacityVertex>;
/// maplibre/maplibre-native#4add9ea original name: SymbolTextAttributes
#[derive(Default, Clone, Debug)]
pub struct SymbolTextAttributes;
/// maplibre/maplibre-native#4add9ea original name: SymbolSizeBinder
#[derive(Default, Clone, Debug)]
pub struct SymbolSizeBinder;

impl SymbolSizeBinder {
    /// maplibre/maplibre-native#4add9ea original name: getVertexSizeData
    pub fn getVertexSizeData(&self, feature: &SymbolGeometryTileFeature) -> Range<f64> {
        // TODO ConstantSymbolSizeBinder
        return 0.0..0.0;
    }
}

/// maplibre/maplibre-native#4add9ea original name: FeatureSortOrder
#[derive(Default, Clone, Debug)]
struct FeatureSortOrder;
/// maplibre/maplibre-native#4add9ea original name: TriangleIndexVector
#[derive(Default, Clone, Debug)]
pub struct TriangleIndexVector {
    pub indices: Vec<u16>,
}
impl TriangleIndexVector {
    /// maplibre/maplibre-native#4add9ea original name: push
    pub fn push(&mut self, a: u16, b: u16, c: u16) {
        //todo!()
        // put them flat into the buffer .len() should return the count of indices
        self.indices.push(a);
        self.indices.push(b);
        self.indices.push(c);
    }

    /// maplibre/maplibre-native#4add9ea original name: len
    pub fn len(&self) -> usize {
        //  todo!()
        // put them flat into the buffer .len() should return the count of indices
        self.indices.len()
    }
}
/// maplibre/maplibre-native#4add9ea original name: UploadPass
struct UploadPass;
/// maplibre/maplibre-native#4add9ea original name: SymbolInstanceReferences
struct SymbolInstanceReferences;
/// maplibre/maplibre-native#4add9ea original name: RenderLayer
struct RenderLayer;
/// maplibre/maplibre-native#4add9ea original name: CrossTileSymbolLayerIndex
struct CrossTileSymbolLayerIndex;
/// maplibre/maplibre-native#4add9ea original name: BucketPlacementData
struct BucketPlacementData;
/// maplibre/maplibre-native#4add9ea original name: Placement
struct Placement;
/// maplibre/maplibre-native#4add9ea original name: TransformState
type TransformState = ViewState;
/// maplibre/maplibre-native#4add9ea original name: RenderTile
struct RenderTile;

/// maplibre/maplibre-native#4add9ea original name: Segment
#[derive(Copy, Clone, Debug)]
pub struct Segment<T> {
    pub vertexOffset: usize,
    pub indexOffset: usize,
    pub vertexLength: usize,
    pub indexLength: usize,

    // One DrawScope per layer ID. This minimizes rebinding in cases where
    // several layers share buckets but have different sets of active
    // attributes. This can happen:
    //   * when two layers have the same layout properties, but differing
    //     data-driven paint properties
    //   * when two fill layers have the same layout properties, but one
    //     uses fill-color and the other uses fill-pattern
    // TODO drawScopes:  BTreeMap<String, gfx::DrawScope>
    pub sortKey: f64,

    pub _phandom_data: PhantomData<T>,
}

/// maplibre/maplibre-native#4add9ea original name: PlacedSymbol
#[derive(Default, Clone, Debug)]
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

/// maplibre/maplibre-native#4add9ea original name: PatternLayerMap
type PatternLayerMap = HashMap<String, PatternDependency>;

/// maplibre/maplibre-native#4add9ea original name: SegmentVector
type SegmentVector<T> = Vec<Segment<T>>;

/// maplibre/maplibre-native#4add9ea original name: SymbolBucketBuffer
#[derive(Default, Clone, Debug)]
pub struct SymbolBucketBuffer {
    pub sharedVertices: VertexVector,
    pub sharedDynamicVertices: DynamicVertexVector,
    pub sharedOpacityVertices: OpacityVertexVector,

    /// maplibre/maplibre-native#4add9ea original name: TriangleIndexVector
    // type TriangleIndexVector = gfx::IndexVector<gfx::Triangles>,
    pub sharedTriangles: TriangleIndexVector,
    pub triangles: TriangleIndexVector,
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

/// maplibre/maplibre-native#4add9ea original name: PaintProperties
#[derive(Clone, Debug)]
pub struct PaintProperties {
    //    iconBinders: SymbolIconProgram::Binders,
    //    textBinders:  SymbolSDFTextProgram::Binders,
}

/// maplibre/maplibre-native#4add9ea original name: CollisionBuffer
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

/// maplibre/maplibre-native#4add9ea original name: CollisionBoxBuffer
//struct CollisionBoxBuffer : public CollisionBuffer {
/// maplibre/maplibre-native#4add9ea original name: LineIndexVector
//    type LineIndexVector = gfx::IndexVector<gfx::Lines>,
//    Box<LineIndexVector> sharedLines = std::make_shared::<LineIndexVector>(),
//    LineIndexVector& lines = *sharedLines,
//    #if MLN_LEGACY_RENDERER
//    std::optional<gfx::IndexBuffer> indexBuffer,
//    #endif // MLN_LEGACY_RENDERER
//}
/// maplibre/maplibre-native#4add9ea original name: CollisionCircleBuffer
//struct CollisionCircleBuffer : public CollisionBuffer {
/// maplibre/maplibre-native#4add9ea original name: TriangleIndexVector
//    //type TriangleIndexVector = gfx::IndexVector<gfx::Triangles>,
//    Box<TriangleIndexVector> sharedTriangles = std::make_shared::<TriangleIndexVector>(),
//    TriangleIndexVector& triangles = *sharedTriangles,
//    #if MLN_LEGACY_RENDERER
//            std::optional<gfx::IndexBuffer> indexBuffer,
//    #endif // MLN_LEGACY_RENDERER
//}

/// maplibre/maplibre-native#4add9ea original name: SymbolBucket
#[derive(Clone, Debug)]
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

    //pub symbolInstances: Vec<SymbolInstance>,
    pub sortKeyRanges: Vec<SortKeyRange>,

    pub paintProperties: HashMap<String, PaintProperties>,

    pub textSizeBinder: Box<SymbolSizeBinder>,
    /// maplibre/maplibre-native#4add9ea original name: VertexVector
    //type VertexVector = gfx::VertexVector<SymbolLayoutVertex>,
    /// maplibre/maplibre-native#4add9ea original name: VertexBuffer
    //type VertexBuffer = gfx::VertexBuffer<SymbolLayoutVertex>,
    /// maplibre/maplibre-native#4add9ea original name: DynamicVertexVector
    //type DynamicVertexVector = gfx::VertexVector<gfx::Vertex<SymbolDynamicLayoutAttributes>>,
    /// maplibre/maplibre-native#4add9ea original name: DynamicVertexBuffer
    //type DynamicVertexBuffer = gfx::VertexBuffer<gfx::Vertex<SymbolDynamicLayoutAttributes>>,
    /// maplibre/maplibre-native#4add9ea original name: OpacityVertexVector
    //type OpacityVertexVector = gfx::VertexVector<gfx::Vertex<SymbolOpacityAttributes>>,
    /// maplibre/maplibre-native#4add9ea original name: OpacityVertexBuffer
    //type OpacityVertexBuffer = gfx::VertexBuffer<gfx::Vertex<SymbolOpacityAttributes>>,
    pub text: SymbolBucketBuffer,

    pub iconSizeBinder: Box<SymbolSizeBinder>,
    pub icon: SymbolBucketBuffer,
    pub sdfIcon: SymbolBucketBuffer,

    /// maplibre/maplibre-native#4add9ea original name: CollisionVertexVector
    //type CollisionVertexVector = gfx::VertexVector<gfx::Vertex<CollisionBoxLayoutAttributes>>,
    /// maplibre/maplibre-native#4add9ea original name: CollisionDynamicVertexVector
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
    /// maplibre/maplibre-native#4add9ea original name: new
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
            // TODO symbolInstances: symbolInstances_,
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
    /// maplibre/maplibre-native#4add9ea original name: upload
    pub fn upload(&self, pass: &UploadPass) {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: hasData
    pub fn hasData(&self) -> bool {
        // todo!()
        true
    }

    /// maplibre/maplibre-native#4add9ea original name: hasTextData
    pub fn hasTextData(&self) -> bool {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: hasIconData
    pub fn hasIconData(&self) -> bool {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: hasSdfIconData
    pub fn hasSdfIconData(&self) -> bool {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: hasIconCollisionBoxData
    pub fn hasIconCollisionBoxData(&self) -> bool {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: hasIconCollisionCircleData
    pub fn hasIconCollisionCircleData(&self) -> bool {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: hasTextCollisionBoxData
    pub fn hasTextCollisionBoxData(&self) -> bool {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: hasTextCollisionCircleData
    pub fn hasTextCollisionCircleData(&self) -> bool {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: hasFormatSectionOverrides
    pub fn hasFormatSectionOverrides(&self) -> bool {
        //  todo!()
        false
    }

    /// maplibre/maplibre-native#4add9ea original name: sortFeatures
    pub fn sortFeatures(&self, angle: f64) {
        todo!()
    }
    // Returns references to the `symbolInstances` items, sorted by viewport Y.
    /// maplibre/maplibre-native#4add9ea original name: getSortedSymbols
    pub fn getSortedSymbols(&self, angle: f64) -> SymbolInstanceReferences {
        todo!()
    }
    // Returns references to the `symbolInstances` items, which belong to the
    // `sortKeyRange` range, returns references to all the symbols if
    // |sortKeyRange| is `std::nullopt`.
    /// maplibre/maplibre-native#4add9ea original name: getSymbols
    pub fn getSymbols(&self, sortKeyRange: &Option<SortKeyRange>) -> SymbolInstanceReferences {
        todo!()
    }

    /// maplibre/maplibre-native#4add9ea original name: getOrCreateIconCollisionBox
    //    pub fn getOrCreateIconCollisionBox(&self,) -> &CollisionBoxBuffer{
    //        if (!self.iconCollisionBox) {
    //            self.iconCollisionBox = std::make_unique<CollisionBoxBuffer>()
    //        }
    //        return *self.iconCollisionBox
    //    }
    /// maplibre/maplibre-native#4add9ea original name: getOrCreateTextCollisionBox
    //    pub fn getOrCreateTextCollisionBox(&self,) -> &CollisionBoxBuffer{
    //        if (!self.textCollisionBox) {
    //            self.textCollisionBox = std::make_unique<CollisionBoxBuffer>()
    //        }
    //        return *self.textCollisionBox
    //    }
    /// maplibre/maplibre-native#4add9ea original name:    getOrCreateIconCollisionCircleBuffer
    //    pub fn    getOrCreateIconCollisionCircleBuffer(&self,)  -> &CollisionCircleBuffer{
    //        if (!self.iconCollisionCircle) {
    //            self.iconCollisionCircle = std::make_unique<CollisionCircleBuffer>()
    //        }
    //        return *self.iconCollisionCircle
    //    }
    /// maplibre/maplibre-native#4add9ea original name:  getOrCreateTextCollisionCircleBuffer
    //    pub fn  getOrCreateTextCollisionCircleBuffer(&self,) -> &CollisionCircleBuffer {
    //        if (!self.textCollisionCircle) {
    //            self.textCollisionCircle = std::make_unique<CollisionCircleBuffer>(),
    //        }
    //        return *self.textCollisionCircle
    //    }

    /// maplibre/maplibre-native#4add9ea original name: getQueryRadius
    pub fn getQueryRadius(&self, layer: RenderLayer) -> f64 {
        return 0.;
    }

    /// maplibre/maplibre-native#4add9ea original name: needsUpload
    pub fn needsUpload(&self) -> bool {
        return self.hasData() && !self.uploaded;
    }

    // Feature geometries are also used to populate the feature index.
    // Obtaining these is a costly operation, so we do it only once, and
    // pass-by--ref the geometries as a second parameter.
    /// maplibre/maplibre-native#4add9ea original name: addFeature
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
    /// maplibre/maplibre-native#4add9ea original name: registerAtCrossTileIndex
    pub fn registerAtCrossTileIndex(
        &self,
        cross_tile_index: &CrossTileSymbolLayerIndex,
        render_tile: &RenderTile,
    ) -> (u32, bool) {
        todo!()
    }
    // Places this bucket to the given placement.
    /// maplibre/maplibre-native#4add9ea original name: place
    pub fn place(
        &self,
        placement: &Placement,
        bucket_placement_data: &BucketPlacementData,
        values: &BTreeSet<u32>,
    ) {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: updateVertices
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
