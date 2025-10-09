/**
 * Schema Validation Tests (TDD Cycle 10)
 *
 * Tests for JSON schema validation.
 *
 * Quality Standards:
 * - Zero SATD comments
 * - Complexity ≤ 20
 * - Comprehensive validation coverage
 */

import { assertEquals } from "https://deno.land/std@0.208.0/assert/mod.ts";
import { SchemaBuilder, validate } from "../../schema.ts";

/**
 * Test: Validate simple object
 */
Deno.test("Schema: Validate simple object", () => {
  const schema = SchemaBuilder.object({
    name: SchemaBuilder.string(),
    age: SchemaBuilder.number(),
  }, ["name", "age"]);

  const result = validate({ name: "Alice", age: 30 }, schema);

  assertEquals(result.valid, true);
});

/**
 * Test: Missing required field
 */
Deno.test("Schema: Missing required field", () => {
  const schema = SchemaBuilder.object({
    name: SchemaBuilder.string(),
    age: SchemaBuilder.number(),
  }, ["name", "age"]);

  const result = validate({ name: "Alice" }, schema);

  assertEquals(result.valid, false);
  if (!result.valid) {
    assertEquals(result.errors.length > 0, true);
    assertEquals(
      result.errors.some((e) => e.includes("age")),
      true,
    );
  }
});

/**
 * Test: Wrong type for field
 */
Deno.test("Schema: Wrong type for field", () => {
  const schema = SchemaBuilder.object({
    name: SchemaBuilder.string(),
    age: SchemaBuilder.number(),
  });

  const result = validate({ name: "Alice", age: "thirty" }, schema);

  assertEquals(result.valid, false);
  if (!result.valid) {
    assertEquals(
      result.errors.some((e) => e.includes("number")),
      true,
    );
  }
});

/**
 * Test: String length validation
 */
Deno.test("Schema: String length validation", () => {
  const schema = SchemaBuilder.object({
    username: SchemaBuilder.string({ minLength: 3, maxLength: 20 }),
  });

  // Too short
  const result1 = validate({ username: "ab" }, schema);
  assertEquals(result1.valid, false);

  // Too long
  const result2 = validate({ username: "a".repeat(25) }, schema);
  assertEquals(result2.valid, false);

  // Valid
  const result3 = validate({ username: "alice" }, schema);
  assertEquals(result3.valid, true);
});

/**
 * Test: Number range validation
 */
Deno.test("Schema: Number range validation", () => {
  const schema = SchemaBuilder.object({
    age: SchemaBuilder.number({ min: 0, max: 120 }),
  });

  // Too low
  const result1 = validate({ age: -1 }, schema);
  assertEquals(result1.valid, false);

  // Too high
  const result2 = validate({ age: 150 }, schema);
  assertEquals(result2.valid, false);

  // Valid
  const result3 = validate({ age: 30 }, schema);
  assertEquals(result3.valid, true);
});

/**
 * Test: Boolean validation
 */
Deno.test("Schema: Boolean validation", () => {
  const schema = SchemaBuilder.object({
    active: SchemaBuilder.boolean(),
  });

  // Valid boolean
  const result1 = validate({ active: true }, schema);
  assertEquals(result1.valid, true);

  // Invalid type
  const result2 = validate({ active: "true" }, schema);
  assertEquals(result2.valid, false);
});

/**
 * Test: Array validation
 */
Deno.test("Schema: Array validation", () => {
  const schema = SchemaBuilder.object({
    tags: SchemaBuilder.array(),
  });

  // Valid array
  const result1 = validate({ tags: ["a", "b", "c"] }, schema);
  assertEquals(result1.valid, true);

  // Invalid type
  const result2 = validate({ tags: "not an array" }, schema);
  assertEquals(result2.valid, false);
});

/**
 * Test: Optional fields
 */
Deno.test("Schema: Optional fields", () => {
  const schema = SchemaBuilder.object({
    name: SchemaBuilder.string(),
    nickname: SchemaBuilder.string(),
  }, ["name"]); // Only name is required

  // Without optional field
  const result1 = validate({ name: "Alice" }, schema);
  assertEquals(result1.valid, true);

  // With optional field
  const result2 = validate({ name: "Alice", nickname: "Ally" }, schema);
  assertEquals(result2.valid, true);
});

/**
 * Test: Input is not an object
 */
Deno.test("Schema: Input is not an object", () => {
  const schema = SchemaBuilder.object({
    name: SchemaBuilder.string(),
  });

  const result1 = validate("not an object", schema);
  assertEquals(result1.valid, false);

  const result2 = validate(123, schema);
  assertEquals(result2.valid, false);

  const result3 = validate(null, schema);
  assertEquals(result3.valid, false);

  const result4 = validate([], schema);
  assertEquals(result4.valid, false);
});

/**
 * Test: Complex nested validation
 */
Deno.test("Schema: Complex nested object", () => {
  const schema = SchemaBuilder.object({
    name: SchemaBuilder.string({ minLength: 1 }),
    age: SchemaBuilder.number({ min: 0, max: 120 }),
    active: SchemaBuilder.boolean(),
    tags: SchemaBuilder.array(),
  }, ["name", "age"]);

  // Valid complex object
  const result = validate({
    name: "Alice",
    age: 30,
    active: true,
    tags: ["developer", "engineer"],
  }, schema);

  assertEquals(result.valid, true);
});

/**
 * Test: Multiple validation errors
 */
Deno.test("Schema: Multiple validation errors", () => {
  const schema = SchemaBuilder.object({
    name: SchemaBuilder.string(),
    age: SchemaBuilder.number(),
    email: SchemaBuilder.string(),
  }, ["name", "age", "email"]);

  // Missing multiple fields
  const result = validate({ name: "Alice" }, schema);

  assertEquals(result.valid, false);
  if (!result.valid) {
    assertEquals(result.errors.length >= 2, true);
  }
});
