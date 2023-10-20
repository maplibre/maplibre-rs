pub struct BackingBufferDescriptor<B> {
    /// The buffer which is used
    pub(crate) buffer: B,
    /// The size of buffer
    pub(crate) inner_size: wgpu::BufferAddress,
}

impl<B> BackingBufferDescriptor<B> {
    pub fn new(buffer: B, inner_size: wgpu::BufferAddress) -> Self {
        Self { buffer, inner_size }
    }
}
