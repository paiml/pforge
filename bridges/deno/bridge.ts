/**
 * Handler-FFI Bridge Integration
 *
 * Connects TypeScript handler system to FFI bridge.
 *
 * Quality Standards:
 * - Zero SATD comments
 * - Complexity ≤ 20 per function
 * - Type-safe JSON serialization
 * - Memory-safe by design
 */

import { FfiBridge } from "./ffi.ts";
import { createContext, type HandlerDef, HandlerRegistry } from "./handler.ts";

/**
 * Integrated bridge combining FFI and handler system
 *
 * Provides:
 * - Handler registration
 * - Automatic FFI routing
 * - JSON serialization/deserialization
 * - Error handling
 */
export class PforgeBridge {
  private ffi: FfiBridge;
  private registry: HandlerRegistry;

  constructor() {
    this.ffi = new FfiBridge();
    this.registry = new HandlerRegistry();
  }

  /**
   * Register a handler
   *
   * @param handler Handler definition
   */
  register<TInput = unknown, TOutput = unknown>(
    handler: HandlerDef<TInput, TOutput>,
  ): void {
    this.registry.register(handler);
  }

  /**
   * Execute handler by name
   *
   * Routes through TypeScript registry if handler exists locally,
   * otherwise delegates to FFI bridge.
   *
   * @param name Handler name
   * @param input Handler input (object or JSON string)
   * @returns Handler result
   */
  async execute<TOutput = unknown>(
    name: string,
    input: unknown,
  ): Promise<
    { success: true; data: TOutput } | { success: false; error: string }
  > {
    // Check if handler is registered locally
    if (this.registry.has(name)) {
      // Execute through TypeScript registry
      const context = createContext(name);
      const result = await this.registry.execute<TOutput>(name, input, context);
      return result;
    }

    // Otherwise, delegate to FFI bridge (Rust handlers)
    try {
      // Serialize input to JSON
      const inputStr = typeof input === "string" ? input : JSON.stringify(input);
      const inputBytes = new TextEncoder().encode(inputStr);

      // Execute through FFI
      const resultBytes = this.ffi.executeHandler(name, inputBytes);

      // Deserialize result
      if (resultBytes.length === 0) {
        return { success: true, data: {} as TOutput };
      }

      const resultStr = new TextDecoder().decode(resultBytes);
      const data = JSON.parse(resultStr) as TOutput;

      return { success: true, data };
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      return {
        success: false,
        error: `Handler execution failed: ${errorMsg}`,
      };
    }
  }

  /**
   * Get pforge version
   *
   * @returns Version string
   */
  version(): string {
    return this.ffi.version();
  }

  /**
   * List all registered handler names
   *
   * @returns Array of handler names
   */
  list(): string[] {
    return this.registry.list();
  }

  /**
   * Check if handler exists
   *
   * @param name Handler name
   * @returns True if handler is registered
   */
  has(name: string): boolean {
    return this.registry.has(name);
  }

  /**
   * Get count of registered handlers
   *
   * @returns Number of handlers
   */
  count(): number {
    return this.registry.count();
  }

  /**
   * Close bridge
   *
   * Cleans up FFI resources.
   */
  close(): void {
    this.ffi.close();
  }
}
