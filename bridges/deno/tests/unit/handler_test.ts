/**
 * Handler System Tests (TDD Cycle 6)
 *
 * Tests for TypeScript handler interface and registry.
 *
 * Quality Standards:
 * - Zero SATD comments
 * - Complexity ≤ 20
 * - Type-safe handler operations
 */

import {
  assertEquals,
  assertExists,
} from "https://deno.land/std@0.208.0/assert/mod.ts";
import {
  createContext,
  type HandlerDef,
  HandlerRegistry,
} from "../../handler.ts";

/**
 * Test: Register and get handler
 */
Deno.test("Handler: Register and get handler", () => {
  const registry = new HandlerRegistry();

  const handler: HandlerDef = {
    name: "test_handler",
    description: "Test handler",
    handler: (input) => ({
      success: true,
      data: { echo: input },
    }),
  };

  registry.register(handler);

  const retrieved = registry.get("test_handler");
  assertExists(retrieved);
  assertEquals(retrieved.name, "test_handler");
  assertEquals(retrieved.description, "Test handler");
});

/**
 * Test: Cannot register duplicate handler
 */
Deno.test("Handler: Cannot register duplicate handler", () => {
  const registry = new HandlerRegistry();

  const handler: HandlerDef = {
    name: "duplicate",
    description: "First",
    handler: () => ({ success: true, data: {} }),
  };

  registry.register(handler);

  // Try to register again
  let errorThrown = false;
  try {
    registry.register({
      name: "duplicate",
      description: "Second",
      handler: () => ({ success: true, data: {} }),
    });
  } catch (e) {
    errorThrown = true;
    const msg = e instanceof Error ? e.message : String(e);
    assertEquals(
      msg.includes("already registered"),
      true,
      "Error should mention already registered",
    );
  }

  assertEquals(errorThrown, true, "Should throw on duplicate registration");
});

/**
 * Test: List handlers
 */
Deno.test("Handler: List all handlers", () => {
  const registry = new HandlerRegistry();

  registry.register({
    name: "handler1",
    description: "First",
    handler: () => ({ success: true, data: {} }),
  });

  registry.register({
    name: "handler2",
    description: "Second",
    handler: () => ({ success: true, data: {} }),
  });

  const names = registry.list();
  assertEquals(names.length, 2);
  assertEquals(names.includes("handler1"), true);
  assertEquals(names.includes("handler2"), true);
});

/**
 * Test: Count handlers
 */
Deno.test("Handler: Count handlers", () => {
  const registry = new HandlerRegistry();

  assertEquals(registry.count(), 0);

  registry.register({
    name: "handler1",
    description: "First",
    handler: () => ({ success: true, data: {} }),
  });

  assertEquals(registry.count(), 1);

  registry.register({
    name: "handler2",
    description: "Second",
    handler: () => ({ success: true, data: {} }),
  });

  assertEquals(registry.count(), 2);
});

/**
 * Test: Has handler
 */
Deno.test("Handler: Check if handler exists", () => {
  const registry = new HandlerRegistry();

  assertEquals(registry.has("nonexistent"), false);

  registry.register({
    name: "exists",
    description: "Exists",
    handler: () => ({ success: true, data: {} }),
  });

  assertEquals(registry.has("exists"), true);
  assertEquals(registry.has("still_nonexistent"), false);
});

/**
 * Test: Clear handlers
 */
Deno.test("Handler: Clear all handlers", () => {
  const registry = new HandlerRegistry();

  registry.register({
    name: "handler1",
    description: "First",
    handler: () => ({ success: true, data: {} }),
  });

  assertEquals(registry.count(), 1);

  registry.clear();

  assertEquals(registry.count(), 0);
  assertEquals(registry.has("handler1"), false);
});

/**
 * Test: Execute handler successfully
 */
Deno.test("Handler: Execute handler successfully", async () => {
  const registry = new HandlerRegistry();

  registry.register({
    name: "echo",
    description: "Echo input",
    handler: (input) => ({
      success: true,
      data: { echo: input },
    }),
  });

  const context = createContext("echo");
  const result = await registry.execute("echo", { message: "hello" }, context);

  assertEquals(result.success, true);
  if (result.success) {
    assertEquals(result.data, { echo: { message: "hello" } });
  }
});

/**
 * Test: Execute nonexistent handler
 */
Deno.test("Handler: Execute nonexistent handler", async () => {
  const registry = new HandlerRegistry();

  const context = createContext("nonexistent");
  const result = await registry.execute("nonexistent", {}, context);

  assertEquals(result.success, false);
  if (!result.success) {
    assertEquals(
      result.error.includes("not found"),
      true,
      "Error should mention not found",
    );
  }
});

/**
 * Test: Handler timeout
 *
 * Simplified: handler that never resolves (no setTimeout leak)
 */
Deno.test("Handler: Handler timeout", async () => {
  const registry = new HandlerRegistry();

  registry.register({
    name: "slow",
    description: "Slow handler",
    handler: () => {
      // Return promise that never resolves (will timeout)
      return new Promise(() => {
        // Never resolves or rejects
      });
    },
    timeoutMs: 50, // 50ms timeout
  });

  const context = createContext("slow");
  const result = await registry.execute("slow", {}, context);

  assertEquals(result.success, false);
  if (!result.success) {
    assertEquals(
      result.error.includes("timeout") || result.error.includes("failed"),
      true,
      "Error should mention timeout or failure",
    );
  }
});

/**
 * Test: Handler error handling
 */
Deno.test("Handler: Handler error handling", async () => {
  const registry = new HandlerRegistry();

  registry.register({
    name: "error",
    description: "Error handler",
    handler: () => {
      throw new Error("Handler error");
    },
  });

  const context = createContext("error");
  const result = await registry.execute("error", {}, context);

  assertEquals(result.success, false);
  if (!result.success) {
    assertEquals(
      result.error.includes("Handler error") || result.error.includes("failed"),
      true,
      "Error should include handler error message",
    );
  }
});

/**
 * Test: Context creation
 */
Deno.test("Handler: Context creation", () => {
  const context = createContext("test_handler");

  assertEquals(context.handlerName, "test_handler");
  assertExists(context.timestamp);
  assertExists(context.log);
  assertEquals(context.timestamp instanceof Date, true);
});

/**
 * Test: Synchronous handler
 */
Deno.test("Handler: Synchronous handler", async () => {
  const registry = new HandlerRegistry();

  registry.register({
    name: "sync",
    description: "Sync handler",
    handler: (input) => ({
      success: true,
      data: { sync: input },
    }),
  });

  const context = createContext("sync");
  const result = await registry.execute("sync", { value: 42 }, context);

  assertEquals(result.success, true);
  if (result.success) {
    assertEquals(result.data, { sync: { value: 42 } });
  }
});
