//! Language bridge FFI for pforge
//!
//! This crate provides a stable C ABI for calling Rust handlers from other languages.
//! It enables zero-copy parameter passing and preserves type safety across language boundaries.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::slice;

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
    let name = match CStr::from_ptr(handler_name).to_str() {
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
    let _input = slice::from_raw_parts(input_json, input_len);

    // TODO: Actually dispatch to handler registry
    // For now, return a simple echo response
    let response = serde_json::json!({
        "handler": name,
        "input_size": input_len,
        "status": "ok"
    });

    match serde_json::to_vec(&response) {
        Ok(data) => {
            let mut boxed = data.into_boxed_slice();
            let data_ptr = boxed.as_mut_ptr();
            let data_len = boxed.len();
            // SAFETY: Transfer ownership to C caller. Memory will be freed via pforge_free_result.
            // This is the correct pattern for FFI memory management.
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
        let _ = Vec::from_raw_parts(result.data, result.data_len, result.data_len);
    }
    if !result.error.is_null() {
        let _ = CString::from_raw(result.error as *mut c_char);
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
    fn test_execute_handler_null_safety() {
        unsafe {
            // Null handler name
            let result = pforge_execute_handler(std::ptr::null(), std::ptr::null(), 0);
            assert_eq!(result.code, -1);
            pforge_free_result(result);
        }
    }

    #[test]
    fn test_execute_handler_success() {
        unsafe {
            let handler_name = CString::new("test_handler").unwrap();
            let input = b"{}";

            let result = pforge_execute_handler(handler_name.as_ptr(), input.as_ptr(), input.len());

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
}
