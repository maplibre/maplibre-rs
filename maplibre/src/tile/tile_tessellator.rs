use crate::render::ShaderVertex;
use crate::tessellation::zero_tessellator::ZeroTessellator;
use crate::tessellation::{IndexDataType, OverAlignedVertexBuffer};
use crate::Style;
use geozero::error::GeozeroError;
use geozero::mvt::tile::Layer;
use geozero::GeozeroDatasource;

#[derive(Default)]
pub struct TileTessellator;

impl TileTessellator {
    /// Tessellate a layer with the given style.
    ///
    /// Return the vertex buffer that contains a list of `ShaderVertex` and the feature indices
    /// which hold the count of indices for each feature.
    pub fn tessellate_layer(
        layer: &mut Layer,
        style: &Style,
    ) -> Result<
        (
            OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
            Vec<u32>,
        ),
        GeozeroError,
    > {
        // TODO : Apply tessellation with styles
        let mut tessellator = ZeroTessellator::<IndexDataType>::default();
        layer
            .process(&mut tessellator)
            .map(|()| (tessellator.buffer.into(), tessellator.feature_indices))
    }
}
