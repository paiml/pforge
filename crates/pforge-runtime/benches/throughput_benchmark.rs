use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use pforge_runtime::{Handler, HandlerRegistry};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ThroughputInput {
    id: u64,
    data: String,
}

#[derive(Debug, Serialize, JsonSchema)]
struct ThroughputOutput {
    id: u64,
    processed: bool,
}

struct ThroughputHandler;

#[async_trait::async_trait]
impl Handler for ThroughputHandler {
    type Input = ThroughputInput;
    type Output = ThroughputOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> pforge_runtime::Result<Self::Output> {
        Ok(ThroughputOutput {
            id: input.id,
            processed: true,
        })
    }
}

fn sequential_throughput(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("sequential_throughput");

    for size in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut registry = HandlerRegistry::new();
            registry.register("throughput", ThroughputHandler);

            b.to_async(&rt).iter(|| async {
                for i in 0..size {
                    let input = ThroughputInput {
                        id: i,
                        data: format!("payload_{}", i),
                    };
                    let input_bytes = serde_json::to_vec(&input).unwrap();
                    let result = registry.dispatch("throughput", &input_bytes).await;
                    black_box(result.unwrap());
                }
            });
        });
    }

    group.finish();
}

fn concurrent_throughput(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .build()
        .unwrap();

    let mut group = c.benchmark_group("concurrent_throughput");

    for num_tasks in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*num_tasks));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tasks),
            num_tasks,
            |b, &num_tasks| {
                let mut registry = HandlerRegistry::new();
                registry.register("throughput", ThroughputHandler);
                let registry = Arc::new(RwLock::new(registry));

                b.to_async(&rt).iter(|| {
                    let registry = registry.clone();
                    async move {
                        let mut handles = Vec::new();

                        for i in 0..num_tasks {
                            let registry = registry.clone();
                            let handle = tokio::spawn(async move {
                                let input = ThroughputInput {
                                    id: i,
                                    data: format!("payload_{}", i),
                                };
                                let input_bytes = serde_json::to_vec(&input).unwrap();
                                let reg = registry.read().await;
                                let result = reg.dispatch("throughput", &input_bytes).await;
                                black_box(result.unwrap());
                            });
                            handles.push(handle);
                        }

                        for handle in handles {
                            handle.await.unwrap();
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

fn payload_size_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("payload_size");

    for size_kb in [1, 10, 100].iter() {
        let size_bytes = size_kb * 1024;
        group.throughput(Throughput::Bytes(size_bytes as u64));

        group.bench_with_input(
            BenchmarkId::new("dispatch", format!("{}KB", size_kb)),
            size_kb,
            |b, &_size_kb| {
                let mut registry = HandlerRegistry::new();
                registry.register("throughput", ThroughputHandler);

                let payload = "x".repeat(size_bytes);
                let input = ThroughputInput {
                    id: 1,
                    data: payload,
                };
                let input_bytes = serde_json::to_vec(&input).unwrap();

                b.to_async(&rt).iter(|| async {
                    let result = registry
                        .dispatch("throughput", black_box(&input_bytes))
                        .await;
                    black_box(result.unwrap());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    sequential_throughput,
    concurrent_throughput,
    payload_size_benchmark
);
criterion_main!(benches);
