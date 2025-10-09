/**
 * Example MCP Server using pforge Deno Bridge
 *
 * Demonstrates:
 * - Handler registration
 * - TypeScript tool implementation
 * - Error handling
 * - Async operations
 *
 * Usage:
 *   deno run --unstable-ffi --allow-ffi --allow-env --allow-read examples/hello_server.ts
 */

import { PforgeBridge } from "../bridge.ts";

/**
 * Main function - setup and run server
 */
async function main() {
  const bridge = new PforgeBridge();

  console.log("🚀 pforge Deno Bridge Example Server");
  console.log(`📦 Version: ${bridge.version()}`);
  console.log();

  // Register tool: greet
  bridge.register({
    name: "greet",
    description: "Greet a user by name",
    handler: (input: { name: string }) => {
      if (!input.name || input.name.trim().length === 0) {
        return {
          success: false,
          error: "Name is required",
        };
      }

      return {
        success: true,
        data: {
          message: `Hello, ${input.name}! 👋`,
          timestamp: new Date().toISOString(),
        },
      };
    },
  });

  // Register tool: add
  bridge.register({
    name: "add",
    description: "Add two numbers",
    handler: (input: { a: number; b: number }) => {
      if (typeof input.a !== "number" || typeof input.b !== "number") {
        return {
          success: false,
          error: "Both a and b must be numbers",
        };
      }

      return {
        success: true,
        data: {
          sum: input.a + input.b,
          operation: `${input.a} + ${input.b} = ${input.a + input.b}`,
        },
      };
    },
  });

  // Register tool: fetch_weather (async example)
  bridge.register({
    name: "fetch_weather",
    description: "Fetch weather data (simulated)",
    handler: async (input: { city: string }) => {
      // Simulate API call delay
      await new Promise((resolve) => setTimeout(resolve, 100));

      // Simulated weather data
      const weather = {
        city: input.city,
        temperature: Math.floor(Math.random() * 30) + 10,
        condition: ["Sunny", "Cloudy", "Rainy"][
          Math.floor(Math.random() * 3)
        ],
        humidity: Math.floor(Math.random() * 50) + 30,
      };

      return {
        success: true,
        data: weather,
      };
    },
    timeoutMs: 5000,
  });

  // Register tool: factorial
  bridge.register({
    name: "factorial",
    description: "Calculate factorial of a number",
    handler: (input: { n: number }) => {
      if (typeof input.n !== "number" || input.n < 0) {
        return {
          success: false,
          error: "Input must be a non-negative number",
        };
      }

      if (input.n > 170) {
        return {
          success: false,
          error: "Number too large (max 170)",
        };
      }

      let result = 1;
      for (let i = 2; i <= input.n; i++) {
        result *= i;
      }

      return {
        success: true,
        data: {
          n: input.n,
          factorial: result,
          formula: `${input.n}!`,
        },
      };
    },
  });

  console.log("✅ Registered tools:");
  for (const name of bridge.list()) {
    console.log(`   - ${name}`);
  }
  console.log();

  // Demo: Execute each tool
  console.log("🔧 Running tool demos:\n");

  // Demo 1: greet
  console.log("1. greet({ name: 'Alice' })");
  const result1 = await bridge.execute("greet", { name: "Alice" });
  console.log("   Result:", JSON.stringify(result1, null, 2));
  console.log();

  // Demo 2: add
  console.log("2. add({ a: 5, b: 3 })");
  const result2 = await bridge.execute("add", { a: 5, b: 3 });
  console.log("   Result:", JSON.stringify(result2, null, 2));
  console.log();

  // Demo 3: fetch_weather
  console.log("3. fetch_weather({ city: 'San Francisco' })");
  const result3 = await bridge.execute("fetch_weather", {
    city: "San Francisco",
  });
  console.log("   Result:", JSON.stringify(result3, null, 2));
  console.log();

  // Demo 4: factorial
  console.log("4. factorial({ n: 10 })");
  const result4 = await bridge.execute("factorial", { n: 10 });
  console.log("   Result:", JSON.stringify(result4, null, 2));
  console.log();

  // Demo 5: Error handling - invalid input
  console.log("5. greet({ name: '' }) - Error handling");
  const result5 = await bridge.execute("greet", { name: "" });
  console.log("   Result:", JSON.stringify(result5, null, 2));
  console.log();

  // Demo 6: Error handling - nonexistent tool
  console.log("6. nonexistent() - Nonexistent tool");
  const result6 = await bridge.execute("nonexistent", {});
  console.log("   Result:", JSON.stringify(result6, null, 2));
  console.log();

  // Cleanup
  bridge.close();
  console.log("✅ Server shutdown complete");
}

// Run main
if (import.meta.main) {
  main().catch((error) => {
    console.error("❌ Error:", error);
    Deno.exit(1);
  });
}
