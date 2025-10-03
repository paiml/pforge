use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use pforge_runtime::{Handler, HandlerRegistry};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct BenchInput {
    value: i32,
}

#[derive(Debug, Serialize, JsonSchema)]
struct BenchOutput {
    result: i32,
}

struct AddHandler;

#[async_trait::async_trait]
impl Handler for AddHandler {
    type Input = BenchInput;
    type Output = BenchOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> pforge_runtime::Result<Self::Output> {
        Ok(BenchOutput {
            result: input.value + 1,
        })
    }
}

struct MultiplyHandler;

#[async_trait::async_trait]
impl Handler for MultiplyHandler {
    type Input = BenchInput;
    type Output = BenchOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> pforge_runtime::Result<Self::Output> {
        Ok(BenchOutput {
            result: input.value * 2,
        })
    }
}

fn handler_dispatch_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("handler_dispatch");

    // Benchmark single handler dispatch
    group.bench_function("single_handler", |b| {
        let mut registry = HandlerRegistry::new();
        registry.register("add", AddHandler);

        let input = serde_json::json!({"value": 42});
        let input_bytes = serde_json::to_vec(&input).unwrap();

        b.to_async(&rt).iter(|| async {
            let result = registry
                .dispatch(black_box("add"), black_box(&input_bytes))
                .await;
            black_box(result.unwrap());
        });
    });

    // Benchmark with multiple handlers (registry lookup)
    group.bench_function("multi_handler_lookup", |b| {
        let mut registry = HandlerRegistry::new();
        registry.register("add", AddHandler);
        registry.register("multiply", MultiplyHandler);

        let input = serde_json::json!({"value": 42});
        let input_bytes = serde_json::to_vec(&input).unwrap();

        b.to_async(&rt).iter(|| async {
            let result = registry
                .dispatch(black_box("multiply"), black_box(&input_bytes))
                .await;
            black_box(result.unwrap());
        });
    });

    // Benchmark registry with many handlers
    for num_handlers in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("registry_scale", num_handlers),
            num_handlers,
            |b, &num| {
                let mut registry = HandlerRegistry::new();

                // Register many handlers
                for i in 0..num {
                    registry.register(format!("handler_{}", i), AddHandler);
                }
                registry.register("target", MultiplyHandler);

                let input = serde_json::json!({"value": 42});
                let input_bytes = serde_json::to_vec(&input).unwrap();

                b.to_async(&rt).iter(|| async {
                    let result = registry
                        .dispatch(black_box("target"), black_box(&input_bytes))
                        .await;
                    black_box(result.unwrap());
                });
            },
        );
    }

    group.finish();
}

fn schema_generation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("schema_generation");

    group.bench_function("input_schema", |b| {
        b.iter(|| {
            let schema = AddHandler::input_schema();
            black_box(schema);
        });
    });

    group.bench_function("output_schema", |b| {
        b.iter(|| {
            let schema = AddHandler::output_schema();
            black_box(schema);
        });
    });

    group.finish();
}

fn serialization_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    let input = BenchInput { value: 42 };
    let output = BenchOutput { result: 84 };

    group.bench_function("input_serialize", |b| {
        b.iter(|| {
            let bytes = serde_json::to_vec(black_box(&input)).unwrap();
            black_box(bytes);
        });
    });

    group.bench_function("output_serialize", |b| {
        b.iter(|| {
            let bytes = serde_json::to_vec(black_box(&output)).unwrap();
            black_box(bytes);
        });
    });

    let input_bytes = serde_json::to_vec(&input).unwrap();
    group.bench_function("input_deserialize", |b| {
        b.iter(|| {
            let input: BenchInput = serde_json::from_slice(black_box(&input_bytes)).unwrap();
            black_box(input);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    handler_dispatch_benchmark,
    schema_generation_benchmark,
    serialization_benchmark
);
criterion_main!(benches);
