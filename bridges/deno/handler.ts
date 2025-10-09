/**
 * TypeScript Handler Interface for pforge Deno Bridge
 *
 * Provides ergonomic TypeScript interface for writing MCP tool handlers.
 *
 * Quality Standards:
 * - Zero SATD comments
 * - Complexity ≤ 20 per function
 * - Type-safe parameter validation
 * - Memory-safe by design
 */

import type { Schema } from "./schema.ts";
import { validate } from "./schema.ts";

/**
 * Handler context passed to all handler executions
 *
 * Contains:
 * - Request metadata
 * - Logging utilities
 * - State access (future)
 */
export interface HandlerContext {
  /** Handler name being executed */
  handlerName: string;

  /** Request timestamp */
  timestamp: Date;

  /** Logger instance */
  log: Logger;
}

/**
 * Logger interface for handler logging
 */
export interface Logger {
  debug(message: string, ...args: unknown[]): void;
  info(message: string, ...args: unknown[]): void;
  warn(message: string, ...args: unknown[]): void;
  error(message: string, ...args: unknown[]): void;
}

/**
 * Handler execution result
 *
 * Success: { success: true, data: T }
 * Error: { success: false, error: string }
 */
export type HandlerResult<T = unknown> =
  | { success: true; data: T }
  | { success: false; error: string };

/**
 * Handler function signature
 *
 * Takes typed input and context, returns typed result.
 */
export type HandlerFn<TInput = unknown, TOutput = unknown> = (
  input: TInput,
  context: HandlerContext,
) => Promise<HandlerResult<TOutput>> | HandlerResult<TOutput>;

/**
 * Handler definition
 *
 * Describes a single MCP tool handler.
 */
export interface HandlerDef<TInput = unknown, TOutput = unknown> {
  /** Handler name (must be unique) */
  name: string;

  /** Handler description */
  description: string;

  /** Handler function */
  handler: HandlerFn<TInput, TOutput>;

  /** Input schema for validation (optional) */
  inputSchema?: Schema;

  /** Timeout in milliseconds (default: 30000) */
  timeoutMs?: number;
}

/**
 * Handler Registry
 *
 * Manages handler registration and lookup.
 */
export class HandlerRegistry {
  private handlers = new Map<string, HandlerDef>();

  /**
   * Register a handler
   *
   * @param handler Handler definition
   * @throws Error if handler with same name already exists
   */
  register<TInput = unknown, TOutput = unknown>(
    handler: HandlerDef<TInput, TOutput>,
  ): void {
    if (this.handlers.has(handler.name)) {
      throw new Error(`Handler '${handler.name}' is already registered`);
    }

    this.handlers.set(handler.name, handler as HandlerDef);
  }

  /**
   * Get handler by name
   *
   * @param name Handler name
   * @returns Handler definition or undefined
   */
  get(name: string): HandlerDef | undefined {
    return this.handlers.get(name);
  }

  /**
   * Check if handler exists
   *
   * @param name Handler name
   * @returns True if handler is registered
   */
  has(name: string): boolean {
    return this.handlers.has(name);
  }

  /**
   * Get all registered handler names
   *
   * @returns Array of handler names
   */
  list(): string[] {
    return Array.from(this.handlers.keys());
  }

  /**
   * Get count of registered handlers
   *
   * @returns Number of handlers
   */
  count(): number {
    return this.handlers.size;
  }

  /**
   * Clear all handlers (for testing)
   */
  clear(): void {
    this.handlers.clear();
  }

  /**
   * Execute a handler by name
   *
   * @param name Handler name
   * @param input Handler input
   * @param context Handler context
   * @returns Handler result
   * @throws Error if handler not found
   */
  async execute<TOutput = unknown>(
    name: string,
    input: unknown,
    context: HandlerContext,
  ): Promise<HandlerResult<TOutput>> {
    const handler = this.handlers.get(name);

    if (!handler) {
      return {
        success: false,
        error: `Handler '${name}' not found`,
      };
    }

    // Validate input if schema is provided
    if (handler.inputSchema) {
      const validation = validate(input, handler.inputSchema);
      if (!validation.valid) {
        return {
          success: false,
          error: `Validation failed: ${validation.errors.join(", ")}`,
        };
      }
    }

    // Execute with timeout
    const timeoutMs = handler.timeoutMs ?? 30000;
    let timeoutId: number | undefined;

    try {
      const timeoutPromise = new Promise<HandlerResult<TOutput>>(
        (_, reject) => {
          timeoutId = setTimeout(
            () => reject(new Error("Handler timeout")),
            timeoutMs,
          );
        },
      );

      const handlerPromise = Promise.resolve(handler.handler(input, context));

      const result = await Promise.race([
        handlerPromise,
        timeoutPromise,
      ]) as HandlerResult<TOutput>;

      return result;
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      return {
        success: false,
        error: `Handler execution failed: ${errorMsg}`,
      };
    } finally {
      // Always clear timeout
      if (timeoutId !== undefined) {
        clearTimeout(timeoutId);
      }
    }
  }
}

/**
 * Console logger implementation
 */
export class ConsoleLogger implements Logger {
  debug(message: string, ..._args: unknown[]): void {
    console.log(`[DEBUG] ${message}`);
  }

  info(message: string, ..._args: unknown[]): void {
    console.log(`[INFO] ${message}`);
  }

  warn(message: string, ..._args: unknown[]): void {
    console.warn(`[WARN] ${message}`);
  }

  error(message: string, ..._args: unknown[]): void {
    console.error(`[ERROR] ${message}`);
  }
}

/**
 * Create handler context
 *
 * @param handlerName Handler name
 * @returns Handler context
 */
export function createContext(handlerName: string): HandlerContext {
  return {
    handlerName,
    timestamp: new Date(),
    log: new ConsoleLogger(),
  };
}
