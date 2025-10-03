#!/bin/bash
# Run all fuzz targets for quick smoke testing
# Usage: ./run_fuzz.sh [duration_seconds]

set -e

DURATION=${1:-60}  # Default 60 seconds per target
RUNS=10000

echo "🔍 Running pforge fuzz targets (${DURATION}s each, max ${RUNS} runs)"
echo "=========================================="

for target in fuzz_config_parser fuzz_handler_dispatch fuzz_validation; do
    echo ""
    echo "📋 Fuzzing: $target"
    echo "----------------------------------------"

    if cargo +nightly fuzz run $target -- \
        -max_total_time=$DURATION \
        -runs=$RUNS \
        -print_final_stats=1 2>&1 | tee "fuzz_${target}.log"; then
        echo "✅ $target: PASS (no crashes found)"
    else
        echo "❌ $target: FAILED (crash detected!)"
        exit 1
    fi
done

echo ""
echo "=========================================="
echo "✅ All fuzz targets completed successfully"
echo ""
echo "Logs saved to: fuzz_*.log"
echo "Corpus saved to: corpus/*/
"
