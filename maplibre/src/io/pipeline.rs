use crate::coords::WorldTileCoords;
use crate::io::geometry_index::IndexedGeometry;
use crate::io::TileRequestID;
use crate::render::ShaderVertex;
use crate::tessellation::{IndexDataType, OverAlignedVertexBuffer};
use downcast_rs::{impl_downcast, Downcast};
use geozero::mvt::tile;
use std::any::Any;
use std::marker::PhantomData;
use std::process::Output;
use std::sync::mpsc;

pub trait PipelineProcessor: Downcast {
    fn finished_tile_tesselation(&mut self, request_id: TileRequestID, coords: &WorldTileCoords) {}
    fn unavailable_layer(&mut self, coords: &WorldTileCoords, layer_name: &str) {}
    fn finished_layer_tesselation(
        &mut self,
        coords: &WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        // Holds for each feature the count of indices.
        feature_indices: Vec<u32>,
        layer_data: tile::Layer,
    ) {
    }
    fn finished_layer_indexing(
        &mut self,
        coords: &WorldTileCoords,
        geometries: Vec<IndexedGeometry<f64>>,
    ) {
    }
}

impl_downcast!(PipelineProcessor);

pub struct PipelineContext {
    pub processor: Box<dyn PipelineProcessor>,
}

impl PipelineContext {
    pub fn teardown(self) -> Box<dyn PipelineProcessor> {
        self.processor
    }
}

pub trait Processable {
    type Input;
    type Output;

    fn process(&self, input: Self::Input, context: &mut PipelineContext) -> Self::Output;
}

pub struct PipelineStep<P, N>
where
    P: Processable,
    N: Processable<Input = P::Output>,
{
    process: P,
    next: N,
}

impl<P, N> PipelineStep<P, N>
where
    P: Processable,
    N: Processable<Input = P::Output>,
{
    pub fn new(process: P, next: N) -> Self {
        Self { process, next }
    }
}

impl<P, N> Processable for PipelineStep<P, N>
where
    P: Processable,
    N: Processable<Input = P::Output>,
{
    type Input = P::Input;
    type Output = N::Output;

    fn process(&self, input: Self::Input, context: &mut PipelineContext) -> Self::Output {
        let output = self.process.process(input, context);
        self.next.process(output, context)
    }
}

pub struct EndStep<I> {
    phantom: PhantomData<I>,
}

impl<I> Default for EndStep<I> {
    fn default() -> Self {
        Self {
            phantom: PhantomData::default(),
        }
    }
}

impl<I> Processable for EndStep<I> {
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
        ClosureProcessable, EndStep, PipelineContext, PipelineProcessor, PipelineStep, Processable,
    };
    use std::sync::mpsc;

    pub struct DummyPipelineProcessor;

    impl PipelineProcessor for DummyPipelineProcessor {}

    fn add_one(input: u32, context: &mut PipelineContext) -> u8 {
        input as u8 + 1
    }

    fn add_two(input: u8, context: &mut PipelineContext) -> u32 {
        input as u32 + 2
    }

    #[test]
    fn test() {
        let mut context = PipelineContext {
            processor: Box::new(DummyPipelineProcessor),
        };
        let output: u32 = PipelineStep {
            process: add_two as fn(u8, &mut PipelineContext) -> u32,
            next: EndStep::default(),
        }
        .process(5u8, &mut context);

        assert_eq!(output, 7);

        let output = PipelineStep {
            process: add_one as fn(u32, &mut PipelineContext) -> u8,
            next: PipelineStep {
                process: add_two as fn(u8, &mut PipelineContext) -> u32,
                next: EndStep::default(),
            },
        }
        .process(5u32, &mut context);

        assert_eq!(output, 8);

        let mut a = 3;
        let closure = |input: u8, context: &mut PipelineContext| -> u32 {
            return input as u32 + 2 + a;
        };
        let output: u32 = PipelineStep {
            process: ClosureProcessable {
                func: closure,
                phantom_i: Default::default(),
            },
            next: EndStep::default(),
        }
        .process(5u8, &mut context);

        assert_eq!(output, 10);

        let processable =
            ClosureProcessable::from(|input: u8, context: &mut PipelineContext| -> u32 {
                return input as u32 + 2 + a;
            });
        let output: u32 = PipelineStep {
            process: processable,
            next: EndStep::default(),
        }
        .process(5u8, &mut context);

        assert_eq!(output, 10);

        let output: u32 = PipelineStep::<ClosureProcessable<_, u8, u32>, _>::new(
            (|input: u8, context: &mut PipelineContext| -> u32 {
                return input as u32 + 2 + a;
            })
            .into(),
            EndStep::<u32>::default(),
        )
        .process(5u8, &mut context);

        assert_eq!(output, 10);
    }
}
