use std::{
    marker::PhantomData,
    ops::{Deref, Range},
    rc::Rc,
};

use crate::{
    coords::WorldTileCoords,
    environment::Environment,
    euclid::{Box2D, Point2D},
    kernel::Kernel,
    plugin::Plugin,
    render::{
        eventually::Eventually,
        graph::RenderGraph,
        shaders::{SDFShaderFeatureMetadata, ShaderLayerMetadata, ShaderSymbolVertex},
        RenderStageLabel,
    },
    schedule::Schedule,
    sdf::resource::GlyphTexture,
    tcs::{system::SystemContainer, tiles::TileComponent, world::World},
    vector::{
        resource::BufferPool,
        tessellation::{IndexDataType, OverAlignedVertexBuffer},
        VectorTransferables,
    },
};
use crate::render::shaders::ShaderSymbolVertexNew;

mod populate_world_system;
mod queue_system;
mod render_commands;
mod resource;
mod resource_system;
mod upload_system;

// Public due to bechmarks
pub mod bidi;
pub  mod buckets;
pub  mod collision_feature;
pub  mod collision_index;
pub  mod collision_system;
pub  mod font_stack;
pub mod geometry;
pub mod geometry_tile_data;
pub mod glyph;
pub mod glyph_atlas;
pub mod glyph_range;
pub mod grid_index;
pub mod image;
pub mod image_atlas;
pub mod layout;
pub mod quads;
pub mod shaping;
pub mod style_types;
pub mod tagged_string;
pub mod tessellation;
pub mod text;
mod util;
mod tessellation_new;

struct SymbolPipeline(wgpu::RenderPipeline);

impl Deref for SymbolPipeline {
    type Target = wgpu::RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type SymbolBufferPool = BufferPool<
    wgpu::Queue,
    wgpu::Buffer,
    ShaderSymbolVertexNew,
    IndexDataType,
    ShaderLayerMetadata,
    SDFShaderFeatureMetadata,
>;

pub struct SdfPlugin<T>(PhantomData<T>);

impl<T: VectorTransferables> Default for SdfPlugin<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<E: Environment, T: VectorTransferables> Plugin<E> for SdfPlugin<T> {
    fn build(
        &self,
        schedule: &mut Schedule,
        kernel: Rc<Kernel<E>>,
        world: &mut World,
        _graph: &mut RenderGraph,
    ) {
        let resources = &mut world.resources;

        resources.insert(Eventually::<SymbolPipeline>::Uninitialized);
        resources.insert(Eventually::<SymbolBufferPool>::Uninitialized);
        resources.insert(Eventually::<GlyphTexture>::Uninitialized);
        resources.insert(Eventually::<(wgpu::Texture, wgpu::Sampler)>::Uninitialized);

        schedule.add_system_to_stage(
            RenderStageLabel::Extract,
            SystemContainer::new(populate_world_system::PopulateWorldSystem::<E, T>::new(
                &kernel,
            )),
        );

        schedule.add_system_to_stage(RenderStageLabel::Prepare, resource_system::resource_system);
        schedule.add_system_to_stage(RenderStageLabel::Queue, upload_system::upload_system); // FIXME tcs: Upload updates the TileView in tileviewpattern -> upload most run before prepare
        schedule.add_system_to_stage(RenderStageLabel::Queue, queue_system::queue_system);

        schedule.add_system_to_stage(
            RenderStageLabel::PhaseSort,
            SystemContainer::new(collision_system::CollisionSystem::new()),
        );
    }
}

pub struct Feature {
    pub bbox: Box2D<f32, TileSpace>,
    pub indices: Range<usize>,
    pub text_anchor: Point2D<f32, TileSpace>,
    pub str: String,
}

pub struct SymbolLayerData {
    pub coords: WorldTileCoords,
    pub source_layer: String,
    pub buffer: OverAlignedVertexBuffer<ShaderSymbolVertex, IndexDataType>,
    pub new_buffer:OverAlignedVertexBuffer<ShaderSymbolVertexNew, IndexDataType>, // TODO
    pub features: Vec<Feature>,
}

#[derive(Default)]
pub struct SymbolLayersDataComponent {
    pub layers: Vec<SymbolLayerData>,
}

impl TileComponent for SymbolLayersDataComponent {}

// TODO where should this live?
pub struct TileSpace; // The unit in which geometries or symbols are on a tile (0-EXTENT)
pub struct ScreenSpace;

// TODO where should this live?
#[derive(Copy, Clone, PartialEq)]
pub enum MapMode {
    ///< continually updating map
    Continuous,
    ///< a once-off still image of an arbitrary viewport
    Static,
    ///< a once-off still image of a single tile
    Tile,
}

// TODO this is just a dummy
#[derive(Copy, Clone)]
pub struct CanonicalTileID {
    pub x: u32,
    pub y: u32,
    pub z: u8,
}

// TODO
#[derive(Copy, Clone)]
pub struct OverscaledTileID {
    pub canonical: CanonicalTileID,
    pub overscaledZ: u8,
}

impl OverscaledTileID {
    pub fn overscaleFactor(&self) -> u32 {
        return 1 << (self.overscaledZ - self.canonical.z);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        euclid::{Point2D, Rect, Size2D},
        sdf::{
            bidi::Char16,
            font_stack::FontStackHasher,
            geometry_tile_data::{GeometryCoordinates, SymbolGeometryTileLayer},
            glyph::{Glyph, GlyphDependencies, GlyphMap, GlyphMetrics, Glyphs},
            glyph_atlas::{GlyphPosition, GlyphPositionMap, GlyphPositions},
            image::ImageMap,
            image_atlas::ImagePositions,
            layout::{
                layout::{BucketParameters, LayerTypeInfo, LayoutParameters},
                symbol_feature::{SymbolGeometryTileFeature, VectorGeometryTileFeature},
                symbol_layout::{FeatureIndex, LayerProperties, SymbolLayer, SymbolLayout},
            },
            style_types::SymbolLayoutProperties_Unevaluated,
            CanonicalTileID, MapMode, OverscaledTileID,
        },
    };

    #[test]
    fn test() {
        let fontStack = vec![
            "Open Sans Regular".to_string(),
            "Arial Unicode MS Regular".to_string(),
        ];

        let mut glyphDependencies = GlyphDependencies::new();

        let tile_id = OverscaledTileID {
            canonical: CanonicalTileID { x: 0, y: 0, z: 0 },
            overscaledZ: 0,
        };
        let mut parameters = BucketParameters {
            tileID: tile_id,
            mode: MapMode::Continuous,
            pixelRatio: 1.0,
            layerType: LayerTypeInfo,
        };
        let mut layout = SymbolLayout::new(
            &parameters,
            &vec![LayerProperties {
                id: "layer".to_string(),
                layer: SymbolLayer {
                    layout: SymbolLayoutProperties_Unevaluated,
                },
            }],
            Box::new(SymbolGeometryTileLayer {
                name: "layer".to_string(),
                features: vec![SymbolGeometryTileFeature::new(Box::new(
                    VectorGeometryTileFeature {
                        geometry: vec![GeometryCoordinates(vec![Point2D::new(1024, 1024)])],
                    },
                ))],
            }),
            &mut LayoutParameters {
                bucketParameters: &mut parameters.clone(),
                glyphDependencies: &mut glyphDependencies,
                imageDependencies: &mut Default::default(),
                availableImages: &mut Default::default(),
            },
        )
        .unwrap();

        assert_eq!(glyphDependencies.len(), 1);

        // Now we prepare the data, when we have the glyphs available

        let image_positions = ImagePositions::new();

        let mut glyphPosition = GlyphPosition {
            rect: Rect::new(Point2D::new(0, 0), Size2D::new(10, 10)),
            metrics: GlyphMetrics {
                width: 18,
                height: 18,
                left: 2,
                top: -8,
                advance: 21,
            },
        };
        let glyphPositions: GlyphPositions = GlyphPositions::from([(
            FontStackHasher::new(&fontStack),
            GlyphPositionMap::from([('中' as Char16, glyphPosition)]),
        )]);

        let mut glyph = Glyph::default();
        glyph.id = '中' as Char16;
        glyph.metrics = glyphPosition.metrics;

        let glyphs: GlyphMap = GlyphMap::from([(
            FontStackHasher::new(&fontStack),
            Glyphs::from([('中' as Char16, Some(glyph))]),
        )]);

        let empty_image_map = ImageMap::new();
        layout.prepareSymbols(&glyphs, &glyphPositions, &empty_image_map, &image_positions);

        let mut output = HashMap::new();
        layout.createBucket(
            image_positions,
            Box::new(FeatureIndex),
            &mut output,
            false,
            false,
            &tile_id.canonical,
        );

        println!("{:#?}", output)
    }
}
