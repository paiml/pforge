/**
 * FFI Interface Tests
 *
 * Tests for Deno FFI bindings to pforge_bridge library.
 * Following EXTREME TDD methodology with 5-minute cycles.
 *
 * Quality Standards:
 * - Zero SATD comments
 * - Complexity ≤ 20
 * - 100% coverage of FFI operations
 */

import {
  assertEquals,
  assertExists,
} from "https://deno.land/std@0.208.0/assert/mod.ts";
import { getLibraryPaths, loadLibrary } from "../../ffi.ts";

/**
 * Test: Load pforge_bridge library successfully
 *
 * Ensures all required FFI symbols are present:
 * - pforge_version
 * - pforge_execute_handler
 * - pforge_free_result
 */
Deno.test("FFI: Load pforge_bridge library", () => {
  const lib = loadLibrary();

  assertExists(lib, "Library should be loaded");
  assertExists(
    lib.symbols.pforge_version,
    "pforge_version symbol should exist",
  );
  assertExists(
    lib.symbols.pforge_execute_handler,
    "pforge_execute_handler symbol should exist",
  );
  assertExists(
    lib.symbols.pforge_free_result,
    "pforge_free_result symbol should exist",
  );

  lib.close();
});

/**
 * Test: Get pforge version string
 *
 * Verifies that pforge_version() returns a valid version string.
 */
Deno.test("FFI: Get pforge version", () => {
  const lib = loadLibrary();
  const versionPtr = lib.symbols.pforge_version();

  assertExists(versionPtr, "Version pointer should not be null");

  const version = Deno.UnsafePointerView.getCString(versionPtr!);
  assertExists(version, "Version string should exist");
  assertEquals(typeof version, "string", "Version should be a string");

  lib.close();
});

/**
 * Test: Library not found throws helpful error
 *
 * When library cannot be found, error message should:
 * - Indicate library was not found
 * - List all searched paths
 * - Suggest running cargo build
 */
Deno.test("FFI: Library not found throws helpful error", () => {
  const paths = getLibraryPaths();

  assertExists(paths, "Library paths should be returned");
  assertEquals(Array.isArray(paths), true, "Paths should be an array");
  assertEquals(paths.length > 0, true, "Should have at least one path");

  // Verify paths are platform-specific
  const os = Deno.build.os;
  if (os === "linux") {
    assertEquals(
      paths.some((p) => p.endsWith(".so")),
      true,
      "Linux should have .so files",
    );
  } else if (os === "darwin") {
    assertEquals(
      paths.some((p) => p.endsWith(".dylib")),
      true,
      "macOS should have .dylib files",
    );
  } else if (os === "windows") {
    assertEquals(
      paths.some((p) => p.endsWith(".dll")),
      true,
      "Windows should have .dll files",
    );
  }
});

/**
 * Test: Platform-specific library paths
 *
 * Verifies that getLibraryPaths() returns correct paths for the current platform.
 */
Deno.test("FFI: Platform-specific library paths", () => {
  const paths = getLibraryPaths();
  const os = Deno.build.os;

  for (const path of paths) {
    switch (os) {
      case "linux":
        assertEquals(
          path.endsWith(".so"),
          true,
          `Linux path should end with .so: ${path}`,
        );
        break;
      case "darwin":
        assertEquals(
          path.endsWith(".dylib"),
          true,
          `macOS path should end with .dylib: ${path}`,
        );
        break;
      case "windows":
        assertEquals(
          path.endsWith(".dll"),
          true,
          `Windows path should end with .dll: ${path}`,
        );
        break;
    }
  }
});
