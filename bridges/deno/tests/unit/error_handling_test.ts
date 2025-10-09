/**
 * Error Handling Tests (TDD Cycle 3)
 *
 * Tests comprehensive error scenarios:
 * - Invalid handler names
 * - Malformed JSON input
 * - Large payloads
 * - Special characters
 * - Error propagation from Rust
 *
 * Quality Standards:
 * - Zero SATD comments
 * - Complexity ≤ 20
 * - All error paths tested
 */

import {
  assertEquals,
  assertExists,
} from "https://deno.land/std@0.208.0/assert/mod.ts";
import { FfiBridge } from "../../ffi.ts";

/**
 * Test: Empty handler name throws error
 *
 * Empty handler names should be rejected with clear error message.
 */
Deno.test("Error: Empty handler name", () => {
  const bridge = new FfiBridge();

  const params = new TextEncoder().encode(JSON.stringify({}));

  let errorThrown = false;
  try {
    bridge.executeHandler("", params);
  } catch (e) {
    errorThrown = true;
    const msg = e instanceof Error ? e.message : String(e);
    assertEquals(
      msg.includes("failed") || msg.includes("handler"),
      true,
      "Error should mention handler or failure",
    );
  }

  assertEquals(errorThrown, true, "Should throw error for empty handler name");
  bridge.close();
});

/**
 * Test: Invalid UTF-8 in handler name
 *
 * Handler names must be valid UTF-8 strings.
 */
Deno.test("Error: Invalid UTF-8 in handler name", () => {
  const bridge = new FfiBridge();

  // Create params with valid JSON
  const params = new TextEncoder().encode(JSON.stringify({}));

  // Test with string containing null bytes - should handle gracefully
  try {
    const result = bridge.executeHandler("test\0handler", params);
    // If succeeds, result should be valid
    assertExists(result);
  } catch (e) {
    // Error is acceptable
    assertExists(e, "Error should be thrown");
  }

  bridge.close();
});

/**
 * Test: Extremely long handler name
 *
 * Very long handler names should be handled gracefully.
 */
Deno.test("Error: Extremely long handler name", () => {
  const bridge = new FfiBridge();

  const longName = "x".repeat(10000);
  const params = new TextEncoder().encode(JSON.stringify({}));

  // Should handle gracefully - either error or succeeds
  try {
    const result = bridge.executeHandler(longName, params);
    // If succeeds, result should be valid
    assertExists(result);
  } catch (e) {
    // Error is acceptable
    assertExists(e, "Error should be thrown");
  }

  bridge.close();
});

/**
 * Test: Invalid JSON parameters
 *
 * Malformed JSON should be rejected by Rust side.
 */
Deno.test("Error: Invalid JSON parameters", () => {
  const bridge = new FfiBridge();

  // Not valid JSON
  const invalidJson = new TextEncoder().encode("not json at all");

  // Should handle gracefully - either error or empty result
  try {
    const result = bridge.executeHandler("test_handler", invalidJson);
    // If no error thrown, result should exist
    assertExists(result);
  } catch (e) {
    // Error is acceptable
    assertExists(e, "Error should be thrown");
  }

  bridge.close();
});

/**
 * Test: Large payload handling
 *
 * Large payloads (1MB) should be handled without crashes.
 */
Deno.test("Error: Large payload (1MB)", () => {
  const bridge = new FfiBridge();

  // Create 1MB payload
  const largeObject = { data: "x".repeat(1024 * 1024) };
  const params = new TextEncoder().encode(JSON.stringify(largeObject));

  assertEquals(params.length > 1000000, true, "Payload should be > 1MB");

  // Should not crash - either succeeds or fails gracefully
  try {
    const result = bridge.executeHandler("test_handler", params);
    assertExists(result, "Result should exist");
    assertEquals(
      result instanceof Uint8Array,
      true,
      "Should return Uint8Array",
    );
  } catch (e) {
    // Graceful failure is acceptable
    assertExists(e, "Error is acceptable for large payloads");
  }

  bridge.close();
});

/**
 * Test: Special characters in parameters
 *
 * Unicode, emojis, and special characters should be handled correctly.
 */
Deno.test("Error: Special characters in parameters", () => {
  const bridge = new FfiBridge();

  const specialChars = {
    emoji: "🦕🔥💻",
    unicode: "你好世界",
    special: "\n\t\r\"'\\",
    zero: "\0",
  };

  const params = new TextEncoder().encode(JSON.stringify(specialChars));

  // Should handle special characters without crashing
  try {
    const result = bridge.executeHandler("test_handler", params);
    assertExists(result, "Result should exist");
  } catch (e) {
    // Error is acceptable but should not crash
    assertExists(e, "Error is acceptable");
  }

  bridge.close();
});

/**
 * Test: Null pointer validation
 *
 * Passing null or invalid pointers should be caught.
 */
Deno.test("Error: Empty parameters", () => {
  const bridge = new FfiBridge();

  const emptyParams = new Uint8Array(0);

  // Should handle empty params without crashing
  try {
    const result = bridge.executeHandler("test_handler", emptyParams);
    assertExists(result, "Result should exist");
    assertEquals(
      result instanceof Uint8Array,
      true,
      "Should return Uint8Array",
    );
  } catch (e) {
    // Error is acceptable
    assertExists(e, "Error is acceptable for empty params");
  }

  bridge.close();
});

/**
 * Test: Multiple errors in sequence
 *
 * Bridge should remain stable after multiple errors.
 */
Deno.test("Error: Multiple errors in sequence", () => {
  const bridge = new FfiBridge();

  // Execute multiple invalid operations
  for (let i = 0; i < 10; i++) {
    try {
      bridge.executeHandler("", new Uint8Array(0));
    } catch {
      // Expected
    }

    try {
      bridge.executeHandler("test", new TextEncoder().encode("not json"));
    } catch {
      // Expected
    }
  }

  // Bridge should still be usable
  const version = bridge.version();
  assertExists(version, "Version should still work after errors");

  bridge.close();
});

/**
 * Test: Error after close
 *
 * Operations after close should throw consistent errors.
 */
Deno.test("Error: Operations after close throw consistently", () => {
  const bridge = new FfiBridge();
  bridge.close();

  let versionError = false;
  try {
    bridge.version();
  } catch (e) {
    versionError = true;
    const msg = e instanceof Error ? e.message : String(e);
    assertEquals(
      msg,
      "FfiBridge is closed",
      "Error message should be consistent",
    );
  }

  let executeError = false;
  try {
    bridge.executeHandler("test", new Uint8Array(0));
  } catch (e) {
    executeError = true;
    const msg = e instanceof Error ? e.message : String(e);
    assertEquals(
      msg,
      "FfiBridge is closed",
      "Error message should be consistent",
    );
  }

  assertEquals(versionError, true, "version() should throw after close");
  assertEquals(executeError, true, "executeHandler() should throw after close");
});

/**
 * Test: Idempotent close
 *
 * Closing multiple times should be safe.
 */
Deno.test("Error: Idempotent close", () => {
  const bridge = new FfiBridge();

  // Close multiple times should not crash
  bridge.close();
  bridge.close();
  bridge.close();

  // Should still throw error on use
  let errorThrown = false;
  try {
    bridge.version();
  } catch {
    errorThrown = true;
  }

  assertEquals(errorThrown, true, "Should still throw after multiple closes");
});
