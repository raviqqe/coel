#[cfg(test)]
mod test {
    use std::ops::{Generator, GeneratorState};
    use std::sync::Arc;

    use futures::executor::block_on;
    use futures::prelude::*;
    use test::Bencher;

    use super::super::core::{Normal, Result};

    fn normal_function() -> Result {
        Ok(Normal::Nil.into())
    }

    #[bench]
    fn bench_normal_function(b: &mut Bencher) {
        b.iter(|| normal_function().unwrap());
    }

    fn generator_function() -> impl Generator<Yield = Async<Never>, Return = Result> {
        return || {
            if false {
                yield Async::Pending;
            }

            Ok(Normal::Nil.into()): Result
        };
    }

    #[bench]
    fn bench_generator_function(b: &mut Bencher) {
        b.iter(|| {
            let mut g = generator_function();

            loop {
                match g.resume() {
                    GeneratorState::Complete(r) => {
                        r.unwrap();
                        break;
                    }
                    GeneratorState::Yielded(_) => {}
                }
            }
        });
    }

    #[async_move]
    fn async_function() -> Result {
        Ok(Normal::Nil.into())
    }

    #[bench]
    fn bench_async_function(b: &mut Bencher) {
        b.iter(|| block_on(async_function()).unwrap());
    }

    #[async_move(boxed_send)]
    fn boxed_async_function() -> Result {
        Ok(Normal::Nil.into())
    }

    #[bench]
    fn bench_boxed_async_function(b: &mut Bencher) {
        b.iter(|| block_on(boxed_async_function()).unwrap());
    }

    #[bench]
    fn bench_box(b: &mut Bencher) {
        b.iter(|| Box::new(Ok(Normal::Nil.into()): Result).unwrap());
    }

    #[bench]
    fn bench_arc(b: &mut Bencher) {
        b.iter(|| {
            Arc::try_unwrap(Arc::new(Ok(Normal::Nil.into()): Result))
                .unwrap()
                .unwrap()
        });
    }
}
