package pforge

/*
#cgo LDFLAGS: -L../../target/release -lpforge_bridge
#include <stdlib.h>

typedef struct {
    int code;
    unsigned char* data;
    size_t data_len;
    const char* error;
} FfiResult;

extern const char* pforge_version();
extern FfiResult pforge_execute_handler(const char* handler_name, const unsigned char* input_json, size_t input_len);
extern void pforge_free_result(FfiResult result);
*/
import "C"
import (
	"encoding/json"
	"errors"
	"fmt"
	"unsafe"
)

// Bridge provides Go interface to pforge FFI
type Bridge struct{}

// Version returns the pforge version
func (b *Bridge) Version() string {
	cVersion := C.pforge_version()
	return C.GoString(cVersion)
}

// ExecuteHandler calls a pforge handler with JSON input
func (b *Bridge) ExecuteHandler(handlerName string, input map[string]interface{}) (map[string]interface{}, error) {
	// Serialize input to JSON
	inputJSON, err := json.Marshal(input)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal input: %w", err)
	}

	// Convert Go string to C string
	cHandlerName := C.CString(handlerName)
	defer C.free(unsafe.Pointer(cHandlerName))

	// Call FFI
	result := C.pforge_execute_handler(
		cHandlerName,
		(*C.uchar)(unsafe.Pointer(&inputJSON[0])),
		C.size_t(len(inputJSON)),
	)
	defer C.pforge_free_result(result)

	// Check for errors
	if result.code != 0 {
		if result.error != nil {
			errorMsg := C.GoString(result.error)
			return nil, fmt.Errorf("handler execution failed (code %d): %s", result.code, errorMsg)
		}
		return nil, fmt.Errorf("handler execution failed with code %d", result.code)
	}

	// Extract result data
	if result.data == nil || result.data_len == 0 {
		return make(map[string]interface{}), nil
	}

	resultBytes := C.GoBytes(unsafe.Pointer(result.data), C.int(result.data_len))

	var output map[string]interface{}
	if err := json.Unmarshal(resultBytes, &output); err != nil {
		return nil, fmt.Errorf("failed to unmarshal result: %w", err)
	}

	return output, nil
}

// NewBridge creates a new pforge bridge instance
func NewBridge() *Bridge {
	return &Bridge{}
}
