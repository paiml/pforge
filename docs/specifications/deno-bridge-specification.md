# pforge Deno/TypeScript Bridge Specification

**Code Name**: Deno Bridge (PFORGE-DENO-1000)
**Version**: 1.0.0-alpha
**License**: MIT
**Status**: IMPLEMENTATION IN PROGRESS

## Executive Summary

The Deno Bridge enables pforge MCP servers to execute handlers written in Deno/TypeScript with native TypeScript support, modern ES modules, and Deno's security model. Built using Deno's FFI API with zero-copy optimization, the bridge maintains <500ns overhead while providing full type safety through TypeScript definitions.

**Design Philosophy**: Deno-first × Zero-Copy FFI × Type Safety Guaranteed

**Core Metrics**:
- FFI overhead: < 500ns per call
- Type safety: 100% (TypeScript strict mode)
- Memory overhead: < 1KB per handler instance
- Cold start: < 50ms (Deno runtime initialization)
- Quality: TDG ≥ 0.75, zero SATD, complexity ≤ 20

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Deno FFI Integration](#deno-ffi-integration)
3. [TypeScript Handler System](#typescript-handler-system)
4. [TDD Implementation Roadmap](#tdd-implementation-roadmap)
5. [Property Testing Strategy](#property-testing-strategy)
6. [Mutation Testing](#mutation-testing)
7. [Quality Gates](#quality-gates)
8. [Performance Targets](#performance-targets)
9. [Security Model](#security-model)
10. [Examples](#examples)

---

## Architecture Overview

### Component Stack

```
┌─────────────────────────────────────────────────────────────┐
│              pforge Runtime (Rust)                          │
│  ┌────────────────────────────────────────────────────┐    │
│  │         HandlerRegistry (FxHashMap)                │    │
│  │  ┌────────┐  ┌────────┐  ┌──────────────────────┐ │    │
│  │  │ Native │  │ Python │  │   Deno Bridge        │ │    │
│  │  │Handler │  │ Bridge │  │   Handler            │ │    │
│  │  └────────┘  └────────┘  └──────────┬───────────┘ │    │
│  └────────────────────────────────────┬─┘              │    │
└────────────────────────────────────────┼────────────────────┘
                                         │
                       C ABI FFI (Stable Interface)
                                         │
                                         ▼
┌─────────────────────────────────────────────────────────────┐
│                Deno Bridge Layer                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Deno FFI (dlopen)                                   │  │
│  │  ┌─────────────────────────────────────────────┐    │  │
│  │  │  Type-Safe TypeScript Interface             │    │  │
│  │  │  - Parameter validation (Zod schemas)       │    │  │
│  │  │  - Result marshaling                        │    │  │
│  │  │  - Error handling                           │    │  │
│  │  └─────────────────────────────────────────────┘    │  │
│  └──────────────────────────────────────────────────────┘  │
│                         │                                   │
│                         ▼                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Deno Runtime (V8 + Tokio)                          │  │
│  │  - Native TypeScript support                        │  │
│  │  - ES modules                                       │  │
│  │  - Permission system                               │  │
│  │  - Web standards API                               │  │
│  └──────────────────────────────────────────────────────┘  │
│                         │                                   │
│                         ▼                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  User TypeScript Handlers                           │  │
│  │  - Type-safe parameter access                       │  │
│  │  - Async/await support                              │  │
│  │  - Web APIs (fetch, crypto, etc.)                   │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Directory Structure

```
bridges/deno/
├── mod.ts                      # Main Deno module (FFI interface)
├── handler.ts                  # Handler base class
├── types.ts                    # TypeScript type definitions
├── error.ts                    # Error handling
├── validator.ts                # Zod-based validation
├── ffi.ts                      # Low-level FFI bindings
├── deno.json                   # Deno configuration
├── import_map.json             # Import map
├── tests/
│   ├── unit/
│   │   ├── ffi_test.ts         # FFI interface tests
│   │   ├── handler_test.ts     # Handler tests
│   │   └── validator_test.ts   # Validation tests
│   ├── integration/
│   │   ├── bridge_test.ts      # Bridge integration tests
│   │   └── e2e_test.ts         # End-to-end tests
│   └── property/
│       ├── roundtrip_test.ts   # Roundtrip property tests
│       └── fuzz_test.ts        # Fuzzing tests
├── examples/
│   ├── hello/
│   │   ├── handler.ts          # Simple handler example
│   │   └── pforge.yaml         # Configuration
│   ├── calculator/
│   │   ├── handler.ts          # Math operations
│   │   └── pforge.yaml
│   └── async_fetch/
│       ├── handler.ts          # Async HTTP handler
│       └── pforge.yaml
├── benchmarks/
│   ├── ffi_overhead_bench.ts   # FFI overhead benchmark
│   ├── throughput_bench.ts     # Throughput benchmark
│   └── memory_bench.ts         # Memory usage benchmark
└── README.md
```

---

## Deno FFI Integration

### FFI Interface Definition

```typescript
// bridges/deno/ffi.ts

/**
 * FFI Result structure matching Rust's FfiResult
 *
 * Memory Layout (C repr):
 * - code: i32 (4 bytes, aligned)
 * - data: *mut u8 (8 bytes pointer)
 * - data_len: usize (8 bytes)
 * - error: *const c_char (8 bytes pointer)
 */
export interface FfiResult {
  code: number;           // i32: 0 = success, non-zero = error
  data: Deno.PointerValue;    // *mut u8: JSON result bytes
  data_len: number;       // usize: result length
  error: Deno.PointerValue;   // *const char: error message
}

/**
 * Deno FFI symbol definitions for pforge_bridge
 */
export const SYMBOLS = {
  // Get pforge version
  pforge_version: {
    parameters: [],
    result: "pointer" as const,
  },

  // Initialize handler
  pforge_handler_init: {
    parameters: ["pointer" as const, "pointer" as const],
    result: "pointer" as const,
  },

  // Execute handler
  pforge_handler_execute: {
    parameters: ["pointer" as const, "pointer" as const, "usize" as const],
    result: {
      struct: ["i32", "pointer", "usize", "pointer"] as const,
    },
  },

  // Free handler
  pforge_handler_free: {
    parameters: ["pointer" as const],
    result: "void" as const,
  },

  // Free result
  pforge_result_free: {
    parameters: [{
      struct: ["i32", "pointer", "usize", "pointer"] as const,
    }],
    result: "void" as const,
  },
} as const;

/**
 * Load pforge bridge library
 *
 * Searches for library in platform-specific locations:
 * - Linux: libpforge_bridge.so
 * - macOS: libpforge_bridge.dylib
 * - Windows: pforge_bridge.dll
 */
export function loadLibrary(): Deno.DynamicLibrary<typeof SYMBOLS> {
  const libPaths = getLibraryPaths();

  for (const path of libPaths) {
    try {
      return Deno.dlopen(path, SYMBOLS);
    } catch (e) {
      // Try next path
      continue;
    }
  }

  throw new Error(
    "Could not find pforge_bridge library. " +
    "Run 'cargo build -p pforge-bridge' first. " +
    `Searched paths: ${libPaths.join(", ")}`
  );
}

/**
 * Get platform-specific library paths
 */
function getLibraryPaths(): string[] {
  const os = Deno.build.os;
  const baseDir = new URL("../../target", import.meta.url).pathname;

  switch (os) {
    case "linux":
      return [
        `${baseDir}/release/libpforge_bridge.so`,
        `${baseDir}/debug/libpforge_bridge.so`,
        "/usr/local/lib/libpforge_bridge.so",
        "/usr/lib/libpforge_bridge.so",
      ];
    case "darwin":
      return [
        `${baseDir}/release/libpforge_bridge.dylib`,
        `${baseDir}/debug/libpforge_bridge.dylib`,
        "/usr/local/lib/libpforge_bridge.dylib",
      ];
    case "windows":
      return [
        `${baseDir}/release/pforge_bridge.dll`,
        `${baseDir}/debug/pforge_bridge.dll`,
      ];
    default:
      throw new Error(`Unsupported platform: ${os}`);
  }
}
```

### Low-Level FFI Wrapper

```typescript
// bridges/deno/ffi.ts (continued)

/**
 * Safe wrapper around FFI library
 *
 * Handles:
 * - Pointer lifecycle management
 * - UTF-8 encoding/decoding
 * - Memory safety
 * - Error propagation
 */
export class FfiBridge {
  private lib: Deno.DynamicLibrary<typeof SYMBOLS>;

  constructor() {
    this.lib = loadLibrary();
  }

  /**
   * Get pforge version
   */
  version(): string {
    const ptr = this.lib.symbols.pforge_version();
    if (ptr === null) {
      throw new Error("Failed to get pforge version");
    }
    return Deno.UnsafePointerView.getCString(ptr);
  }

  /**
   * Initialize handler
   *
   * @param handlerType - Handler type ("deno")
   * @param config - JSON configuration string
   * @returns Opaque handler pointer
   */
  initHandler(handlerType: string, config: string): Deno.PointerValue {
    const typePtr = toCString(handlerType);
    const configPtr = toCString(config);

    try {
      const handle = this.lib.symbols.pforge_handler_init(typePtr, configPtr);

      if (handle === null) {
        throw new Error("Failed to initialize handler");
      }

      return handle;
    } finally {
      // Free temporary C strings
      Deno.UnsafePointer.of(typePtr)?.free();
      Deno.UnsafePointer.of(configPtr)?.free();
    }
  }

  /**
   * Execute handler
   *
   * @param handle - Handler pointer from initHandler
   * @param params - JSON parameters as Uint8Array
   * @returns Result as Uint8Array
   */
  executeHandler(handle: Deno.PointerValue, params: Uint8Array): Uint8Array {
    const paramsPtr = Deno.UnsafePointer.of(params);

    const result = this.lib.symbols.pforge_handler_execute(
      handle,
      paramsPtr,
      params.length
    );

    try {
      if (result.code !== 0) {
        const errorMsg = result.error !== null
          ? Deno.UnsafePointerView.getCString(result.error)
          : "Unknown error";
        throw new Error(`Handler execution failed (code ${result.code}): ${errorMsg}`);
      }

      if (result.data === null || result.data_len === 0) {
        return new Uint8Array(0);
      }

      // Copy result data before freeing
      const view = new Deno.UnsafePointerView(result.data);
      const resultData = new Uint8Array(result.data_len);
      for (let i = 0; i < result.data_len; i++) {
        resultData[i] = view.getUint8(i);
      }

      return resultData;
    } finally {
      // Always free the result
      this.lib.symbols.pforge_result_free(result);
    }
  }

  /**
   * Free handler
   */
  freeHandler(handle: Deno.PointerValue): void {
    if (handle !== null) {
      this.lib.symbols.pforge_handler_free(handle);
    }
  }

  /**
   * Close library
   */
  close(): void {
    this.lib.close();
  }
}

/**
 * Convert JavaScript string to null-terminated C string
 */
function toCString(str: string): Deno.PointerValue {
  const encoder = new TextEncoder();
  const bytes = encoder.encode(str + "\0");
  return Deno.UnsafePointer.of(bytes);
}
```

---

## TypeScript Handler System

### Handler Base Class

```typescript
// bridges/deno/handler.ts

import { z } from "https://deno.land/x/zod@v3.22.4/mod.ts";
import type { FfiBridge } from "./ffi.ts";

/**
 * Base handler interface
 */
export interface HandlerConfig {
  name?: string;
  description?: string;
  timeout?: number; // milliseconds
}

/**
 * Handler execution context
 */
export interface HandlerContext {
  handlerName: string;
  startTime: number;
  timeout?: number;
}

/**
 * Abstract base class for Deno handlers
 *
 * Provides:
 * - Type-safe parameter validation
 * - Automatic JSON serialization
 * - Error handling
 * - Timeout management
 * - Async/await support
 */
export abstract class PforgeHandler<TInput = unknown, TOutput = unknown> {
  protected config: HandlerConfig;
  protected inputSchema?: z.ZodType<TInput>;
  protected outputSchema?: z.ZodType<TOutput>;

  constructor(config: HandlerConfig = {}) {
    this.config = config;
  }

  /**
   * Main handler method - implement in subclasses
   */
  abstract handle(params: TInput, ctx: HandlerContext): Promise<TOutput>;

  /**
   * Execute handler with validation and error handling
   *
   * @internal Called by FFI bridge
   */
  async execute(paramsJson: Uint8Array, ctx: HandlerContext): Promise<Uint8Array> {
    const decoder = new TextDecoder();
    const encoder = new TextEncoder();

    try {
      // Parse input JSON
      const paramsStr = decoder.decode(paramsJson);
      let params: unknown;

      try {
        params = JSON.parse(paramsStr);
      } catch (e) {
        throw new ValidationError(`Invalid JSON: ${e.message}`);
      }

      // Validate input schema
      const validatedParams = this.inputSchema
        ? this.inputSchema.parse(params)
        : params as TInput;

      // Execute with timeout
      const result = this.config.timeout
        ? await this.executeWithTimeout(validatedParams, ctx)
        : await this.handle(validatedParams, ctx);

      // Validate output schema
      const validatedResult = this.outputSchema
        ? this.outputSchema.parse(result)
        : result;

      // Serialize result
      const resultJson = JSON.stringify(validatedResult);
      return encoder.encode(resultJson);

    } catch (error) {
      // Re-throw known errors
      if (error instanceof ValidationError || error instanceof TimeoutError) {
        throw error;
      }

      // Wrap unknown errors
      throw new HandlerError(
        `Handler execution failed: ${error.message}`,
        { cause: error }
      );
    }
  }

  /**
   * Execute handler with timeout
   */
  private async executeWithTimeout(
    params: TInput,
    ctx: HandlerContext
  ): Promise<TOutput> {
    const timeoutMs = this.config.timeout!;

    const timeoutPromise = new Promise<never>((_, reject) => {
      setTimeout(() => {
        reject(new TimeoutError(`Handler timed out after ${timeoutMs}ms`));
      }, timeoutMs);
    });

    return Promise.race([
      this.handle(params, ctx),
      timeoutPromise,
    ]);
  }
}

/**
 * Decorator for handler methods
 */
export function handler(config: HandlerConfig = {}) {
  return function <T extends { new(...args: any[]): PforgeHandler }>(
    target: T
  ): T {
    // Attach config to class
    Object.defineProperty(target.prototype, "config", {
      value: config,
      writable: false,
    });

    return target;
  };
}

/**
 * Validation error
 */
export class ValidationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ValidationError";
  }
}

/**
 * Timeout error
 */
export class TimeoutError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "TimeoutError";
  }
}

/**
 * Handler execution error
 */
export class HandlerError extends Error {
  constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = "HandlerError";
  }
}
```

### Type-Safe Parameter Validation

```typescript
// bridges/deno/validator.ts

import { z } from "https://deno.land/x/zod@v3.22.4/mod.ts";

/**
 * Create type-safe handler with Zod schemas
 *
 * Example:
 * ```ts
 * const handler = createTypedHandler(
 *   z.object({ name: z.string() }),
 *   z.object({ greeting: z.string() }),
 *   async ({ name }) => ({ greeting: `Hello, ${name}!` })
 * );
 * ```
 */
export function createTypedHandler<TInput, TOutput>(
  inputSchema: z.ZodType<TInput>,
  outputSchema: z.ZodType<TOutput>,
  handler: (params: TInput, ctx: HandlerContext) => Promise<TOutput>,
  config: HandlerConfig = {}
): PforgeHandler<TInput, TOutput> {
  return new class extends PforgeHandler<TInput, TOutput> {
    constructor() {
      super(config);
      this.inputSchema = inputSchema;
      this.outputSchema = outputSchema;
    }

    async handle(params: TInput, ctx: HandlerContext): Promise<TOutput> {
      return handler(params, ctx);
    }
  }();
}

/**
 * Common parameter schemas
 */
export const commonSchemas = {
  /**
   * String parameter
   */
  string: (options?: {
    min?: number;
    max?: number;
    pattern?: RegExp;
  }) => {
    let schema = z.string();
    if (options?.min) schema = schema.min(options.min);
    if (options?.max) schema = schema.max(options.max);
    if (options?.pattern) schema = schema.regex(options.pattern);
    return schema;
  },

  /**
   * Number parameter
   */
  number: (options?: {
    min?: number;
    max?: number;
    int?: boolean;
  }) => {
    let schema = options?.int ? z.number().int() : z.number();
    if (options?.min !== undefined) schema = schema.min(options.min);
    if (options?.max !== undefined) schema = schema.max(options.max);
    return schema;
  },

  /**
   * Array parameter
   */
  array: <T>(itemSchema: z.ZodType<T>, options?: {
    min?: number;
    max?: number;
  }) => {
    let schema = z.array(itemSchema);
    if (options?.min) schema = schema.min(options.min);
    if (options?.max) schema = schema.max(options.max);
    return schema;
  },

  /**
   * Object parameter
   */
  object: z.object,

  /**
   * Optional parameter
   */
  optional: <T>(schema: z.ZodType<T>) => schema.optional(),

  /**
   * Union type
   */
  union: z.union,

  /**
   * Literal value
   */
  literal: z.literal,
};
```

---

## TDD Implementation Roadmap

### Phase 1: Deno FFI Interface (TDD Cycles 1-5)

#### Cycle 1: FFI Library Loading (5 minutes)

**RED (2 min)**: Write failing test

```typescript
// bridges/deno/tests/unit/ffi_test.ts

import { assertEquals, assertExists } from "https://deno.land/std@0.208.0/assert/mod.ts";
import { loadLibrary } from "../../ffi.ts";

Deno.test("FFI: Load pforge_bridge library", () => {
  const lib = loadLibrary();
  assertExists(lib);
  assertExists(lib.symbols.pforge_version);
  assertExists(lib.symbols.pforge_handler_init);
  assertExists(lib.symbols.pforge_handler_execute);
  assertExists(lib.symbols.pforge_handler_free);
  assertExists(lib.symbols.pforge_result_free);
  lib.close();
});

Deno.test("FFI: Get pforge version", () => {
  const lib = loadLibrary();
  const version = lib.symbols.pforge_version();
  assertExists(version);
  lib.close();
});

Deno.test("FFI: Library not found throws error", () => {
  // Temporarily rename library to simulate not found
  // Should throw with helpful error message
  let errorThrown = false;
  try {
    // Test with invalid path
    Deno.dlopen("/invalid/path/libpforge_bridge.so", {});
  } catch (e) {
    errorThrown = true;
    assertEquals(e.message.includes("Could not find"), true);
  }
  assertEquals(errorThrown, true);
});
```

**GREEN (2 min)**: Implement minimum code

```typescript
// bridges/deno/ffi.ts (implement getLibraryPaths and loadLibrary)
```

**REFACTOR (1 min)**: Clean up, run `deno fmt`, `deno lint`

**COMMIT**: `git commit -m "feat(deno): implement FFI library loading"`

#### Cycle 2: FfiBridge Wrapper (5 minutes)

**RED**: Write failing tests for FfiBridge class

```typescript
// bridges/deno/tests/unit/ffi_test.ts (continued)

Deno.test("FfiBridge: Initialize and get version", () => {
  const bridge = new FfiBridge();
  const version = bridge.version();
  assertExists(version);
  assertEquals(typeof version, "string");
  bridge.close();
});

Deno.test("FfiBridge: Initialize handler", () => {
  const bridge = new FfiBridge();
  const handle = bridge.initHandler("deno", JSON.stringify({ test: true }));
  assertExists(handle);
  bridge.freeHandler(handle);
  bridge.close();
});

Deno.test("FfiBridge: Execute handler returns result", () => {
  const bridge = new FfiBridge();
  const handle = bridge.initHandler("deno", "{}");

  const params = new TextEncoder().encode(JSON.stringify({ value: 42 }));
  const result = bridge.executeHandler(handle, params);

  assertExists(result);
  assertEquals(result instanceof Uint8Array, true);

  bridge.freeHandler(handle);
  bridge.close();
});
```

**GREEN**: Implement FfiBridge class

**REFACTOR**: Extract helper functions, add error handling

**COMMIT**: `git commit -m "feat(deno): implement FfiBridge wrapper"`

#### Cycle 3: Error Handling (5 minutes)

**RED**: Write failing tests for error cases

```typescript
Deno.test("FfiBridge: Null handle throws error", () => {
  const bridge = new FfiBridge();

  let errorThrown = false;
  try {
    bridge.executeHandler(null as any, new Uint8Array());
  } catch (e) {
    errorThrown = true;
    assertEquals(e.message.includes("Null pointer"), true);
  }

  assertEquals(errorThrown, true);
  bridge.close();
});

Deno.test("FfiBridge: Invalid JSON throws error", () => {
  const bridge = new FfiBridge();
  const handle = bridge.initHandler("deno", "{}");

  const invalidJson = new TextEncoder().encode("not json");

  let errorThrown = false;
  try {
    bridge.executeHandler(handle, invalidJson);
  } catch (e) {
    errorThrown = true;
    assertEquals(e.message.includes("failed"), true);
  }

  assertEquals(errorThrown, true);
  bridge.freeHandler(handle);
  bridge.close();
});
```

**GREEN**: Add error checking to FfiBridge

**REFACTOR**: Extract error types, improve messages

**COMMIT**: `git commit -m "feat(deno): add FFI error handling"`

#### Cycle 4: Memory Safety (5 minutes)

**RED**: Write property tests for memory safety

```typescript
// bridges/deno/tests/property/ffi_property_test.ts

import { assertEquals } from "https://deno.land/std@0.208.0/assert/mod.ts";
import { FfiBridge } from "../../ffi.ts";

/**
 * Property: Multiple init/execute/free cycles should not leak memory
 */
Deno.test("Property: FFI operations are memory safe", () => {
  const bridge = new FfiBridge();

  // Run 1000 cycles
  for (let i = 0; i < 1000; i++) {
    const handle = bridge.initHandler("deno", "{}");
    const params = new TextEncoder().encode(JSON.stringify({ iteration: i }));
    const result = bridge.executeHandler(handle, params);
    assertEquals(result instanceof Uint8Array, true);
    bridge.freeHandler(handle);
  }

  bridge.close();
});

/**
 * Property: Closing bridge invalidates all handles
 */
Deno.test("Property: Closed bridge rejects operations", () => {
  const bridge = new FfiBridge();
  const handle = bridge.initHandler("deno", "{}");
  bridge.close();

  let errorThrown = false;
  try {
    const params = new TextEncoder().encode("{}");
    bridge.executeHandler(handle, params);
  } catch (e) {
    errorThrown = true;
  }

  assertEquals(errorThrown, true);
});
```

**GREEN**: Add lifecycle tracking to FfiBridge

**REFACTOR**: Add `FinalizationRegistry` for automatic cleanup

**COMMIT**: `git commit -m "feat(deno): ensure FFI memory safety"`

#### Cycle 5: Performance Benchmarking (5 minutes)

**RED**: Write performance tests

```typescript
// bridges/deno/benchmarks/ffi_overhead_bench.ts

import { FfiBridge } from "../ffi.ts";

Deno.bench("FFI: Initialize handler", () => {
  const bridge = new FfiBridge();
  const handle = bridge.initHandler("deno", "{}");
  bridge.freeHandler(handle);
  bridge.close();
});

Deno.bench("FFI: Execute handler (1KB payload)", () => {
  const bridge = new FfiBridge();
  const handle = bridge.initHandler("deno", "{}");

  const payload = { data: "x".repeat(1024) };
  const params = new TextEncoder().encode(JSON.stringify(payload));

  bridge.executeHandler(handle, params);
  bridge.freeHandler(handle);
  bridge.close();
});

Deno.bench("FFI: Roundtrip (init + execute + free)", () => {
  const bridge = new FfiBridge();
  const handle = bridge.initHandler("deno", "{}");
  const params = new TextEncoder().encode(JSON.stringify({ value: 42 }));
  bridge.executeHandler(handle, params);
  bridge.freeHandler(handle);
  bridge.close();
});
```

**GREEN**: Optimize hot paths in FfiBridge

**REFACTOR**: Add caching where applicable

**COMMIT**: `git commit -m "perf(deno): optimize FFI operations"`

### Phase 2: TypeScript Handler System (TDD Cycles 6-10)

#### Cycle 6: PforgeHandler Base Class (5 minutes)

**RED**: Write failing tests

```typescript
// bridges/deno/tests/unit/handler_test.ts

import { assertEquals, assertExists } from "https://deno.land/std@0.208.0/assert/mod.ts";
import { PforgeHandler, HandlerContext } from "../../handler.ts";

class TestHandler extends PforgeHandler<{ value: number }, { result: number }> {
  async handle(params: { value: number }, _ctx: HandlerContext) {
    return { result: params.value * 2 };
  }
}

Deno.test("Handler: Execute simple handler", async () => {
  const handler = new TestHandler();
  const params = new TextEncoder().encode(JSON.stringify({ value: 21 }));
  const ctx: HandlerContext = {
    handlerName: "test",
    startTime: Date.now(),
  };

  const result = await handler.execute(params, ctx);
  const resultObj = JSON.parse(new TextDecoder().decode(result));

  assertEquals(resultObj.result, 42);
});

Deno.test("Handler: Invalid JSON throws ValidationError", async () => {
  const handler = new TestHandler();
  const params = new TextEncoder().encode("not json");
  const ctx: HandlerContext = {
    handlerName: "test",
    startTime: Date.now(),
  };

  let errorThrown = false;
  try {
    await handler.execute(params, ctx);
  } catch (e) {
    errorThrown = true;
    assertEquals(e.name, "ValidationError");
  }

  assertEquals(errorThrown, true);
});
```

**GREEN**: Implement PforgeHandler base class

**REFACTOR**: Extract JSON handling, improve error types

**COMMIT**: `git commit -m "feat(deno): implement handler base class"`

#### Cycle 7: Zod Schema Validation (5 minutes)

**RED**: Write validation tests

```typescript
import { z } from "https://deno.land/x/zod@v3.22.4/mod.ts";

class ValidatedHandler extends PforgeHandler<
  { name: string; age: number },
  { message: string }
> {
  constructor() {
    super();
    this.inputSchema = z.object({
      name: z.string().min(1),
      age: z.number().int().min(0).max(150),
    });
    this.outputSchema = z.object({
      message: z.string(),
    });
  }

  async handle(params: { name: string; age: number }, _ctx: HandlerContext) {
    return { message: `${params.name} is ${params.age} years old` };
  }
}

Deno.test("Handler: Schema validation succeeds for valid input", async () => {
  const handler = new ValidatedHandler();
  const params = new TextEncoder().encode(
    JSON.stringify({ name: "Alice", age: 30 })
  );
  const ctx: HandlerContext = { handlerName: "test", startTime: Date.now() };

  const result = await handler.execute(params, ctx);
  const resultObj = JSON.parse(new TextDecoder().decode(result));

  assertEquals(resultObj.message, "Alice is 30 years old");
});

Deno.test("Handler: Schema validation fails for invalid input", async () => {
  const handler = new ValidatedHandler();
  const params = new TextEncoder().encode(
    JSON.stringify({ name: "", age: 200 }) // Invalid
  );
  const ctx: HandlerContext = { handlerName: "test", startTime: Date.now() };

  let errorThrown = false;
  try {
    await handler.execute(params, ctx);
  } catch (e) {
    errorThrown = true;
    assertEquals(e.name, "ValidationError");
  }

  assertEquals(errorThrown, true);
});
```

**GREEN**: Add schema validation to execute()

**REFACTOR**: Improve error messages from Zod

**COMMIT**: `git commit -m "feat(deno): add Zod schema validation"`

#### Cycle 8: Timeout Handling (5 minutes)

**RED**: Write timeout tests

```typescript
class SlowHandler extends PforgeHandler<{}, {}> {
  constructor(timeout?: number) {
    super({ timeout });
  }

  async handle(_params: {}, _ctx: HandlerContext) {
    await new Promise(resolve => setTimeout(resolve, 1000));
    return {};
  }
}

Deno.test("Handler: Timeout enforced", async () => {
  const handler = new SlowHandler(100); // 100ms timeout
  const params = new TextEncoder().encode("{}");
  const ctx: HandlerContext = { handlerName: "test", startTime: Date.now() };

  let errorThrown = false;
  try {
    await handler.execute(params, ctx);
  } catch (e) {
    errorThrown = true;
    assertEquals(e.name, "TimeoutError");
  }

  assertEquals(errorThrown, true);
});

Deno.test("Handler: No timeout when not configured", async () => {
  const handler = new SlowHandler(); // No timeout
  const params = new TextEncoder().encode("{}");
  const ctx: HandlerContext = { handlerName: "test", startTime: Date.now() };

  // Should complete without error
  const result = await handler.execute(params, ctx);
  assertExists(result);
});
```

**GREEN**: Implement executeWithTimeout()

**REFACTOR**: Extract timeout logic

**COMMIT**: `git commit -m "feat(deno): add handler timeout support"`

#### Cycle 9: Type-Safe Helper Functions (5 minutes)

**RED**: Write tests for createTypedHandler

```typescript
import { createTypedHandler } from "../../validator.ts";

Deno.test("Validator: createTypedHandler works", async () => {
  const handler = createTypedHandler(
    z.object({ x: z.number(), y: z.number() }),
    z.object({ sum: z.number() }),
    async ({ x, y }) => ({ sum: x + y })
  );

  const params = new TextEncoder().encode(JSON.stringify({ x: 5, y: 3 }));
  const ctx: HandlerContext = { handlerName: "test", startTime: Date.now() };

  const result = await handler.execute(params, ctx);
  const resultObj = JSON.parse(new TextDecoder().decode(result));

  assertEquals(resultObj.sum, 8);
});
```

**GREEN**: Implement createTypedHandler

**REFACTOR**: Add JSDoc, improve type inference

**COMMIT**: `git commit -m "feat(deno): add typed handler helpers"`

#### Cycle 10: Handler Decorator (5 minutes)

**RED**: Write decorator tests

```typescript
import { handler } from "../../handler.ts";

@handler({ name: "calculator", timeout: 5000 })
class CalculatorHandler extends PforgeHandler<
  { op: string; a: number; b: number },
  { result: number }
> {
  async handle(params: { op: string; a: number; b: number }, _ctx: HandlerContext) {
    const { op, a, b } = params;
    switch (op) {
      case "add": return { result: a + b };
      case "sub": return { result: a - b };
      default: throw new Error("Invalid operation");
    }
  }
}

Deno.test("Decorator: Attaches config to handler", () => {
  const h = new CalculatorHandler();
  assertEquals(h.config.name, "calculator");
  assertEquals(h.config.timeout, 5000);
});
```

**GREEN**: Implement @handler decorator

**REFACTOR**: Improve decorator type safety

**COMMIT**: `git commit -m "feat(deno): add handler decorator"`

### Phase 3: Error Handling & Type Safety (TDD Cycles 11-15)

[Similar detailed TDD cycles for error handling, type safety, etc.]

### Phase 4: Integration Tests & Examples (TDD Cycles 16-20)

[Integration tests connecting Rust bridge to Deno handlers]

### Phase 5: Quality Gates & Documentation (TDD Cycles 21-25)

[PMAT quality gates, documentation, mutation testing]

---

## Property Testing Strategy

### Roundtrip Properties

```typescript
// bridges/deno/tests/property/roundtrip_test.ts

import { assertEquals } from "https://deno.land/std@0.208.0/assert/mod.ts";
import { FfiBridge } from "../../ffi.ts";

/**
 * Property: JSON roundtrip preserves data
 *
 * For all valid JSON values v:
 *   decode(encode(v)) == v
 */
Deno.test("Property: JSON roundtrip preserves data", () => {
  const testCases = [
    null,
    true,
    false,
    0,
    42,
    -42,
    3.14,
    "",
    "hello",
    "unicode: 🦕",
    [],
    [1, 2, 3],
    {},
    { key: "value" },
    { nested: { deeply: { value: 42 } } },
  ];

  const encoder = new TextEncoder();
  const decoder = new TextDecoder();

  for (const testValue of testCases) {
    const json = JSON.stringify(testValue);
    const bytes = encoder.encode(json);
    const restored = decoder.decode(bytes);
    const parsed = JSON.parse(restored);

    assertEquals(parsed, testValue);
  }
});

/**
 * Property: Handler execution is deterministic
 *
 * For all inputs i:
 *   handle(i) == handle(i)
 */
Deno.test("Property: Handler execution is deterministic", async () => {
  class DeterministicHandler extends PforgeHandler<{ value: number }, { result: number }> {
    async handle(params: { value: number }, _ctx: HandlerContext) {
      return { result: params.value * 2 };
    }
  }

  const handler = new DeterministicHandler();
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();

  for (let i = 0; i < 100; i++) {
    const params = encoder.encode(JSON.stringify({ value: i }));
    const ctx: HandlerContext = { handlerName: "test", startTime: Date.now() };

    const result1 = await handler.execute(params, ctx);
    const result2 = await handler.execute(params, ctx);

    const obj1 = JSON.parse(decoder.decode(result1));
    const obj2 = JSON.parse(decoder.decode(result2));

    assertEquals(obj1, obj2);
  }
});
```

### Fuzzing Strategy

```typescript
// bridges/deno/tests/property/fuzz_test.ts

/**
 * Fuzz test: Random JSON inputs don't crash handler
 */
Deno.test("Fuzz: Handler handles random JSON gracefully", async () => {
  class RobustHandler extends PforgeHandler<unknown, { status: string }> {
    async handle(_params: unknown, _ctx: HandlerContext) {
      return { status: "ok" };
    }
  }

  const handler = new RobustHandler();
  const encoder = new TextEncoder();

  // Generate random JSON
  for (let i = 0; i < 1000; i++) {
    const randomValue = generateRandomJson(5); // Max depth 5
    const params = encoder.encode(JSON.stringify(randomValue));
    const ctx: HandlerContext = { handlerName: "test", startTime: Date.now() };

    try {
      await handler.execute(params, ctx);
    } catch (e) {
      // Should not crash, only throw known error types
      assertEquals(
        e instanceof ValidationError ||
        e instanceof TimeoutError ||
        e instanceof HandlerError,
        true
      );
    }
  }
});

function generateRandomJson(maxDepth: number): unknown {
  if (maxDepth === 0) {
    // Generate primitive
    const primitives = [null, true, false, 42, "test"];
    return primitives[Math.floor(Math.random() * primitives.length)];
  }

  const choice = Math.random();
  if (choice < 0.3) {
    // Array
    const length = Math.floor(Math.random() * 5);
    return Array.from({ length }, () => generateRandomJson(maxDepth - 1));
  } else if (choice < 0.6) {
    // Object
    const keys = ["a", "b", "c", "d", "e"];
    const obj: Record<string, unknown> = {};
    for (let i = 0; i < Math.floor(Math.random() * 5); i++) {
      obj[keys[i]] = generateRandomJson(maxDepth - 1);
    }
    return obj;
  } else {
    // Primitive
    return generateRandomJson(0);
  }
}
```

---

## Mutation Testing

### Mutation Testing Strategy

```toml
# .mutation-testing.toml (if using stryker or similar for TypeScript)

[mutate]
files = [
  "bridges/deno/**/*.ts",
]

[exclude]
files = [
  "bridges/deno/tests/**",
  "bridges/deno/benchmarks/**",
  "bridges/deno/examples/**",
]

[mutations]
arithmetic = true           # +, -, *, /, %
conditional = true          # <, >, <=, >=, ==, !=
logical = true              # &&, ||, !
assignment = true           # =, +=, -=
string = true               # String literals
return = true               # Return values

[thresholds]
mutation_score = 90         # Minimum 90% mutation kill rate
```

### Critical Mutation Test Cases

```typescript
// bridges/deno/tests/mutation/critical_mutations_test.ts

/**
 * Mutation: Ensure error code checking is critical
 *
 * Original:  if (result.code !== 0)
 * Mutant:    if (result.code !== 1)  // Should be caught by tests
 */
Deno.test("Mutation: Error code check is tested", async () => {
  const bridge = new FfiBridge();
  const handle = bridge.initHandler("deno", "{}");

  // Force error by passing invalid params
  const invalidParams = new Uint8Array([0xFF, 0xFF]);

  let errorThrown = false;
  try {
    bridge.executeHandler(handle, invalidParams);
  } catch (e) {
    errorThrown = true;
  }

  assertEquals(errorThrown, true);
  bridge.freeHandler(handle);
  bridge.close();
});

/**
 * Mutation: Ensure timeout comparison is critical
 *
 * Original:  if (elapsed > timeout)
 * Mutant:    if (elapsed >= timeout)  // Should be caught
 */
Deno.test("Mutation: Timeout comparison is tested", async () => {
  class TimedHandler extends PforgeHandler<{}, {}> {
    constructor() {
      super({ timeout: 100 });
    }

    async handle(_params: {}, _ctx: HandlerContext) {
      await new Promise(resolve => setTimeout(resolve, 150));
      return {};
    }
  }

  const handler = new TimedHandler();
  const params = new TextEncoder().encode("{}");
  const ctx: HandlerContext = { handlerName: "test", startTime: Date.now() };

  let timedOut = false;
  try {
    await handler.execute(params, ctx);
  } catch (e) {
    if (e instanceof TimeoutError) {
      timedOut = true;
    }
  }

  assertEquals(timedOut, true);
});
```

---

## Quality Gates

### PMAT Quality Configuration

```yaml
# bridges/deno/.pmat/quality-gates.yaml

gates:
  - name: complexity
    tool: deno task complexity
    max_cyclomatic: 20
    max_cognitive: 15
    fail_on_violation: true

  - name: satd
    tool: grep -r "TODO\|FIXME\|HACK\|XXX" --exclude-dir=tests
    max_count: 0
    fail_on_violation: true
    exclude_patterns:
      - "# Phase marker:"  # Allow phase markers in specs

  - name: test_coverage
    tool: deno task coverage
    min_line_coverage: 80
    min_branch_coverage: 75
    fail_on_violation: true

  - name: type_check
    tool: deno check mod.ts
    fail_on_violation: true

  - name: linting
    tool: deno lint
    fail_on_violation: true

  - name: formatting
    tool: deno fmt --check
    fail_on_violation: true

  - name: dependency_audit
    tool: deno task audit
    fail_on_violation: true
```

### Deno Configuration

```json
// bridges/deno/deno.json
{
  "tasks": {
    "test": "deno test --allow-ffi --allow-read",
    "test:watch": "deno test --allow-ffi --allow-read --watch",
    "coverage": "deno test --allow-ffi --allow-read --coverage=coverage && deno coverage coverage --lcov > coverage.lcov",
    "bench": "deno bench --allow-ffi --allow-read",
    "complexity": "deno run --allow-read https://deno.land/x/complexity/cli.ts **/*.ts",
    "audit": "deno info --json | jq '.modules | length'",
    "lint": "deno lint",
    "fmt": "deno fmt",
    "check": "deno check mod.ts"
  },
  "fmt": {
    "lineWidth": 100,
    "indentWidth": 2,
    "semiColons": true,
    "singleQuote": false
  },
  "lint": {
    "rules": {
      "tags": ["recommended"],
      "include": ["ban-untagged-todo"],
      "exclude": []
    }
  },
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true
  }
}
```

### Pre-Commit Hook

```bash
#!/bin/bash
# bridges/deno/.git/hooks/pre-commit

set -e

echo "🔍 Running Deno quality gates..."

# Type checking
echo "📝 Type checking..."
deno check mod.ts

# Linting
echo "🔍 Linting..."
deno lint

# Formatting
echo "✨ Checking formatting..."
deno fmt --check

# Tests
echo "🧪 Running tests..."
deno test --allow-ffi --allow-read --quiet

# Coverage
echo "📊 Checking coverage..."
deno task coverage --quiet
MIN_COVERAGE=80
COVERAGE=$(deno coverage coverage --quiet | grep "Covered" | awk '{print $2}' | tr -d '%')
if (( $(echo "$COVERAGE < $MIN_COVERAGE" | bc -l) )); then
  echo "❌ Coverage $COVERAGE% is below minimum $MIN_COVERAGE%"
  exit 1
fi

# SATD check
echo "🚨 Checking for technical debt comments..."
SATD_COUNT=$(grep -r "TODO\|FIXME\|HACK\|XXX" --exclude-dir=tests --exclude-dir=.git . | wc -l || true)
if [ "$SATD_COUNT" -gt 0 ]; then
  echo "❌ Found $SATD_COUNT SATD comments"
  grep -r "TODO\|FIXME\|HACK\|XXX" --exclude-dir=tests --exclude-dir=.git . || true
  exit 1
fi

echo "✅ All quality gates passed!"
```

---

## Performance Targets

### Performance Metrics

| Metric | Target | Rationale |
|--------|--------|-----------|
| FFI overhead | < 500ns | Deno FFI is slower than Go cgo but faster than Python ctypes |
| Cold start | < 50ms | Deno runtime initialization is fast |
| Handler dispatch | < 10μs | Includes JSON parse + validation + execute |
| Memory per handler | < 1KB | Deno's V8 isolates are lightweight |
| Throughput | > 10K req/s | Sequential execution on single core |

### Benchmarking Suite

```typescript
// bridges/deno/benchmarks/comprehensive_bench.ts

import { FfiBridge } from "../ffi.ts";
import { PforgeHandler, HandlerContext } from "../handler.ts";

// Baseline: Empty handler
class NoOpHandler extends PforgeHandler<{}, {}> {
  async handle(_params: {}, _ctx: HandlerContext) {
    return {};
  }
}

Deno.bench("Baseline: Empty handler", async () => {
  const handler = new NoOpHandler();
  const params = new TextEncoder().encode("{}");
  const ctx: HandlerContext = { handlerName: "noop", startTime: Date.now() };
  await handler.execute(params, ctx);
});

// Small payload
class EchoHandler extends PforgeHandler<{ msg: string }, { msg: string }> {
  async handle(params: { msg: string }, _ctx: HandlerContext) {
    return { msg: params.msg };
  }
}

Deno.bench("Small payload: Echo handler (100 bytes)", async () => {
  const handler = new EchoHandler();
  const params = new TextEncoder().encode(
    JSON.stringify({ msg: "x".repeat(100) })
  );
  const ctx: HandlerContext = { handlerName: "echo", startTime: Date.now() };
  await handler.execute(params, ctx);
});

// Large payload
Deno.bench("Large payload: Echo handler (10KB)", async () => {
  const handler = new EchoHandler();
  const params = new TextEncoder().encode(
    JSON.stringify({ msg: "x".repeat(10240) })
  );
  const ctx: HandlerContext = { handlerName: "echo", startTime: Date.now() };
  await handler.execute(params, ctx);
});

// Validation overhead
import { z } from "https://deno.land/x/zod@v3.22.4/mod.ts";

class ValidatedEchoHandler extends PforgeHandler<{ msg: string }, { msg: string }> {
  constructor() {
    super();
    this.inputSchema = z.object({ msg: z.string().min(1).max(10000) });
    this.outputSchema = z.object({ msg: z.string() });
  }

  async handle(params: { msg: string }, _ctx: HandlerContext) {
    return { msg: params.msg };
  }
}

Deno.bench("Validation: Zod schema validation", async () => {
  const handler = new ValidatedEchoHandler();
  const params = new TextEncoder().encode(
    JSON.stringify({ msg: "test" })
  );
  const ctx: HandlerContext = { handlerName: "echo", startTime: Date.now() };
  await handler.execute(params, ctx);
});

// Async operations
class AsyncHandler extends PforgeHandler<{}, { timestamp: number }> {
  async handle(_params: {}, _ctx: HandlerContext) {
    // Simulate async I/O
    await new Promise(resolve => setTimeout(resolve, 1));
    return { timestamp: Date.now() };
  }
}

Deno.bench("Async: Handler with 1ms delay", async () => {
  const handler = new AsyncHandler();
  const params = new TextEncoder().encode("{}");
  const ctx: HandlerContext = { handlerName: "async", startTime: Date.now() };
  await handler.execute(params, ctx);
});
```

---

## Security Model

### Deno Permissions

```typescript
// bridges/deno/permissions.ts

/**
 * Required Deno permissions for bridge operations
 */
export const REQUIRED_PERMISSIONS = {
  // FFI access to load native library
  ffi: true,

  // Read access for loading library
  read: [
    "../target/release/libpforge_bridge.so",
    "../target/debug/libpforge_bridge.so",
    "/usr/local/lib/libpforge_bridge.so",
    "/usr/lib/libpforge_bridge.so",
  ],
} as const;

/**
 * Check if all required permissions are granted
 */
export async function checkPermissions(): Promise<boolean> {
  // Check FFI permission
  const ffiStatus = await Deno.permissions.query({ name: "ffi" });
  if (ffiStatus.state !== "granted") {
    console.error("FFI permission required: --allow-ffi");
    return false;
  }

  // Check read permission
  for (const path of REQUIRED_PERMISSIONS.read) {
    const readStatus = await Deno.permissions.query({
      name: "read",
      path,
    });
    if (readStatus.state !== "granted") {
      console.error(`Read permission required for: ${path}`);
      return false;
    }
  }

  return true;
}
```

### Handler Isolation

```typescript
/**
 * Run handler in isolated context with restricted permissions
 */
export async function runIsolated<TInput, TOutput>(
  handler: PforgeHandler<TInput, TOutput>,
  params: TInput,
  permissions: Partial<Deno.PermissionOptions> = {}
): Promise<TOutput> {
  // Create worker with restricted permissions
  const worker = new Worker(
    new URL("./worker.ts", import.meta.url).href,
    {
      type: "module",
      deno: {
        permissions: {
          net: permissions.net ?? false,
          read: permissions.read ?? false,
          write: permissions.write ?? false,
          env: permissions.env ?? false,
          run: permissions.run ?? false,
          ffi: false, // Never allow FFI in handler
        },
      },
    }
  );

  return new Promise((resolve, reject) => {
    worker.onmessage = (e) => {
      if (e.data.error) {
        reject(new Error(e.data.error));
      } else {
        resolve(e.data.result);
      }
      worker.terminate();
    };

    worker.onerror = (e) => {
      reject(e);
      worker.terminate();
    };

    worker.postMessage({ params });
  });
}
```

---

## Examples

### Example 1: Hello World

```typescript
// bridges/deno/examples/hello/handler.ts

import { PforgeHandler, HandlerContext } from "../../mod.ts";
import { z } from "https://deno.land/x/zod@v3.22.4/mod.ts";

/**
 * Simple hello world handler
 */
export class HelloHandler extends PforgeHandler<
  { name: string },
  { greeting: string }
> {
  constructor() {
    super({ name: "hello", description: "Say hello" });

    this.inputSchema = z.object({
      name: z.string().min(1).max(100),
    });

    this.outputSchema = z.object({
      greeting: z.string(),
    });
  }

  async handle(params: { name: string }, _ctx: HandlerContext) {
    return {
      greeting: `Hello, ${params.name}! 🦕`,
    };
  }
}
```

```yaml
# bridges/deno/examples/hello/pforge.yaml
forge:
  name: hello-deno-server
  version: 0.1.0
  transport: stdio

tools:
  - type: bridge
    name: hello
    language: deno
    module: examples/hello/handler.ts
    handler: HelloHandler
    description: "Say hello in TypeScript"
    params:
      name:
        type: string
        required: true
        description: "Name to greet"
```

### Example 2: Calculator with Validation

```typescript
// bridges/deno/examples/calculator/handler.ts

import { createTypedHandler } from "../../validator.ts";
import { z } from "https://deno.land/x/zod@v3.22.4/mod.ts";

const Operation = z.enum(["add", "subtract", "multiply", "divide"]);

export const calculatorHandler = createTypedHandler(
  z.object({
    operation: Operation,
    a: z.number(),
    b: z.number(),
  }),
  z.object({
    result: z.number(),
  }),
  async ({ operation, a, b }) => {
    switch (operation) {
      case "add":
        return { result: a + b };
      case "subtract":
        return { result: a - b };
      case "multiply":
        return { result: a * b };
      case "divide":
        if (b === 0) {
          throw new Error("Division by zero");
        }
        return { result: a / b };
    }
  },
  {
    name: "calculator",
    description: "Perform arithmetic operations",
    timeout: 5000,
  }
);
```

### Example 3: Async HTTP Fetch

```typescript
// bridges/deno/examples/async_fetch/handler.ts

import { PforgeHandler, HandlerContext } from "../../mod.ts";
import { z } from "https://deno.land/x/zod@v3.22.4/mod.ts";

/**
 * Fetch URL and return response
 */
export class FetchHandler extends PforgeHandler<
  { url: string; method?: string },
  { status: number; body: string; headers: Record<string, string> }
> {
  constructor() {
    super({
      name: "fetch",
      description: "Fetch HTTP URL",
      timeout: 30000,
    });

    this.inputSchema = z.object({
      url: z.string().url(),
      method: z.enum(["GET", "POST", "PUT", "DELETE"]).optional().default("GET"),
    });

    this.outputSchema = z.object({
      status: z.number(),
      body: z.string(),
      headers: z.record(z.string()),
    });
  }

  async handle(
    params: { url: string; method?: string },
    _ctx: HandlerContext
  ) {
    const response = await fetch(params.url, {
      method: params.method ?? "GET",
    });

    const body = await response.text();

    const headers: Record<string, string> = {};
    response.headers.forEach((value, key) => {
      headers[key] = value;
    });

    return {
      status: response.status,
      body,
      headers,
    };
  }
}
```

---

## Success Metrics

### Technical Metrics
- [ ] All tests pass (100% pass rate)
- [ ] Test coverage ≥ 80% (line coverage)
- [ ] Mutation kill rate ≥ 90%
- [ ] FFI overhead < 500ns
- [ ] Type check passes with strict mode
- [ ] Zero SATD comments (except phase markers)
- [ ] Complexity ≤ 20 per function
- [ ] Zero `any` types in production code

### Quality Metrics
- [ ] All property tests pass (1000+ iterations)
- [ ] Fuzz tests complete without crashes
- [ ] Memory leak free (1M+ operations)
- [ ] Benchmark suite comprehensive
- [ ] All error paths tested
- [ ] All public APIs documented
- [ ] Examples work end-to-end

### Integration Metrics
- [ ] Works with existing pforge runtime
- [ ] Compatible with Python/Go bridges
- [ ] YAML configuration supported
- [ ] Error messages actionable
- [ ] Deno LSP support works
- [ ] CI/CD pipeline green

---

## Document Version

**Version**: 1.0.0-alpha
**Status**: IMPLEMENTATION IN PROGRESS
**Last Updated**: 2025-01-20
**Author**: Pragmatic AI Labs
**Ticket**: PFORGE-DENO-1000
