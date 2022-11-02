use std::marker::PhantomData;

use downcast_rs::Downcast;
use geozero::mvt::tile;

use crate::{
    coords::WorldTileCoords,
    error::Error,
    io::geometry_index::IndexedGeometry,
    render::ShaderVertex,
    tessellation::{IndexDataType, OverAlignedVertexBuffer},
};

/// Processes events which happen during the pipeline execution
// FIXME (wasm-executor): handle results for messages below
pub trait PipelineProcessor: Downcast {
    fn tile_finished(&mut self, _coords: &WorldTileCoords) -> Result<(), Error> {
        Ok(())
    }
    fn layer_unavailable(
        &mut self,
        _coords: &WorldTileCoords,
        _layer_name: &str,
    ) -> Result<(), Error> {
        Ok(())
    }
    fn layer_tesselation_finished(
        &mut self,
        _coords: &WorldTileCoords,
        _buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        _feature_indices: Vec<u32>,
        _layer_data: tile::Layer,
    ) -> Result<(), Error> {
        Ok(())
    }
    fn layer_indexing_finished(
        &mut self,
        _coords: &WorldTileCoords,
        _geometries: Vec<IndexedGeometry<f64>>,
    ) -> Result<(), Error> {
        Ok(())
    }
}

/// Context which is available to each step within a [`DataPipeline`]
pub struct PipelineContext {
    processor: Box<dyn PipelineProcessor>,
}

impl PipelineContext {
    pub fn new<P>(processor: P) -> Self
    where
        P: PipelineProcessor,
    {
        Self {
            processor: Box::new(processor),
        }
    }

    pub fn take_processor<P>(self) -> Option<Box<P>>
    where
        P: PipelineProcessor,
    {
        self.processor.into_any().downcast::<P>().ok()
    }

    pub fn processor_mut(&mut self) -> &mut dyn PipelineProcessor {
        self.processor.as_mut()
    }
}

pub trait Processable {
    type Input;
    type Output;

    fn process(&self, input: Self::Input, context: &mut PipelineContext) -> Self::Output;
}

/// A pipeline which consists of multiple steps. Steps are [`Processable`] workloads. Later steps
/// depend on previous ones.
pub struct DataPipeline<P, N>
where
    P: Processable,
    N: Processable<Input = P::Output>,
{
    step: P,
    next_step: N,
}

impl<P, N> DataPipeline<P, N>
where
    P: Processable,
    N: Processable<Input = P::Output>,
{
    pub fn new(step: P, next_step: N) -> Self {
        Self { step, next_step }
    }
}

impl<P, N> Processable for DataPipeline<P, N>
where
    P: Processable,
    N: Processable<Input = P::Output>,
{
    type Input = P::Input;
    type Output = N::Output;

    fn process(&self, input: Self::Input, context: &mut PipelineContext) -> Self::Output {
        let output = self.step.process(input, context);
        self.next_step.process(output, context)
    }
}

/// Marks the end of a [`DataPipeline`]
pub struct PipelineEnd<I> {
    phantom: PhantomData<I>,
}

impl<I> Default for PipelineEnd<I> {
    fn default() -> Self {
        Self {
            phantom: PhantomData::default(),
        }
    }
}

impl<I> Processable for PipelineEnd<I> {
    type Input = I;
    type Output = I;

    fn process(&self, input: Self::Input, _context: &mut PipelineContext) -> Self::Output {
        input
    }
}

impl<I, O> Processable for &fn(input: I, context: &mut PipelineContext) -> O {
    type Input = I;
    type Output = O;

    fn process(&self, input: Self::Input, context: &mut PipelineContext) -> Self::Output {
        (self)(input, context)
    }
}

impl<I, O> Processable for fn(input: I, context: &mut PipelineContext) -> O {
    type Input = I;
    type Output = O;

    fn process(&self, input: Self::Input, context: &mut PipelineContext) -> Self::Output {
        (self)(input, context)
    }
}

// TODO: Implementing Processable directly on Fn is not possible for some strange reason:
//       https://github.com/rust-lang/rust/issues/25041
pub struct ClosureProcessable<F, I, O>
where
    F: Fn(I, &mut PipelineContext) -> O,
{
    func: F,
    phantom_i: PhantomData<I>,
}

impl<F, I, O> From<F> for ClosureProcessable<F, I, O>
where
    F: Fn(I, &mut PipelineContext) -> O,
{
    fn from(func: F) -> Self {
        ClosureProcessable {
            func,
            phantom_i: PhantomData::default(),
        }
    }
}

impl<F, I, O> Processable for ClosureProcessable<F, I, O>
where
    F: Fn(I, &mut PipelineContext) -> O,
{
    type Input = I;
    type Output = O;

    fn process(&self, input: Self::Input, context: &mut PipelineContext) -> Self::Output {
        (self.func)(input, context)
    }
}

#[cfg(test)]
mod tests {
    use crate::io::pipeline::{
        ClosureProcessable, DataPipeline, PipelineContext, PipelineEnd, PipelineProcessor,
        Processable,
    };

    pub struct DummyPipelineProcessor;

    impl PipelineProcessor for DummyPipelineProcessor {}

    fn add_one(input: u32, _context: &mut PipelineContext) -> u8 {
        input as u8 + 1
    }

    fn add_two(input: u8, _context: &mut PipelineContext) -> u32 {
        input as u32 + 2
    }

    #[test]
    fn test_fn_pointer() {
        let mut context = PipelineContext::new(DummyPipelineProcessor);
        let output: u32 = DataPipeline::new(
            add_two as fn(u8, &mut PipelineContext) -> u32,
            PipelineEnd::default(),
        )
        .process(5u8, &mut context);
        assert_eq!(output, 7);

        let output: u32 = DataPipeline::new(
            add_one as fn(u32, &mut PipelineContext) -> u8,
            DataPipeline::new(
                add_two as fn(u8, &mut PipelineContext) -> u32,
                PipelineEnd::default(),
            ),
        )
        .process(5u32, &mut context);
        assert_eq!(output, 8);
    }

    #[test]
    fn test_closure() {
        let mut context = PipelineContext::new(DummyPipelineProcessor);
        let outer_value = 3;

        // using from()
        let closure =
            ClosureProcessable::from(|input: u8, _context: &mut PipelineContext| -> u32 {
                input as u32 + 2 + outer_value
            });
        let output: u32 =
            DataPipeline::new(closure, PipelineEnd::default()).process(5u8, &mut context);
        assert_eq!(output, 10);

        // with into()
        let output: u32 = DataPipeline::<ClosureProcessable<_, u8, u32>, _>::new(
            (|input: u8, _context: &mut PipelineContext| -> u32 { input as u32 + 2 + outer_value })
                .into(),
            PipelineEnd::<u32>::default(),
        )
        .process(5u8, &mut context);
        assert_eq!(output, 10);
    }
}
