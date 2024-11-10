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
    pub label_anchor: Point2D<f64, TileSpace>,
    pub o: Point2D<f64, TileSpace>,
    pub glyph_offset_y: f64,
    pub tx: u16,
    pub ty: u16,
    pub size_data: Range<f64>,
    pub is_sdf: bool,
    pub pixel_offset: Point2D<f64, TileSpace>,
    pub min_font_scale: Point2D<f64, TileSpace>,
}

impl SymbolVertex {
    /// maplibre/maplibre-native#4add9ea original name: new
    pub fn new(
        label_anchor: Point2D<f64, TileSpace>,
        o: Point2D<f64, TileSpace>,
        glyph_offset_y: f64,
        tx: u16,
        ty: u16,
        size_data: Range<f64>,
        is_sdf: bool,
        pixel_offset: Point2D<f64, TileSpace>,
        min_font_scale: Point2D<f64, TileSpace>,
    ) -> SymbolVertex {
        Self {
            label_anchor,
            o,
            glyph_offset_y,
            tx,
            ty,
            size_data,
            is_sdf,
            pixel_offset,
            min_font_scale,
        }
    }
}

/// maplibre/maplibre-native#4add9ea original name: DynamicVertex
#[derive(Copy, Clone, Debug)]
pub struct DynamicVertex {
    anchor_point: Point2D<f64, TileSpace>,
    label_angle: f64,
}
/// maplibre/maplibre-native#4add9ea original name: OpacityVertex
#[derive(Copy, Clone, Debug)]
pub struct OpacityVertex {
    placed: bool,
    opacity: f64,
}

impl DynamicVertex {
    /// maplibre/maplibre-native#4add9ea original name: new
    pub fn new(anchor_point: Point2D<f64, TileSpace>, label_angle: f64) -> Self {
        Self {
            anchor_point,
            label_angle,
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
    pub fn get_vertex_size_data(&self, feature: &SymbolGeometryTileFeature) -> Range<f64> {
        // TODO ConstantSymbolSizeBinder
        0.0..0.0
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
    pub vertex_offset: usize,
    pub index_offset: usize,
    pub vertex_length: usize,
    pub index_length: usize,

    // One DrawScope per layer ID. This minimizes rebinding in cases where
    // several layers share buckets but have different sets of active
    // attributes. This can happen:
    //   * when two layers have the same layout properties, but differing
    //     data-driven paint properties
    //   * when two fill layers have the same layout properties, but one
    //     uses fill-color and the other uses fill-pattern
    // TODO drawScopes:  BTreeMap<String, gfx::DrawScope>
    pub sort_key: f64,

    pub _phandom_data: PhantomData<T>,
}

/// maplibre/maplibre-native#4add9ea original name: PlacedSymbol
#[derive(Default, Clone, Debug)]
pub struct PlacedSymbol {
    pub anchor_point: Point2D<f64, TileSpace>,
    pub segment: usize,
    pub lower_size: f64,
    pub upper_size: f64,
    pub line_offset: [f64; 2],
    pub writing_modes: WritingModeType,
    pub line: GeometryCoordinates,
    pub tile_distances: Vec<f64>,
    pub glyph_offsets: Vec<f64>,
    pub hidden: bool,
    pub vertex_start_index: usize,
    /// The crossTileID is only filled/used on the foreground for variable text anchors
    pub cross_tile_id: u32,
    /// The placedOrientation is only used when symbol layer's property is set to
    /// support placement for orientation variants.
    pub placed_orientation: Option<TextWritingModeType>,
    pub angle: f64,
    /// Reference to placed icon, only applicable for text symbols.
    pub placed_icon_index: Option<usize>,
}

/// maplibre/maplibre-native#4add9ea original name: PatternLayerMap
type PatternLayerMap = HashMap<String, PatternDependency>;

/// maplibre/maplibre-native#4add9ea original name: SegmentVector
type SegmentVector<T> = Vec<Segment<T>>;

/// maplibre/maplibre-native#4add9ea original name: SymbolBucketBuffer
#[derive(Default, Clone, Debug)]
pub struct SymbolBucketBuffer {
    pub shared_vertices: VertexVector,
    pub shared_dynamic_vertices: DynamicVertexVector,
    pub shared_opacity_vertices: OpacityVertexVector,

    /// maplibre/maplibre-native#4add9ea original name: TriangleIndexVector
    // type TriangleIndexVector = gfx::IndexVector<gfx::Triangles>,
    pub shared_triangles: TriangleIndexVector,
    pub triangles: TriangleIndexVector,
    //TODO triangles: &TriangleIndexVector = *sharedTriangles,
    pub segments: SegmentVector<SymbolTextAttributes>,
    pub placed_symbols: Vec<PlacedSymbol>,
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
    bucket_leader_id: String,
    sorted_angle: f64,

    // Flags
    // TODO what are the initial values?
    icons_need_linear: bool,
    sort_features_by_y: bool,
    static_uploaded: bool,
    placement_changes_uploaded: bool,
    dynamic_uploaded: bool,
    sort_uploaded: bool,
    icons_in_text: bool,
    // Set and used by placement.
    pub just_reloaded: bool,
    has_variable_placement: bool,
    has_uninitialized_symbols: bool,

    //pub symbolInstances: Vec<SymbolInstance>,
    pub sort_key_ranges: Vec<SortKeyRange>,

    pub paint_properties: HashMap<String, PaintProperties>,

    pub text_size_binder: Box<SymbolSizeBinder>,
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

    pub icon_size_binder: Box<SymbolSizeBinder>,
    pub icon: SymbolBucketBuffer,
    pub sdf_icon: SymbolBucketBuffer,

    /// maplibre/maplibre-native#4add9ea original name: CollisionVertexVector
    //type CollisionVertexVector = gfx::VertexVector<gfx::Vertex<CollisionBoxLayoutAttributes>>,
    /// maplibre/maplibre-native#4add9ea original name: CollisionDynamicVertexVector
    //type CollisionDynamicVertexVector = gfx::VertexVector<gfx::Vertex<CollisionBoxDynamicAttributes>>,

    // iconCollisionBox: Box<CollisionBoxBuffer>,
    // textCollisionBox:   Box<CollisionBoxBuffer>,
    // iconCollisionCircle:  Box<CollisionCircleBuffer>,
    // textCollisionCircle: Box<CollisionCircleBuffer> ,
    tile_pixel_ratio: f64,
    bucket_instance_id: u32,
    allow_vertical_placement: bool,
    placement_modes: Vec<TextWritingModeType>,
    has_format_section_overrides: Option<bool>,

    feature_sort_order: FeatureSortOrder,

    uploaded: bool,
}

static MAX_BUCKET_INSTANCE_ID: u32 = 0;

impl SymbolBucket {
    /// maplibre/maplibre-native#4add9ea original name: new
    pub fn new(
        layout_: SymbolLayoutProperties_PossiblyEvaluated,
        paint_properties: &BTreeMap<String, LayerProperties>,
        text_size: &PropertyValue<f64>,
        icon_size: &PropertyValue<f64>,
        zoom: f64,
        icons_need_linear: bool,
        sort_features_by_y: bool,
        bucket_name: String,
        symbol_instances: Vec<SymbolInstance>,
        sort_key_ranges: Vec<SortKeyRange>,
        tile_pixel_ratio: f64,
        allow_vertical_placement: bool,
        placement_modes: Vec<TextWritingModeType>,
        icons_in_text: bool,
    ) -> Self {
        // TODO MAX_BUCKET_INSTANCE_ID += 1;

        // TODO
        // for pair in paintProperties_ {
        //      let evaluated = getEvaluated::<SymbolLayerProperties>(pair.second);
        //     self_.paintProperties.emplace(
        //         std::piecewise_ruct,
        //         std::forward_as_tuple(pair.first),
        //         std::forward_as_tuple(PaintProperties{{RenderSymbolLayer::iconPaintProperties(evaluated), zoom},
        //                                               {RenderSymbolLayer::textPaintProperties(evaluated), zoom}}));
        // }
        Self {
            layout: layout_,
            bucket_leader_id: bucket_name,
            sorted_angle: f64::MAX,
            icons_need_linear: icons_need_linear
                || icon_size.is_data_driven()
                || !icon_size.is_zoomant(),
            sort_features_by_y,
            static_uploaded: false,
            placement_changes_uploaded: false,
            dynamic_uploaded: false,
            sort_uploaded: false,
            icons_in_text: false,
            just_reloaded: false,
            has_variable_placement: false,
            has_uninitialized_symbols: false,
            // TODO symbolInstances: symbolInstances_,
            sort_key_ranges,
            paint_properties: Default::default(),
            text_size_binder: Default::default(),
            // TODO textSizeBinder: SymbolSizeBinder::create(zoom, textSize, TextSize::defaultValue()),
            text: SymbolBucketBuffer::default(),
            icon_size_binder: Default::default(),
            // TODO iconSizeBinder: SymbolSizeBinder::create(zoom, iconSize, IconSize::defaultValue()),
            icon: SymbolBucketBuffer::default(),
            sdf_icon: SymbolBucketBuffer::default(),
            tile_pixel_ratio,
            bucket_instance_id: MAX_BUCKET_INSTANCE_ID,
            allow_vertical_placement,
            placement_modes,
            has_format_section_overrides: None,
            feature_sort_order: FeatureSortOrder,
            uploaded: false,
        }
    }

    // As long as this bucket has a Prepare render pass, this function is
    // getting called. Typically, this only happens once when the bucket is
    // being rendered for the first time.
    /// maplibre/maplibre-native#4add9ea original name: upload
    pub fn upload(&self, pass: &UploadPass) {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: hasData
    pub fn has_data(&self) -> bool {
        // todo!()
        true
    }

    /// maplibre/maplibre-native#4add9ea original name: hasTextData
    pub fn has_text_data(&self) -> bool {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: has_icon_data
    pub fn has_icon_data(&self) -> bool {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: hasSdfIconData
    pub fn has_sdf_icon_data(&self) -> bool {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: hasIconCollisionBoxData
    pub fn has_icon_collision_box_data(&self) -> bool {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: hasIconCollisionCircleData
    pub fn has_icon_collision_circle_data(&self) -> bool {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: hasTextCollisionBoxData
    pub fn has_text_collision_box_data(&self) -> bool {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: hasTextCollisionCircleData
    pub fn has_text_collision_circle_data(&self) -> bool {
        todo!()
    }
    /// maplibre/maplibre-native#4add9ea original name: hasFormatSectionOverrides
    pub fn has_format_section_overrides(&self) -> bool {
        //  todo!()
        false
    }

    /// maplibre/maplibre-native#4add9ea original name: sortFeatures
    pub fn sort_features(&self, angle: f64) {
        todo!()
    }
    // Returns references to the `symbolInstances` items, sorted by viewport Y.
    /// maplibre/maplibre-native#4add9ea original name: getSortedSymbols
    pub fn get_sorted_symbols(&self, angle: f64) -> SymbolInstanceReferences {
        todo!()
    }
    // Returns references to the `symbolInstances` items, which belong to the
    // `sortKeyRange` range, returns references to all the symbols if
    // |sortKeyRange| is `std::nullopt`.
    /// maplibre/maplibre-native#4add9ea original name: getSymbols
    pub fn get_symbols(&self, sort_key_range: &Option<SortKeyRange>) -> SymbolInstanceReferences {
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
    pub fn get_query_radius(&self, layer: RenderLayer) -> f64 {
        0.
    }

    /// maplibre/maplibre-native#4add9ea original name: needsUpload
    pub fn needs_upload(&self) -> bool {
        self.has_data() && !self.uploaded
    }

    // Feature geometries are also used to populate the feature index.
    // Obtaining these is a costly operation, so we do it only once, and
    // pass-by--ref the geometries as a second parameter.
    /// maplibre/maplibre-native#4add9ea original name: addFeature
    pub fn add_feature(
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
    pub fn register_at_cross_tile_index(
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
    pub fn update_vertices(
        placement: &Placement,
        update_opacities: bool,
        transform_state: &TransformState,
        render_tile: &RenderTile,
        values: &BTreeSet<u32>,
    ) {
        todo!()
    }
}
