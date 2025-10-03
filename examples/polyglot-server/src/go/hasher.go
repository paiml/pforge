package main

import (
	"crypto/md5"
	"crypto/sha1"
	"crypto/sha256"
	"crypto/sha512"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"hash"
	"os"
)

type HashResult struct {
	Hash      string `json:"hash"`
	Algorithm string `json:"algorithm"`
	Data      string `json:"data"`
}

func calculateHash(algorithm, data string) (string, error) {
	var h hash.Hash

	switch algorithm {
	case "md5":
		h = md5.New()
	case "sha1":
		h = sha1.New()
	case "sha256":
		h = sha256.New()
	case "sha512":
		h = sha512.New()
	default:
		return "", fmt.Errorf("unsupported algorithm: %s", algorithm)
	}

	h.Write([]byte(data))
	return hex.EncodeToString(h.Sum(nil)), nil
}

func main() {
	if len(os.Args) < 3 {
		result := map[string]string{"error": "algorithm and data arguments required"}
		json.NewEncoder(os.Stdout).Encode(result)
		os.Exit(1)
	}

	algorithm := os.Args[1]
	data := os.Args[2]

	hashValue, err := calculateHash(algorithm, data)
	if err != nil {
		result := map[string]string{"error": err.Error()}
		json.NewEncoder(os.Stdout).Encode(result)
		os.Exit(1)
	}

	result := HashResult{
		Hash:      hashValue,
		Algorithm: algorithm,
		Data:      data,
	}

	json.NewEncoder(os.Stdout).Encode(result)
}
