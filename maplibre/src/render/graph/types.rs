/// The sample count when doing multisampling.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SampleCount {
    One = 1,
    Four = 4,
}

impl Default for SampleCount {
    fn default() -> Self {
        Self::One
    }
}

impl TryFrom<u8> for SampleCount {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::One,
            4 => Self::Four,
            v => return Err(v),
        })
    }
}

impl SampleCount {
    /// Determines if a resolve texture is needed for this texture.
    pub fn needs_resolve(self) -> bool {
        self != Self::One
    }
}

/// Output of wgpu_profiler's code.
pub type RendererStatistics = Vec<wgpu_profiler::GpuTimerScopeResult>;
