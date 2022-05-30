use crate::coords::WorldTileCoords;
use crate::io::{LayerTessellateMessage, TessellateMessage, TileRequestID, TileTessellateMessage};
use crate::render::ShaderVertex;
use crate::tessellation::{IndexDataType, OverAlignedVertexBuffer};
use downcast_rs::{impl_downcast, Downcast};
use geozero::mvt::tile;
use std::any::Any;
use std::marker::PhantomData;
use std::process::Output;
use std::sync::mpsc;

pub trait PipelineProcessor: Downcast {
    fn finished_tile_tesselation(&mut self, request_id: TileRequestID, coords: &WorldTileCoords);
    fn unavailable_layer(&mut self, coords: &WorldTileCoords, layer_name: &str);
    fn finished_layer_tesselation(
        &mut self,
        coords: &WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        // Holds for each feature the count of indices.
        feature_indices: Vec<u32>,
        layer_data: tile::Layer,
    );
}

impl_downcast!(PipelineProcessor);

pub struct HeadedPipelineProcessor {
    pub message_sender: mpsc::Sender<TessellateMessage>,
}

impl PipelineProcessor for HeadedPipelineProcessor {
    fn finished_tile_tesselation(&mut self, request_id: TileRequestID, coords: &WorldTileCoords) {
        self.message_sender
            .send(TessellateMessage::Tile(TileTessellateMessage {
                request_id,
                coords: *coords,
            }))
            .unwrap();
    }

    fn unavailable_layer(&mut self, coords: &WorldTileCoords, layer_name: &str) {
        self.message_sender.send(TessellateMessage::Layer(
            LayerTessellateMessage::UnavailableLayer {
                coords: *coords,
                layer_name: layer_name.to_owned(),
            },
        ));
    }
    fn finished_layer_tesselation(
        &mut self,
        coords: &WorldTileCoords,
        buffer: OverAlignedVertexBuffer<ShaderVertex, IndexDataType>,
        feature_indices: Vec<u32>,
        layer_data: tile::Layer,
    ) {
        self.message_sender
            .send(TessellateMessage::Layer(
                LayerTessellateMessage::TessellatedLayer {
                    coords: *coords,
                    buffer,
                    feature_indices,
                    layer_data,
                },
            ))
            .unwrap();
    }
}

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

#[derive(Default)]
pub struct EndStep<I> {
    phantom: PhantomData<I>,
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

pub struct FnProcessable<I: 'static, O: 'static> {
    func: &'static fn(I, context: &mut PipelineContext) -> O,
}

impl<I, O> Processable for FnProcessable<I, O> {
    type Input = I;
    type Output = O;

    fn process(&self, input: Self::Input, context: &mut PipelineContext) -> Self::Output {
        (self.func)(input, context)
    }
}

pub struct ClosureProcessable<F, I, O>
where
    F: Fn(I, &mut PipelineContext) -> O,
{
    func: F,
    phantom_i: PhantomData<I>,
    phantom_o: PhantomData<O>,
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

pub struct Closure2Processable<I, O> {
    func: fn(I, context: &mut PipelineContext) -> O,
}

impl<I, O> Processable for Closure2Processable<I, O> {
    type Input = I;
    type Output = O;

    fn process(&self, input: Self::Input, context: &mut PipelineContext) -> Self::Output {
        (self.func)(input, context)
    }
}

pub mod steps {
    use crate::io::pipeline::{EndStep, PipelineContext, PipelineStep, Processable};
    use crate::io::{TileRequest, TileRequestID};
    use crate::tessellation::zero_tessellator::ZeroTessellator;
    use crate::tessellation::IndexDataType;
    use geozero::GeozeroDatasource;
    use prost::Message;
    use std::collections::HashSet;

    pub struct ParseTileStep {}

    impl Processable for ParseTileStep {
        type Input = (TileRequest, TileRequestID, Box<[u8]>);
        type Output = (TileRequest, TileRequestID, geozero::mvt::Tile);

        fn process(
            &self,
            (tile_request, request_id, data): Self::Input,
            _context: &mut PipelineContext,
        ) -> Self::Output {
            let tile = geozero::mvt::Tile::decode(data.as_ref()).expect("failed to load tile");
            (tile_request, request_id, tile)
        }
    }

    pub struct IndexLayerStep {}

    pub struct TessellateLayerStep {}

    impl Processable for TessellateLayerStep {
        type Input = (TileRequest, TileRequestID, geozero::mvt::Tile);
        type Output = ();

        fn process(
            &self,
            (tile_request, request_id, mut tile): Self::Input,
            context: &mut PipelineContext,
        ) -> Self::Output {
            let coords = &tile_request.coords;

            for layer in &mut tile.layers {
                let cloned_layer = layer.clone();
                let layer_name: &str = &cloned_layer.name;
                if !tile_request.layers.contains(layer_name) {
                    continue;
                }

                tracing::info!("layer {} at {} ready", layer_name, coords);

                let mut tessellator = ZeroTessellator::<IndexDataType>::default();
                if let Err(e) = layer.process(&mut tessellator) {
                    context.processor.unavailable_layer(coords, layer_name);

                    tracing::error!(
                        "layer {} at {} tesselation failed {:?}",
                        layer_name,
                        &coords,
                        e
                    );
                } else {
                    context.processor.finished_layer_tesselation(
                        coords,
                        tessellator.buffer.into(),
                        tessellator.feature_indices,
                        cloned_layer,
                    )
                }
            }

            let available_layers: HashSet<_> = tile
                .layers
                .iter()
                .map(|layer| layer.name.clone())
                .collect::<HashSet<_>>();

            for missing_layer in tile_request.layers.difference(&available_layers) {
                context.processor.unavailable_layer(coords, missing_layer);

                tracing::info!(
                    "requested layer {} at {} not found in tile",
                    missing_layer,
                    &coords
                );
            }

            tracing::info!("tile tessellated at {} finished", &tile_request.coords);

            context
                .processor
                .finished_tile_tesselation(request_id, &tile_request.coords);
        }
    }

    pub fn build_vector_tile_pipeline(
    ) -> impl Processable<Input = <ParseTileStep as Processable>::Input> {
        PipelineStep {
            process: ParseTileStep {},
            next: PipelineStep {
                process: TessellateLayerStep {},
                next: EndStep::default(),
            },
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::io::pipeline::steps::build_vector_tile_pipeline;
        use crate::io::pipeline::{HeadedPipelineProcessor, PipelineContext, Processable};
        use crate::io::TileRequest;
        use std::sync::mpsc;

        #[test]
        fn test() {
            let mut context = PipelineContext {
                processor: Box::new(HeadedPipelineProcessor {
                    message_sender: mpsc::channel().0,
                }),
            };

            let pipeline = build_vector_tile_pipeline();
            let output = pipeline.process(
                (
                    TileRequest {
                        coords: (0, 0, 0).into(),
                        layers: Default::default(),
                    },
                    0,
                    Box::new([0]),
                ),
                &mut context,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::io::pipeline::{
        Closure2Processable, ClosureProcessable, EndStep, FnProcessable, HeadedPipelineProcessor,
        PipelineContext, PipelineStep, Processable,
    };
    use std::sync::mpsc;

    fn add_one(input: u32, context: &mut PipelineContext) -> u8 {
        input as u8 + 1
    }

    fn add_two(input: u8, context: &mut PipelineContext) -> u32 {
        input as u32 + 2
    }

    #[test]
    fn test() {
        let mut context = PipelineContext {
            processor: Box::new(HeadedPipelineProcessor {
                message_sender: mpsc::channel().0,
            }),
        };
        let output: u32 = PipelineStep {
            process: FnProcessable {
                func: &(add_two as fn(u8, &mut PipelineContext) -> u32),
            },
            next: EndStep::default(),
        }
        .process(5u8, &mut context);

        assert_eq!(output, 7);

        let output = PipelineStep {
            process: &(add_one as fn(u32, &mut PipelineContext) -> u8),
            next: PipelineStep {
                process: &(add_two as fn(u8, &mut PipelineContext) -> u32),
                next: EndStep::default(),
            },
        }
        .process(5u32, &mut context);

        assert_eq!(output, 8);

        let output: u32 = PipelineStep {
            process: ClosureProcessable {
                func: |input: u8, context| -> u32 {
                    return input as u32 + 2;
                },
                phantom_i: Default::default(),
                phantom_o: Default::default(),
            },
            next: EndStep::default(),
        }
        .process(5u8, &mut context);

        assert_eq!(output, 7);

        let output: u32 = PipelineStep {
            process: Closure2Processable {
                func: |input: u8, context| -> u32 { input as u32 + 2 },
            },
            next: EndStep::default(),
        }
        .process(5u8, &mut context);

        assert_eq!(output, 7);
    }
}
