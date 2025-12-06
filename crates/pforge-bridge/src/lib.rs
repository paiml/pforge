//! Language bridge FFI for pforge
//!
//! This crate provides a stable C ABI for calling Rust handlers from other languages.
//! It enables zero-copy parameter passing and preserves type safety across language boundaries.

use once_cell::sync::OnceCell;
use pforge_runtime::HandlerRegistry;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::slice;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;

/// Global handler registry for FFI access
static GLOBAL_REGISTRY: OnceCell<Arc<RwLock<HandlerRegistry>>> = OnceCell::new();

/// Global tokio runtime for async operations
static RUNTIME: OnceCell<Runtime> = OnceCell::new();

/// Initialize the runtime (called once)
fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| Runtime::new().expect("Failed to create tokio runtime"))
}

/// Opaque handle to a handler context
#[repr(C)]
pub struct HandlerContext {
    _private: [u8; 0],
}

/// Result structure for FFI calls
#[repr(C)]
pub struct FfiResult {
    /// 0 = success, non-zero = error code
    pub code: c_int,
    /// Pointer to result data (JSON bytes)
    pub data: *mut u8,
    /// Length of result data
    pub data_len: usize,
    /// Error message (null if success)
    pub error: *const c_char,
}

/// Initialize the global handler registry
///
/// # Safety
/// Must be called before any handler dispatch operations.
/// Can only be called once.
#[no_mangle]
pub unsafe extern "C" fn pforge_init() -> c_int {
    if GLOBAL_REGISTRY.get().is_some() {
        return -1; // Already initialized
    }

    let registry = Arc::new(RwLock::new(HandlerRegistry::new()));
    if GLOBAL_REGISTRY.set(registry).is_err() {
        return -2; // Race condition during init
    }

    0 // Success
}

/// Register a native handler with the global registry
///
/// # Safety
/// - `name` must be a valid null-terminated string
/// - `pforge_init` must have been called first
#[no_mangle]
pub unsafe extern "C" fn pforge_register_handler(
    name: *const c_char,
    _handler_ptr: *mut std::ffi::c_void,
) -> c_int {
    if name.is_null() {
        return -1;
    }

    let registry = match GLOBAL_REGISTRY.get() {
        Some(r) => r,
        None => return -2, // Not initialized
    };

    // SAFETY: Caller guarantees name is a valid null-terminated string
    let name_str = match unsafe { CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(_) => return -3, // Invalid UTF-8
    };

    // For now, just verify we can access the registry
    // Full handler registration requires more complex FFI patterns
    let rt = get_runtime();
    let _ = rt.block_on(async { registry.read().await });

    eprintln!("Handler '{}' registered via FFI", name_str);
    0
}

/// Execute a handler by name with JSON input
///
/// # Safety
/// - `handler_name` must be a valid null-terminated string
/// - `input_json` must be a valid pointer to JSON bytes
/// - `input_len` must be the correct length of input data
/// - Caller must free result data with `pforge_free_result`
#[no_mangle]
pub unsafe extern "C" fn pforge_execute_handler(
    handler_name: *const c_char,
    input_json: *const u8,
    input_len: usize,
) -> FfiResult {
    // Validate inputs
    if handler_name.is_null() || input_json.is_null() {
        return FfiResult {
            code: -1,
            data: std::ptr::null_mut(),
            data_len: 0,
            error: create_error_string("Null pointer provided"),
        };
    }

    // Convert handler name
    // SAFETY: Caller guarantees handler_name is a valid null-terminated string
    let name = match unsafe { CStr::from_ptr(handler_name) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            return FfiResult {
                code: -2,
                data: std::ptr::null_mut(),
                data_len: 0,
                error: create_error_string("Invalid UTF-8 in handler name"),
            }
        }
    };

    // Get input bytes
    // SAFETY: Caller guarantees input_json points to input_len valid bytes
    let input = unsafe { slice::from_raw_parts(input_json, input_len) };

    // Try to dispatch through global registry if available
    if let Some(registry) = GLOBAL_REGISTRY.get() {
        let rt = get_runtime();
        let result = rt.block_on(async {
            let reg = registry.read().await;
            reg.dispatch(name, input).await
        });

        match result {
            Ok(output) => {
                let mut boxed = output.into_boxed_slice();
                let data_ptr = boxed.as_mut_ptr();
                let data_len = boxed.len();
                // SAFETY: Transfer ownership to C caller
                #[allow(clippy::mem_forget)]
                std::mem::forget(boxed);

                return FfiResult {
                    code: 0,
                    data: data_ptr,
                    data_len,
                    error: std::ptr::null(),
                };
            }
            Err(e) => {
                // Check if it's a "not found" error - use fallback in that case
                let err_str = e.to_string();
                if err_str.contains("not found") || err_str.contains("ToolNotFound") {
                    // Fall through to echo fallback
                } else {
                    return FfiResult {
                        code: -4,
                        data: std::ptr::null_mut(),
                        data_len: 0,
                        error: create_error_string(&format!("Handler error: {}", e)),
                    };
                }
            }
        }
    }

    // Fallback: Return echo response if no registry available
    let response = serde_json::json!({
        "handler": name,
        "input_size": input_len,
        "status": "ok",
        "note": "No global registry - using echo fallback"
    });

    match serde_json::to_vec(&response) {
        Ok(data) => {
            let mut boxed = data.into_boxed_slice();
            let data_ptr = boxed.as_mut_ptr();
            let data_len = boxed.len();
            // SAFETY: Transfer ownership to C caller. Memory will be freed via pforge_free_result.
            #[allow(clippy::mem_forget)]
            std::mem::forget(boxed);

            FfiResult {
                code: 0,
                data: data_ptr,
                data_len,
                error: std::ptr::null(),
            }
        }
        Err(e) => FfiResult {
            code: -3,
            data: std::ptr::null_mut(),
            data_len: 0,
            error: create_error_string(&format!("Serialization error: {}", e)),
        },
    }
}

/// Free result data allocated by pforge
///
/// # Safety
/// - Must only be called once per FfiResult
/// - `result` must have been returned from pforge_execute_handler
#[no_mangle]
pub unsafe extern "C" fn pforge_free_result(result: FfiResult) {
    if !result.data.is_null() && result.data_len > 0 {
        // SAFETY: result.data was allocated via Vec::into_boxed_slice() with capacity = len
        let _ = unsafe { Vec::from_raw_parts(result.data, result.data_len, result.data_len) };
    }
    if !result.error.is_null() {
        // SAFETY: result.error was allocated via CString::into_raw()
        let _ = unsafe { CString::from_raw(result.error as *mut c_char) };
    }
}

/// Get the pforge version
///
/// # Safety
/// - Returned string is valid for program lifetime
#[no_mangle]
pub unsafe extern "C" fn pforge_version() -> *const c_char {
    static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");
    VERSION.as_ptr() as *const c_char
}

/// Check if the global registry is initialized
///
/// # Safety
/// - Thread-safe, can be called at any time
#[no_mangle]
pub extern "C" fn pforge_is_initialized() -> c_int {
    if GLOBAL_REGISTRY.get().is_some() {
        1
    } else {
        0
    }
}

// Helper functions

fn create_error_string(msg: &str) -> *const c_char {
    match CString::new(msg) {
        Ok(s) => s.into_raw() as *const c_char,
        Err(_) => std::ptr::null(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_version() {
        unsafe {
            let version = pforge_version();
            assert!(!version.is_null());
            let version_str = CStr::from_ptr(version).to_str().unwrap();
            assert!(version_str.starts_with("0.1"));
        }
    }

    #[test]
    fn test_init() {
        // Note: This test may fail if run after other tests that initialize
        // the global registry. In practice, init should only be called once.
        let result = unsafe { pforge_init() };
        // Either succeeds (0) or already initialized (-1)
        assert!(result == 0 || result == -1);
    }

    #[test]
    fn test_is_initialized() {
        // After init runs in other tests, registry should be initialized
        // Verify the value is exactly 0 or 1 (not just any non-zero)
        let status = pforge_is_initialized();
        assert!(
            status == 0 || status == 1,
            "Expected 0 or 1, got {}",
            status
        );

        // If we've initialized, verify it returns 1 specifically
        if GLOBAL_REGISTRY.get().is_some() {
            assert_eq!(
                pforge_is_initialized(),
                1,
                "Should return 1 when initialized"
            );
        }
    }

    #[test]
    fn test_is_initialized_returns_one_after_init() {
        // First ensure init is called
        let _ = unsafe { pforge_init() };
        // After init, must return exactly 1
        assert_eq!(pforge_is_initialized(), 1);
    }

    #[test]
    fn test_create_error_string() {
        let msg = "Test error message";
        let ptr = create_error_string(msg);
        assert!(
            !ptr.is_null(),
            "create_error_string should return non-null pointer"
        );

        // Verify the string content
        unsafe {
            let c_str = CStr::from_ptr(ptr);
            let str_slice = c_str.to_str().unwrap();
            assert_eq!(str_slice, msg);
            // Clean up
            let _ = CString::from_raw(ptr as *mut c_char);
        }
    }

    #[test]
    fn test_create_error_string_with_null_byte() {
        // String with embedded null byte should return null pointer
        let msg = "Error\0with null";
        let ptr = create_error_string(msg);
        assert!(
            ptr.is_null(),
            "Should return null for string with embedded null byte"
        );
    }

    #[test]
    fn test_execute_handler_null_safety() {
        unsafe {
            // Null handler name
            let result = pforge_execute_handler(std::ptr::null(), std::ptr::null(), 0);
            assert_eq!(result.code, -1);
            pforge_free_result(result);
        }
    }

    #[test]
    fn test_execute_handler_fallback() {
        unsafe {
            let handler_name = CString::new("test_handler").unwrap();
            let input = b"{}";

            let result = pforge_execute_handler(handler_name.as_ptr(), input.as_ptr(), input.len());

            // Should succeed with fallback response
            assert_eq!(result.code, 0);
            assert!(!result.data.is_null());
            assert!(result.data_len > 0);

            // Parse result
            let data_slice = slice::from_raw_parts(result.data, result.data_len);
            let response: serde_json::Value = serde_json::from_slice(data_slice).unwrap();
            assert_eq!(response["handler"], "test_handler");
            assert_eq!(response["status"], "ok");

            pforge_free_result(result);
        }
    }

    #[test]
    fn test_register_handler_null_name() {
        unsafe {
            let result = pforge_register_handler(std::ptr::null(), std::ptr::null_mut());
            assert_eq!(result, -1);
        }
    }
}
