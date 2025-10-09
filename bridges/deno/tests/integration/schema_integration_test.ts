/**
 * Schema Integration Tests (TDD Cycle 10)
 *
 * Tests for schema validation integrated with handlers.
 *
 * Quality Standards:
 * - Zero SATD comments
 * - Complexity ≤ 20
 * - End-to-end schema validation
 */

import { assertEquals } from "https://deno.land/std@0.208.0/assert/mod.ts";
import { PforgeBridge } from "../../bridge.ts";
import { SchemaBuilder } from "../../schema.ts";

/**
 * Test: Handler with schema validation - valid input
 */
Deno.test("Schema Integration: Valid input passes validation", async () => {
  const bridge = new PforgeBridge();

  bridge.register({
    name: "create_user",
    description: "Create a user",
    handler: (input: { name: string; age: number }) => ({
      success: true,
      data: { id: 123, name: input.name, age: input.age },
    }),
    inputSchema: SchemaBuilder.object({
      name: SchemaBuilder.string({ minLength: 1 }),
      age: SchemaBuilder.number({ min: 0, max: 120 }),
    }, ["name", "age"]),
  });

  const result = await bridge.execute("create_user", {
    name: "Alice",
    age: 30,
  });

  assertEquals(result.success, true);
  if (result.success) {
    const data = result.data as { id: number; name: string; age: number };
    assertEquals(data.name, "Alice");
    assertEquals(data.age, 30);
  }

  bridge.close();
});

/**
 * Test: Handler with schema validation - invalid input
 */
Deno.test("Schema Integration: Invalid input fails validation", async () => {
  const bridge = new PforgeBridge();

  bridge.register({
    name: "create_user",
    description: "Create a user",
    handler: () => ({
      success: true,
      data: { id: 123 },
    }),
    inputSchema: SchemaBuilder.object({
      name: SchemaBuilder.string({ minLength: 1 }),
      age: SchemaBuilder.number({ min: 0, max: 120 }),
    }, ["name", "age"]),
  });

  // Missing required field
  const result = await bridge.execute("create_user", {
    name: "Alice",
  });

  assertEquals(result.success, false);
  if (!result.success) {
    assertEquals(result.error.includes("Validation failed"), true);
    assertEquals(result.error.includes("age"), true);
  }

  bridge.close();
});

/**
 * Test: Handler without schema - no validation
 */
Deno.test("Schema Integration: Handler without schema skips validation", async () => {
  const bridge = new PforgeBridge();

  bridge.register({
    name: "flexible_handler",
    description: "Flexible handler",
    handler: (input: unknown) => ({
      success: true,
      data: { received: input },
    }),
    // No schema provided
  });

  // Any input should work
  const result = await bridge.execute("flexible_handler", {
    anything: "goes",
    number: 42,
  });

  assertEquals(result.success, true);

  bridge.close();
});

/**
 * Test: Schema validation with string constraints
 */
Deno.test("Schema Integration: String length validation", async () => {
  const bridge = new PforgeBridge();

  bridge.register({
    name: "set_username",
    description: "Set username",
    handler: (input: { username: string }) => ({
      success: true,
      data: { username: input.username },
    }),
    inputSchema: SchemaBuilder.object({
      username: SchemaBuilder.string({ minLength: 3, maxLength: 20 }),
    }, ["username"]),
  });

  // Too short
  const result1 = await bridge.execute("set_username", { username: "ab" });
  assertEquals(result1.success, false);

  // Valid
  const result2 = await bridge.execute("set_username", { username: "alice" });
  assertEquals(result2.success, true);

  bridge.close();
});

/**
 * Test: Schema validation with number constraints
 */
Deno.test("Schema Integration: Number range validation", async () => {
  const bridge = new PforgeBridge();

  bridge.register({
    name: "set_age",
    description: "Set age",
    handler: (input: { age: number }) => ({
      success: true,
      data: { age: input.age },
    }),
    inputSchema: SchemaBuilder.object({
      age: SchemaBuilder.number({ min: 0, max: 120 }),
    }, ["age"]),
  });

  // Too low
  const result1 = await bridge.execute("set_age", { age: -1 });
  assertEquals(result1.success, false);

  // Valid
  const result2 = await bridge.execute("set_age", { age: 30 });
  assertEquals(result2.success, true);

  bridge.close();
});

/**
 * Test: Multiple validation errors
 */
Deno.test("Schema Integration: Multiple validation errors", async () => {
  const bridge = new PforgeBridge();

  bridge.register({
    name: "create_profile",
    description: "Create profile",
    handler: () => ({
      success: true,
      data: {},
    }),
    inputSchema: SchemaBuilder.object({
      name: SchemaBuilder.string(),
      email: SchemaBuilder.string(),
      age: SchemaBuilder.number(),
    }, ["name", "email", "age"]),
  });

  // Missing multiple fields
  const result = await bridge.execute("create_profile", {
    name: "Alice",
  });

  assertEquals(result.success, false);
  if (!result.success) {
    assertEquals(result.error.includes("email"), true);
    assertEquals(result.error.includes("age"), true);
  }

  bridge.close();
});
