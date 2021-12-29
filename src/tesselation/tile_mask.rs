use std::ops::Range;

use lyon::tessellation::VertexBuffers;

use crate::render::shader_ffi::GpuVertexUniform;
use crate::tesselation::Tesselated;

const EXTENT: f32 = 4096.0;

pub struct TileMask();

impl Tesselated<u32> for TileMask {
    fn tesselate_stroke(
        &self,
        _buffer: &mut VertexBuffers<GpuVertexUniform, u32>,
        _prim_id: u32,
    ) -> Range<u32> {
        0..0
    }

    fn tesselate_fill(
        &self,
        buffer: &mut VertexBuffers<GpuVertexUniform, u32>,
        prim_id: u32,
    ) -> Range<u32> {
        let initial_indices_count = buffer.indices.len();

        buffer.vertices = vec![
            GpuVertexUniform::new([0.0, 0.0], [0.0, 0.0], prim_id),
            GpuVertexUniform::new([EXTENT, 0.0], [0.0, 0.0], prim_id),
            GpuVertexUniform::new([0.0, EXTENT], [0.0, 0.0], prim_id),
            GpuVertexUniform::new([EXTENT, EXTENT], [0.0, 0.0], prim_id),
        ];

        buffer.indices = vec![0, 2, 1, 3, 2, 1];

        initial_indices_count as u32..buffer.indices.len() as u32
    }
}
