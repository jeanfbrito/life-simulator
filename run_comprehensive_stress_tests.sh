#!/bin/bash
# Comprehensive Stress Test Runner
# Runs life-simulator with different entity counts and captures performance metrics

set -e

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_DIR="stress_test_results_${TIMESTAMP}"
TEST_DURATION=${STRESS_TEST_DURATION:-30}

mkdir -p "$RESULTS_DIR"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║      COMPREHENSIVE ENTITY STRESS TEST SUITE                    ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "Configuration:"
echo "  Test Duration: ${TEST_DURATION} seconds per scenario"
echo "  Results Directory: ${RESULTS_DIR}"
echo ""

# Build the binary
echo "📦 Building life-simulator in release mode..."
cargo build --release --bin life-simulator 2>&1 | grep -vE "^warning:" | tail -5

echo "✅ Build complete"
echo ""

# Test scenarios
declare -a SCENARIOS=(
    "100:config/spawn_config_stress_100.ron"
    "300:config/spawn_config_stress_300.ron"
    "500:config/spawn_config_stress_test.ron"
    "700:config/spawn_config_stress_700.ron"
)

for scenario in "${SCENARIOS[@]}"; do
    IFS=':' read -r entity_count config_file <<< "$scenario"

    if [ ! -f "$config_file" ]; then
        echo "⚠️  Config not found: $config_file - skipping"
        continue
    fi

    echo "═══════════════════════════════════════════════════════════════"
    echo "🧪 Testing: ${entity_count} entities"
    echo "   Config: ${config_file}"
    echo "   Duration: ${TEST_DURATION} seconds"
    echo "═══════════════════════════════════════════════════════════════"

    LOG_FILE="${RESULTS_DIR}/stress_${entity_count}_entities.log"

    # Run simulator with timeout
    START_TIME=$(date +%s)

    (
        SPAWN_CONFIG="$config_file" \
        DISABLE_WEB_SERVER=1 \
        RUST_LOG=info \
        cargo run --release --bin life-simulator 2>&1 | grep -E "(Tick #|TPS|entities|SIMULATION)"
    ) > "$LOG_FILE" 2>&1 &

    SIM_PID=$!
    echo "   Started simulator (PID: $SIM_PID)"

    # Wait for test duration
    sleep "$TEST_DURATION"

    # Kill the simulator
    kill $SIM_PID 2>/dev/null || true
    wait $SIM_PID 2>/dev/null || true

    END_TIME=$(date +%s)
    ELAPSED=$((END_TIME - START_TIME))

    echo "   ✅ Test completed (${ELAPSED}s)"
    echo "   📄 Log: $LOG_FILE"

    # Extract performance summary
    TICK_COUNT=$(grep -c "Tick #" "$LOG_FILE" 2>/dev/null || echo "0")

    if [ "$TICK_COUNT" -gt 0 ]; then
        AVG_TPS=$(echo "scale=2; $TICK_COUNT / $ELAPSED" | bc)
        echo "   📊 Ticks: $TICK_COUNT | Avg TPS: ${AVG_TPS}"
    fi

    echo ""
done

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                  ALL TESTS COMPLETED                           ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "📁 Results saved to: $RESULTS_DIR/"
echo ""
echo "📊 Summary Report:"
echo "═══════════════════════════════════════════════════════════════"

for scenario in "${SCENARIOS[@]}"; do
    IFS=':' read -r entity_count config_file <<< "$scenario"
    LOG_FILE="${RESULTS_DIR}/stress_${entity_count}_entities.log"

    if [ -f "$LOG_FILE" ]; then
        TICK_COUNT=$(grep -c "Tick #" "$LOG_FILE" 2>/dev/null || echo "0")

        # Determine targets
        case $entity_count in
            100) TARGET_TPS="10.0"; TARGET_MS="50" ;;
            300) TARGET_TPS="10.0"; TARGET_MS="75" ;;
            500) TARGET_TPS="10.0"; TARGET_MS="100" ;;
            700) TARGET_TPS="8.0";  TARGET_MS="150" ;;
            *) TARGET_TPS="10.0"; TARGET_MS="100" ;;
        esac

        if [ "$TICK_COUNT" -gt 0 ]; then
            ACTUAL_TPS=$(echo "scale=2; $TICK_COUNT / $TEST_DURATION" | bc)
            STATUS="✅"

            # Simple pass/fail check
            if (( $(echo "$ACTUAL_TPS < $TARGET_TPS * 0.8" | bc -l) )); then
                STATUS="⚠️"
            fi

            printf "%-12s | %5s ticks | %6s TPS (target: %s) | %s\n" \
                "${entity_count} entities" "$TICK_COUNT" "$ACTUAL_TPS" "$TARGET_TPS" "$STATUS"
        else
            printf "%-12s | NO DATA - check log file\n" "${entity_count} entities"
        fi
    fi
done

echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Next steps:"
echo "  1. Review detailed logs in $RESULTS_DIR/"
echo "  2. Check for performance bottlenecks"
echo "  3. Run flamegraph for detailed profiling"
echo ""
