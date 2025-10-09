/**
 * Example Server Tests (TDD Cycle 8)
 *
 * Tests for example MCP server functionality.
 *
 * Quality Standards:
 * - Zero SATD comments
 * - Complexity ≤ 20
 * - Demonstrates real-world usage
 */

import { assertEquals } from "https://deno.land/std@0.208.0/assert/mod.ts";
import { PforgeBridge } from "../../bridge.ts";

/**
 * Test: Example server - greet tool
 */
Deno.test("Example: greet tool", async () => {
  const bridge = new PforgeBridge();

  bridge.register({
    name: "greet",
    description: "Greet user",
    handler: (input: { name: string }) => {
      if (!input.name || input.name.trim().length === 0) {
        return { success: false, error: "Name is required" };
      }
      return {
        success: true,
        data: { message: `Hello, ${input.name}! 👋` },
      };
    },
  });

  const result = await bridge.execute("greet", { name: "Alice" });

  assertEquals(result.success, true);
  if (result.success) {
    const data = result.data as { message: string };
    assertEquals(
      data.message.includes("Alice"),
      true,
    );
  }

  bridge.close();
});

/**
 * Test: Example server - add tool
 */
Deno.test("Example: add tool", async () => {
  const bridge = new PforgeBridge();

  bridge.register({
    name: "add",
    description: "Add numbers",
    handler: (input: { a: number; b: number }) => ({
      success: true,
      data: { sum: input.a + input.b },
    }),
  });

  const result = await bridge.execute("add", { a: 5, b: 3 });

  assertEquals(result.success, true);
  if (result.success) {
    const data = result.data as { sum: number };
    assertEquals(data.sum, 8);
  }

  bridge.close();
});

/**
 * Test: Example server - async weather tool
 */
Deno.test("Example: async weather tool", async () => {
  const bridge = new PforgeBridge();

  bridge.register({
    name: "weather",
    description: "Get weather",
    handler: async (input: { city: string }) => {
      await new Promise((resolve) => setTimeout(resolve, 10));
      return {
        success: true,
        data: {
          city: input.city,
          temperature: 72,
          condition: "Sunny",
        },
      };
    },
  });

  const result = await bridge.execute("weather", { city: "SF" });

  assertEquals(result.success, true);
  if (result.success) {
    const data = result.data as { city: string; temperature: number };
    assertEquals(data.city, "SF");
    assertEquals(data.temperature, 72);
  }

  bridge.close();
});

/**
 * Test: Example server - factorial tool
 */
Deno.test("Example: factorial tool", async () => {
  const bridge = new PforgeBridge();

  bridge.register({
    name: "factorial",
    description: "Calculate factorial",
    handler: (input: { n: number }) => {
      if (input.n < 0) {
        return { success: false, error: "Must be non-negative" };
      }

      let result = 1;
      for (let i = 2; i <= input.n; i++) {
        result *= i;
      }

      return { success: true, data: { factorial: result } };
    },
  });

  const result = await bridge.execute("factorial", { n: 5 });

  assertEquals(result.success, true);
  if (result.success) {
    const data = result.data as { factorial: number };
    assertEquals(data.factorial, 120); // 5! = 120
  }

  bridge.close();
});

/**
 * Test: Example server - error handling
 */
Deno.test("Example: error handling in tools", async () => {
  const bridge = new PforgeBridge();

  bridge.register({
    name: "validate",
    description: "Validate input",
    handler: (input: { value: number }) => {
      if (input.value < 0) {
        return { success: false, error: "Value must be positive" };
      }
      return { success: true, data: { valid: true } };
    },
  });

  // Valid input
  const result1 = await bridge.execute("validate", { value: 10 });
  assertEquals(result1.success, true);

  // Invalid input
  const result2 = await bridge.execute("validate", { value: -5 });
  assertEquals(result2.success, false);

  bridge.close();
});

/**
 * Test: Example server - multiple tools
 */
Deno.test("Example: multiple tools registered", () => {
  const bridge = new PforgeBridge();

  bridge.register({
    name: "tool1",
    description: "Tool 1",
    handler: () => ({ success: true, data: { id: 1 } }),
  });

  bridge.register({
    name: "tool2",
    description: "Tool 2",
    handler: () => ({ success: true, data: { id: 2 } }),
  });

  bridge.register({
    name: "tool3",
    description: "Tool 3",
    handler: () => ({ success: true, data: { id: 3 } }),
  });

  assertEquals(bridge.count(), 3);

  const handlers = bridge.list();
  assertEquals(handlers.length, 3);
  assertEquals(handlers.includes("tool1"), true);
  assertEquals(handlers.includes("tool2"), true);
  assertEquals(handlers.includes("tool3"), true);

  bridge.close();
});
