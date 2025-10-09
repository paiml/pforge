/**
 * Deno FFI Bindings for pforge_bridge
 *
 * Provides low-level FFI interface to Rust pforge_bridge library.
 *
 * Quality Standards:
 * - Zero SATD comments
 * - Complexity ≤ 20 per function
 * - Type-safe pointer operations
 * - Memory-safe by design
 */

/**
 * FFI Result structure matching Rust's FfiResult
 *
 * Memory Layout (C repr):
 * - [0] code: i32 (4 bytes, aligned)
 * - [1] data: *mut u8 (8 bytes pointer)
 * - [2] data_len: usize (8 bytes)
 * - [3] error: *const c_char (8 bytes pointer)
 *
 * Note: Deno FFI returns structs as arrays, not objects
 */
export type FfiResult = [
  number, // code: i32
  Deno.PointerValue, // data: *mut u8
  number | bigint, // data_len: usize
  Deno.PointerValue, // error: *const c_char
];

/**
 * Deno FFI symbol definitions for pforge_bridge
 *
 * Matches Rust FFI interface in crates/pforge-bridge/src/lib.rs
 */
export const SYMBOLS = {
  // Get pforge version string
  pforge_version: {
    parameters: [],
    result: "pointer" as const,
  },

  // Execute handler with JSON input/output
  pforge_execute_handler: {
    parameters: ["pointer" as const, "pointer" as const, "usize" as const],
    result: {
      struct: ["i32", "pointer", "usize", "pointer"] as const,
    },
  },

  // Free result from pforge_execute_handler
  pforge_free_result: {
    parameters: [{
      struct: ["i32", "pointer", "usize", "pointer"] as const,
    }],
    result: "void" as const,
  },
} as const;

/**
 * Get platform-specific library paths
 *
 * Returns array of paths to search for pforge_bridge library,
 * ordered by preference (release before debug, standard locations last).
 *
 * Complexity: O(1) - fixed number of paths per platform
 */
export function getLibraryPaths(): string[] {
  const os = Deno.build.os;

  // Get base directory relative to this file
  // Assumes bridges/deno/ffi.ts -> ../../target/
  const scriptDir = new URL(".", import.meta.url).pathname;
  const baseDir = `${scriptDir}../../target`;

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
        "/Library/Frameworks/libpforge_bridge.dylib",
      ];

    case "windows":
      return [
        `${baseDir}\\release\\pforge_bridge.dll`,
        `${baseDir}\\debug\\pforge_bridge.dll`,
        "C:\\Windows\\System32\\pforge_bridge.dll",
      ];

    default:
      throw new Error(`Unsupported platform: ${os}`);
  }
}

/**
 * Load pforge bridge library
 *
 * Searches for library in platform-specific locations.
 * Throws descriptive error if library not found.
 *
 * @returns DynamicLibrary handle with FFI symbols
 * @throws Error if library cannot be loaded
 *
 * Complexity: O(n) where n = number of paths (typically 3-4)
 */
export function loadLibrary(): Deno.DynamicLibrary<typeof SYMBOLS> {
  const libPaths = getLibraryPaths();
  const errors: string[] = [];

  for (const path of libPaths) {
    try {
      return Deno.dlopen(path, SYMBOLS);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      errors.push(`${path}: ${msg}`);
      continue;
    }
  }

  // All paths failed - throw helpful error
  throw new Error(
    "Could not find pforge_bridge library.\n\n" +
      "Searched paths:\n" +
      errors.map((e) => `  - ${e}`).join("\n") +
      "\n\n" +
      "To fix:\n" +
      "  1. Build the library: cargo build -p pforge-bridge\n" +
      "  2. Or install to system: cargo install --path crates/pforge-bridge\n" +
      "  3. Or set LD_LIBRARY_PATH (Linux) / DYLD_LIBRARY_PATH (macOS)",
  );
}

/**
 * Safe wrapper around FFI library
 *
 * Handles:
 * - Pointer lifecycle management
 * - UTF-8 encoding/decoding
 * - Memory safety
 * - Error propagation
 *
 * Lifecycle:
 * 1. constructor() - Load library
 * 2. Use FFI methods
 * 3. close() - Unload library
 */
export class FfiBridge {
  private lib: Deno.DynamicLibrary<typeof SYMBOLS>;
  private closed = false;

  constructor() {
    this.lib = loadLibrary();
  }

  /**
   * Get pforge version
   *
   * @returns Version string (e.g., "0.1.0")
   * @throws Error if library is closed or version unavailable
   *
   * Complexity: O(1)
   */
  version(): string {
    this.assertNotClosed();

    const ptr = this.lib.symbols.pforge_version();
    if (ptr === null) {
      throw new Error("Failed to get pforge version (null pointer returned)");
    }

    return Deno.UnsafePointerView.getCString(ptr);
  }

  /**
   * Execute handler by name
   *
   * @param handlerName Name of handler to execute
   * @param params JSON parameters as Uint8Array
   * @returns Result as Uint8Array
   * @throws Error if execution fails
   *
   * Complexity: O(n) where n = result size (for copying)
   */
  executeHandler(handlerName: string, params: Uint8Array): Uint8Array {
    this.assertNotClosed();

    // Validate handler name
    if (!handlerName || handlerName.trim().length === 0) {
      throw new Error("handler name cannot be empty");
    }

    const namePtr = this.toCString(handlerName);
    const paramsPtr = Deno.UnsafePointer.of(params as Uint8Array<ArrayBuffer>);

    // Deno FFI returns structs as arrays: [code, data, data_len, error]
    const result = this.lib.symbols.pforge_execute_handler(
      namePtr,
      paramsPtr,
      BigInt(params.length),
    ) as unknown as FfiResult;

    // Unpack struct array
    const [code, data, dataLen, error] = result;
    const dataSizeNum = typeof dataLen === "bigint" ? Number(dataLen) : dataLen;

    // Debug logging (commented out for production)
    // console.log("FFI Result:", { code, data, dataLen: dataSizeNum, error });

    try {
      if (code !== 0) {
        let errorMsg = "Unknown error";
        if (error !== null) {
          try {
            errorMsg = Deno.UnsafePointerView.getCString(
              error as Deno.PointerObject<unknown>,
            );
          } catch {
            errorMsg = `Error code: ${code}`;
          }
        }

        throw new Error(
          `Handler execution failed (code ${code}): ${errorMsg}`,
        );
      }

      if (data === null || dataSizeNum === 0) {
        return new Uint8Array(0);
      }

      // Copy result data before freeing
      const view = new Deno.UnsafePointerView(
        data as Deno.PointerObject<unknown>,
      );
      const resultData = new Uint8Array(dataSizeNum);

      for (let i = 0; i < dataSizeNum; i++) {
        resultData[i] = view.getUint8(i);
      }

      return resultData;
    } finally {
      // Always free the result
      this.lib.symbols.pforge_free_result(result as unknown as BufferSource);
    }
  }

  /**
   * Close library
   *
   * After calling this, all methods will throw.
   * Idempotent - safe to call multiple times.
   *
   * Complexity: O(1)
   */
  close(): void {
    if (!this.closed) {
      this.lib.close();
      this.closed = true;
    }
  }

  /**
   * Assert library is not closed
   *
   * @throws Error if library is closed
   */
  private assertNotClosed(): void {
    if (this.closed) {
      throw new Error("FfiBridge is closed");
    }
  }

  /**
   * Convert JavaScript string to null-terminated C string pointer
   *
   * @param str String to convert
   * @returns Pointer to null-terminated UTF-8 bytes
   *
   * Complexity: O(n) where n = string length
   */
  private toCString(str: string): Deno.PointerValue {
    const encoder = new TextEncoder();
    const bytes = encoder.encode(str + "\0");
    return Deno.UnsafePointer.of(bytes);
  }
}
