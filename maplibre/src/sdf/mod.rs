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
    legacy::TileSpace,
    plugin::Plugin,
    render::{
        eventually::Eventually,
        graph::RenderGraph,
        shaders::{
            SDFShaderFeatureMetadata, ShaderLayerMetadata, ShaderSymbolVertex,
            ShaderSymbolVertexNew,
        },
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

pub mod collision_system;
mod populate_world_system;
mod queue_system;
mod render_commands;
mod resource;
mod resource_system;
mod upload_system;

pub mod tessellation;
mod tessellation_new;
pub mod text;

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
    pub new_buffer: OverAlignedVertexBuffer<ShaderSymbolVertexNew, IndexDataType>, // TODO
    pub features: Vec<Feature>,
}

#[derive(Default)]
pub struct SymbolLayersDataComponent {
    pub layers: Vec<SymbolLayerData>,
}

impl TileComponent for SymbolLayersDataComponent {}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        euclid::{Point2D, Rect, Size2D},
        legacy::{
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
            overscaled_z: 0,
        };
        let mut parameters = BucketParameters {
            tile_id: tile_id,
            mode: MapMode::Continuous,
            pixel_ratio: 1.0,
            layer_type: LayerTypeInfo,
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
                bucket_parameters: &mut parameters.clone(),
                glyph_dependencies: &mut glyphDependencies,
                image_dependencies: &mut Default::default(),
                available_images: &mut Default::default(),
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
        layout.prepare_symbols(&glyphs, &glyphPositions, &empty_image_map, &image_positions);

        let mut output = HashMap::new();
        layout.create_bucket(
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
