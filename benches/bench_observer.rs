use async_observer::{Observer, Subject};
use async_trait::async_trait;
use criterion::{Criterion, criterion_group, criterion_main};
use std::sync::Arc;
use tokio::runtime::Runtime;

// A simple observer that does nothing, to measure the overhead of the observer pattern itself.
struct EmptyObserver;

#[async_trait]
impl Observer<u32> for EmptyObserver {
    async fn update(&self, _data: &u32) {
        // Do nothing
    }
}

// A helper function to create a subject with a specified number of observers
fn setup_subject(num_observers: usize) -> Arc<Subject<u32>> {
    let subject = Arc::new(Subject::default());
    for _ in 0..num_observers {
        subject.attach(Arc::new(EmptyObserver));
    }
    subject
}

fn observer_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let num_observers: usize = 10_000;

    // Group for `attach` and `detach` benchmarks
    let mut group = c.benchmark_group("Observer Operations");

    // Benchmarking `attach`
    group.bench_function("attach_observer", |b| {
        let subject = setup_subject(num_observers);
        b.iter(|| {
            let _handle = rt.block_on(async {
                subject.attach(Arc::new(EmptyObserver));
            });
        });
    });

    // Benchmarking `notify`
    group.bench_function("notify_all", |b| {
        let subject = setup_subject(num_observers);
        b.to_async(&rt).iter(|| async {
            let data = 42;
            subject.notify(&data).await;
        });
    });

    // Benchmarking `detach`
    group.bench_function("detach_observer", |b| {
        let subject = setup_subject(num_observers);
        let handle = subject.attach(Arc::new(EmptyObserver));
        let id_to_remove = handle.get_id();

        b.iter(|| {
            rt.block_on(async {
                subject.detach(id_to_remove);
            });
        });
    });

    group.finish();
}

criterion_group!(benches, observer_benchmark);
criterion_main!(benches);
