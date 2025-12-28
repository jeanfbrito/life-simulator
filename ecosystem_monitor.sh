#!/bin/bash

# Ecosystem Balance Monitoring Script
# Runs simulation and extracts population data every 500 ticks

LOG_FILE="ecosystem_test_$(date +%Y%m%d_%H%M%S).log"
POPULATION_DATA="population_data_$(date +%Y%m%d_%H%M%S).csv"

echo "Starting ecosystem balance test..."
echo "Log file: $LOG_FILE"
echo "Population data: $POPULATION_DATA"
echo ""

# CSV header
echo "tick,wolves,foxes,deer,rabbits,total_entities,timestamp" > "$POPULATION_DATA"

# Run simulation in background and monitor
RUST_LOG=info cargo run --release --bin life-simulator 2>&1 | tee "$LOG_FILE" &
SIM_PID=$!

echo "Simulation running with PID: $SIM_PID"
echo "Monitoring for 10,000 ticks (approximately 16 minutes)..."
echo ""

# Monitor the log file for population data
tail -f "$LOG_FILE" 2>/dev/null | while read line; do
    # Extract tick number and entity counts
    if echo "$line" | grep -q "Tick.*entities"; then
        TICK=$(echo "$line" | grep -oE "Tick [0-9]+" | grep -oE "[0-9]+")
        
        # Extract species counts from subsequent lines
        WOLVES=$(grep -A 50 "Tick $TICK" "$LOG_FILE" | grep -i "wolf" | grep -oE "[0-9]+ entities" | grep -oE "[0-9]+" | head -1)
        FOXES=$(grep -A 50 "Tick $TICK" "$LOG_FILE" | grep -i "fox" | grep -oE "[0-9]+ entities" | grep -oE "[0-9]+" | head -1)
        DEER=$(grep -A 50 "Tick $TICK" "$LOG_FILE" | grep -i "deer" | grep -oE "[0-9]+ entities" | grep -oE "[0-9]+" | head -1)
        RABBITS=$(grep -A 50 "Tick $TICK" "$LOG_FILE" | grep -i "rabbit" | grep -oE "[0-9]+ entities" | grep -oE "[0-9]+" | head -1)
        
        # Only log every 500 ticks
        if [ $((TICK % 500)) -eq 0 ] && [ "$TICK" -gt 0 ]; then
            TOTAL=$((${WOLVES:-0} + ${FOXES:-0} + ${DEER:-0} + ${RABBITS:-0}))
            TIMESTAMP=$(date +%H:%M:%S)
            echo "$TICK,${WOLVES:-0},${FOXES:-0},${DEER:-0},${RABBITS:-0},$TOTAL,$TIMESTAMP" >> "$POPULATION_DATA"
            echo "[$(date +%H:%M:%S)] Tick $TICK: W:${WOLVES:-0} F:${FOXES:-0} D:${DEER:-0} R:${RABBITS:-0} Total:$TOTAL"
        fi
        
        # Stop after 10,000 ticks
        if [ "$TICK" -ge 10000 ]; then
            echo ""
            echo "Reached 10,000 ticks. Stopping simulation..."
            kill $SIM_PID 2>/dev/null
            break
        fi
    fi
done

echo ""
echo "Test complete!"
echo "Results saved to: $POPULATION_DATA"
