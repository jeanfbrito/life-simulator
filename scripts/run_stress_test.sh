#!/bin/bash
# Entity Count Stress Test Runner
#
# This script runs comprehensive stress testing with various entity counts
# and generates performance analysis reports.
#
# Usage:
#   ./scripts/run_stress_test.sh
#   ./scripts/run_stress_test.sh --duration 120 --release
#   ./scripts/run_stress_test.sh --quick   # Fast test runs
#

set -e

# Configuration
BUILD_MODE="release"
TEST_DURATION=60
QUICK_MODE=false
OUTPUT_DIR="stress_test_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --duration)
            TEST_DURATION=$2
            shift 2
            ;;
        --quick)
            QUICK_MODE=true
            TEST_DURATION=10
            shift
            ;;
        --debug)
            BUILD_MODE="debug"
            shift
            ;;
        *)
            echo "Unknown argument: $1"
            exit 1
            ;;
    esac
done

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║        ENTITY COUNT STRESS TEST SUITE                          ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "Configuration:"
echo "  Build Mode: $BUILD_MODE"
echo "  Test Duration: $TEST_DURATION seconds"
echo "  Output Directory: $OUTPUT_DIR"
echo ""

# Verify stress_test binary exists
echo "Building stress_test binary..."
cargo build --bin stress_test --"$BUILD_MODE" 2>&1 | grep -v "^warning:" || true

BINARY="target/$BUILD_MODE/stress_test"
if [ ! -f "$BINARY" ]; then
    echo "Error: stress_test binary not found at $BINARY"
    exit 1
fi

echo "✓ Binary ready"
echo ""

# Define test scenarios
declare -a SCENARIOS=("100" "300" "500" "700")
declare -a CONFIGS=(
    "config/spawn_config_stress_100.ron"
    "config/spawn_config_stress_300.ron"
    "config/spawn_config_stress_test.ron"
    "config/spawn_config_stress_700.ron"
)

# Run each scenario
for i in "${!SCENARIOS[@]}"; do
    scenario="${SCENARIOS[$i]}"
    config="${CONFIGS[$i]}"

    if [ ! -f "$config" ]; then
        echo "⚠️  Config not found: $config"
        continue
    fi

    echo "Running stress test: $scenario entities"
    echo "  Config: $config"
    echo "  Duration: $TEST_DURATION seconds"

    OUTPUT_FILE="$OUTPUT_DIR/stress_test_${scenario}_entities_${TIMESTAMP}.log"

    timeout $((TEST_DURATION + 30)) "$BINARY" \
        2>&1 | tee "$OUTPUT_FILE" \
        || echo "Test completed or timed out"

    echo "  Results saved to: $OUTPUT_FILE"
    echo ""
done

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║        STRESS TEST SUITE COMPLETE                             ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "Output files:"
ls -lh "$OUTPUT_DIR"/stress_test_*_${TIMESTAMP}.log 2>/dev/null || echo "No output files found"
echo ""
echo "Next steps:"
echo "  1. Review results in $OUTPUT_DIR/"
echo "  2. Compare TPS and tick times across entity counts"
echo "  3. Identify scaling bottlenecks"
echo "  4. Run: cargo flamegraph --bin stress_test (if available)"
