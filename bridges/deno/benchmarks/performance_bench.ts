/**
 * Performance Benchmarks (TDD Cycle 5)
 *
 * Benchmarks for Deno FFI bridge performance:
 * - FFI overhead: < 500ns per call
 * - Throughput: > 10K requests/second
 * - Memory: < 1KB per handler
 *
 * Quality Standards:
 * - Zero SATD comments
 * - Complexity ≤ 20
 * - Realistic workload scenarios
 */

import { FfiBridge } from "../ffi.ts";

/**
 * Benchmark: FFI call overhead
 *
 * Target: < 500ns per call
 * Measures: Time to call version() (simple FFI operation)
 */
Deno.bench("FFI Overhead: version() call", { group: "ffi-overhead" }, () => {
  const bridge = new FfiBridge();
  bridge.version();
  bridge.close();
});

/**
 * Benchmark: Handler execution overhead
 *
 * Target: < 500ns per executeHandler() call
 * Measures: Time to execute handler with small payload
 */
Deno.bench(
  "FFI Overhead: executeHandler() with small payload",
  { group: "ffi-overhead" },
  () => {
    const bridge = new FfiBridge();
    const params = new TextEncoder().encode(JSON.stringify({ test: true }));
    bridge.executeHandler("test_handler", params);
    bridge.close();
  },
);

/**
 * Benchmark: Throughput - Sequential execution
 *
 * Target: > 10K requests/second
 * Measures: Sustained throughput for sequential requests
 */
Deno.bench(
  "Throughput: 1000 sequential executeHandler() calls",
  { group: "throughput" },
  () => {
    const bridge = new FfiBridge();
    const params = new TextEncoder().encode(JSON.stringify({ value: 42 }));

    for (let i = 0; i < 1000; i++) {
      bridge.executeHandler("test_handler", params);
    }

    bridge.close();
  },
);

/**
 * Benchmark: Throughput - version() calls
 *
 * Simpler operation for baseline comparison
 */
Deno.bench(
  "Throughput: 1000 sequential version() calls",
  { group: "throughput" },
  () => {
    const bridge = new FfiBridge();

    for (let i = 0; i < 1000; i++) {
      bridge.version();
    }

    bridge.close();
  },
);

/**
 * Benchmark: Create/close overhead
 *
 * Measures: Time to create and close bridge instances
 */
Deno.bench(
  "Overhead: Create and close bridge",
  { group: "lifecycle" },
  () => {
    const bridge = new FfiBridge();
    bridge.close();
  },
);

/**
 * Benchmark: Large payload handling
 *
 * Measures: Performance with 10KB payloads
 */
Deno.bench(
  "Large Payload: 10KB executeHandler()",
  { group: "payload-size" },
  () => {
    const bridge = new FfiBridge();
    const largeData = { data: "x".repeat(10 * 1024) };
    const params = new TextEncoder().encode(JSON.stringify(largeData));

    try {
      bridge.executeHandler("test_handler", params);
    } catch {
      // Error acceptable
    }

    bridge.close();
  },
);

/**
 * Benchmark: Medium payload handling
 *
 * Measures: Performance with 1KB payloads
 */
Deno.bench(
  "Medium Payload: 1KB executeHandler()",
  { group: "payload-size" },
  () => {
    const bridge = new FfiBridge();
    const mediumData = { data: "x".repeat(1024) };
    const params = new TextEncoder().encode(JSON.stringify(mediumData));

    try {
      bridge.executeHandler("test_handler", params);
    } catch {
      // Error acceptable
    }

    bridge.close();
  },
);

/**
 * Benchmark: Empty payload handling
 *
 * Measures: Baseline performance with minimal payload
 */
Deno.bench(
  "Small Payload: Empty JSON executeHandler()",
  { group: "payload-size" },
  () => {
    const bridge = new FfiBridge();
    const params = new TextEncoder().encode("{}");

    try {
      bridge.executeHandler("test_handler", params);
    } catch {
      // Error acceptable
    }

    bridge.close();
  },
);

/**
 * Benchmark: Reuse bridge instance
 *
 * Measures: Performance when reusing single bridge instance
 * This should be faster than create/use/close per operation
 */
Deno.bench(
  "Reuse: 100 calls on single bridge instance",
  { group: "reuse" },
  () => {
    const bridge = new FfiBridge();
    const params = new TextEncoder().encode("{}");

    for (let i = 0; i < 100; i++) {
      try {
        bridge.executeHandler("test_handler", params);
      } catch {
        // Error acceptable
      }
    }

    bridge.close();
  },
);

/**
 * Benchmark: Mixed operations
 *
 * Measures: Performance with mixed version/execute operations
 */
Deno.bench(
  "Mixed: Alternating version() and executeHandler()",
  { group: "mixed" },
  () => {
    const bridge = new FfiBridge();
    const params = new TextEncoder().encode("{}");

    for (let i = 0; i < 50; i++) {
      bridge.version();
      try {
        bridge.executeHandler("test_handler", params);
      } catch {
        // Error acceptable
      }
    }

    bridge.close();
  },
);
