package main

import (
	"fmt"
	"log"

	"./pforge"
)

func main() {
	bridge := pforge.NewBridge()

	fmt.Println("=" + "=" * 59)
	fmt.Printf("pforge Go Bridge Example\n")
	fmt.Printf("Version: %s\n", bridge.Version())
	fmt.Println("=" + "=" * 59)

	// Execute handler with JSON input
	input := map[string]interface{}{
		"operation": "greet",
		"name":      "Go",
		"count":     5,
	}

	fmt.Printf("\nInput: %+v\n", input)

	result, err := bridge.ExecuteHandler("go_handler", input)
	if err != nil {
		log.Fatalf("Error: %v", err)
	}

	fmt.Printf("Output: %+v\n", result)
	fmt.Printf("Handler: %v\n", result["handler"])
	fmt.Printf("Status: %v\n", result["status"])
}
