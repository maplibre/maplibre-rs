use std::marker::PhantomData;
use std::process::Output;

pub trait Processable {
    type Input;
    type Output;

    fn process(&self, input: Self::Input) -> Self::Output;
}

pub struct PipelineStep<P, N>
where
    P: Processable,
    N: Processable<Input = P::Output>,
{
    func: P,
    next: N,
}

impl<P, N> Processable for PipelineStep<P, N>
where
    P: Processable,
    N: Processable<Input = P::Output>,
{
    type Input = P::Input;
    type Output = N::Output;

    fn process(&self, input: Self::Input) -> Self::Output {
        let output = self.func.process(input);
        self.next.process(output)
    }
}

#[derive(Default)]
pub struct EndStep<I> {
    phantom: PhantomData<I>,
}

impl<I> Processable for EndStep<I> {
    type Input = I;
    type Output = I;

    fn process(&self, input: Self::Input) -> Self::Output {
        input
    }
}

impl<I, O> Processable for &fn(I) -> O {
    type Input = I;
    type Output = O;

    fn process(&self, input: Self::Input) -> Self::Output {
        (self)(input)
    }
}

impl<I, O> Processable for fn(I) -> O {
    type Input = I;
    type Output = O;

    fn process(&self, input: Self::Input) -> Self::Output {
        (self)(input)
    }
}

pub struct FnProcessable<I: 'static, O: 'static> {
    func: &'static fn(I) -> O,
}

impl<I, O> Processable for FnProcessable<I, O> {
    type Input = I;
    type Output = O;

    fn process(&self, input: Self::Input) -> Self::Output {
        (self.func)(input)
    }
}

pub struct ClosureProcessable<F, I, O>
where
    F: Fn(I) -> O,
{
    func: F,
    phantom_i: PhantomData<I>,
    phantom_o: PhantomData<O>,
}

impl<F, I, O> Processable for ClosureProcessable<F, I, O>
where
    F: Fn(I) -> O,
{
    type Input = I;
    type Output = O;

    fn process(&self, input: Self::Input) -> Self::Output {
        (self.func)(input)
    }
}

pub struct Closure2Processable<I, O> {
    func: fn(I) -> O,
}

impl<I, O> Processable for Closure2Processable<I, O> {
    type Input = I;
    type Output = O;

    fn process(&self, input: Self::Input) -> Self::Output {
        (self.func)(input)
    }
}

impl<I, O> Processable for dyn Fn(I) -> O {
    type Input = I;
    type Output = O;

    fn process(&self, input: Self::Input) -> Self::Output {
        (self)(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::io::pipeline::{
        Closure2Processable, ClosureProcessable, EndStep, FnProcessable, PipelineStep, Processable,
    };

    fn add_one(input: u32) -> u8 {
        input as u8 + 1
    }

    fn add_two(input: u8) -> u32 {
        input as u32 + 2
    }

    #[test]
    fn test() {
        let output: u32 = PipelineStep {
            func: FnProcessable {
                func: &(add_two as fn(u8) -> u32),
            },
            next: EndStep::default(),
        }
        .process(5u8);

        assert_eq!(output, 7);

        let output = PipelineStep {
            func: &(add_one as fn(u32) -> u8),
            next: PipelineStep {
                func: &(add_two as fn(u8) -> u32),
                next: EndStep::default(),
            },
        }
        .process(5);

        assert_eq!(output, 8);

        let output: u32 = PipelineStep {
            func: ClosureProcessable {
                func: |input: u8| -> u32 {
                    return input as u32 + 2;
                },
                phantom_i: Default::default(),
                phantom_o: Default::default(),
            },
            next: EndStep::default(),
        }
        .process(5u8);

        assert_eq!(output, 7);

        let output: u32 = PipelineStep {
            func: Closure2Processable {
                func: |input: u8| -> u32 { input as u32 + 2 },
            },
            next: EndStep::default(),
        }
        .process(5u8);

        assert_eq!(output, 7);
    }
}
