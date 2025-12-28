#!/bin/bash

# Ecosystem Balance Testing Script
# Runs 10,000 tick simulation and monitors population dynamics

set -e

echo "════════════════════════════════════════════════════════════════"
echo "🧪 ECOSYSTEM BALANCE TEST"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Create results directory
RESULTS_DIR="ecosystem_test_results_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"
echo "📁 Results directory: $RESULTS_DIR"
echo ""

LOG_FILE="$RESULTS_DIR/simulation.log"
POPULATION_CSV="$RESULTS_DIR/population_data.csv"
EVENTS_LOG="$RESULTS_DIR/events.log"
SUMMARY_FILE="$RESULTS_DIR/summary.txt"

# CSV header
echo "tick,wolves,foxes,deer,rabbits,total_prey,total_predators,total,prey_predator_ratio,timestamp" > "$POPULATION_CSV"

# Start simulation in background
echo "🚀 Starting simulation with RUST_LOG=info..."
RUST_LOG=info cargo run --release --bin life-simulator > "$LOG_FILE" 2>&1 &
SIM_PID=$!

echo "   Simulation PID: $SIM_PID"
echo "   Monitoring output..."
echo ""

# Monitor log file
tail -f "$LOG_FILE" 2>/dev/null | while IFS= read -r line; do
    echo "$line"
    
    # Extract tick number from various log formats
    if echo "$line" | grep -q "Tick"; then
        TICK=$(echo "$line" | grep -oE "Tick [0-9]+" | grep -oE "[0-9]+" | head -1)
        
        # Wait a moment for entity counts to be logged
        sleep 0.5
        
        # Extract species counts (look for pattern like "Wolf: 10 entities")
        WOLVES=$(grep -A 100 "Tick $TICK" "$LOG_FILE" | grep -i "Wolf:" | grep -oE "[0-9]+ entities" | grep -oE "[0-9]+" | head -1)
        FOXES=$(grep -A 100 "Tick $TICK" "$LOG_FILE" | grep -i "Fox:" | grep -oE "[0-9]+ entities" | grep -oE "[0-9]+" | head -1)
        DEER=$(grep -A 100 "Tick $TICK" "$LOG_FILE" | grep -i "Deer:" | grep -oE "[0-9]+ entities" | grep -oE "[0-9]+" | head -1)
        RABBITS=$(grep -A 100 "Tick $TICK" "$LOG_FILE" | grep -i "Rabbit:" | grep -oE "[0-9]+ entities" | grep -oE "[0-9]+" | head -1)
        
        # Default to 0 if not found
        WOLVES=${WOLVES:-0}
        FOXES=${FOXES:-0}
        DEER=${DEER:-0}
        RABBITS=${RABBITS:-0}
        
        # Only log every 500 ticks
        if [ $((TICK % 500)) -eq 0 ] && [ "$TICK" -gt 0 ]; then
            TOTAL_PREY=$((DEER + RABBITS))
            TOTAL_PRED=$((WOLVES + FOXES))
            TOTAL=$((TOTAL_PREY + TOTAL_PRED))
            
            # Calculate ratio (avoid division by zero)
            if [ "$TOTAL_PRED" -gt 0 ]; then
                RATIO=$(echo "scale=2; $TOTAL_PREY / $TOTAL_PRED" | bc)
            else
                RATIO="∞"
            fi
            
            TIMESTAMP=$(date +%H:%M:%S)
            
            echo "$TICK,$WOLVES,$FOXES,$DEER,$RABBITS,$TOTAL_PREY,$TOTAL_PRED,$TOTAL,$RATIO,$TIMESTAMP" >> "$POPULATION_CSV"
            
            echo ""
            echo "════════════════════════════════════════════════════════════════"
            echo "📊 Tick $TICK Population Status ($(date +%H:%M:%S))"
            echo "────────────────────────────────────────────────────────────────"
            echo "  Prey:      Rabbits: $RABBITS  Deer: $DEER  (Total: $TOTAL_PREY)"
            echo "  Predators: Wolves: $WOLVES   Foxes: $FOXES  (Total: $TOTAL_PRED)"
            echo "  Ratio:     ${RATIO}:1 (Prey:Predator)"
            echo "  Total:     $TOTAL entities"
            echo "════════════════════════════════════════════════════════════════"
            echo ""
        fi
        
        # Stop after 10,000 ticks
        if [ "$TICK" -ge 10000 ]; then
            echo ""
            echo "✅ Reached 10,000 ticks. Stopping simulation..."
            kill $SIM_PID 2>/dev/null || true
            sleep 2
            break
        fi
    fi
    
    # Extract important events
    if echo "$line" | grep -qE "(birth|died|hunting|pack|mating|flee)"; then
        echo "$line" >> "$EVENTS_LOG"
    fi
done

# Wait for simulation to fully stop
wait $SIM_PID 2>/dev/null || true

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "✅ SIMULATION COMPLETE"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "📁 Results saved to: $RESULTS_DIR/"
echo "   - simulation.log: Full simulation log"
echo "   - population_data.csv: Population data over time"
echo "   - events.log: Key ecosystem events"
echo ""

# Generate summary
echo "Generating analysis summary..."

{
    echo "════════════════════════════════════════════════════════════════"
    echo "ECOSYSTEM BALANCE TEST SUMMARY"
    echo "════════════════════════════════════════════════════════════════"
    echo "Test Date: $(date)"
    echo ""
    echo "POPULATION STATISTICS"
    echo "────────────────────────────────────────────────────────────────"
    
    if [ -s "$POPULATION_CSV" ]; then
        echo ""
        echo "Population Data (every 500 ticks):"
        echo ""
        column -t -s',' "$POPULATION_CSV" | head -25
        echo ""
        
        # Calculate averages (skip header)
        tail -n +2 "$POPULATION_CSV" | awk -F',' '{
            w+=$2; f+=$3; d+=$4; r+=$5; n++
        } END {
            if(n>0) {
                printf "AVERAGES ACROSS TEST:\n"
                printf "  Wolves:    %.1f\n", w/n
                printf "  Foxes:     %.1f\n", f/n
                printf "  Deer:      %.1f\n", d/n
                printf "  Rabbits:   %.1f\n", r/n
                printf "  Total:     %.1f\n", (w+f+d+r)/n
            }
        }'
        
        echo ""
        echo "FINAL POPULATIONS:"
        tail -1 "$POPULATION_CSV" | awk -F',' '{
            printf "  Wolves:    %d\n", $2
            printf "  Foxes:     %d\n", $3
            printf "  Deer:      %d\n", $4
            printf "  Rabbits:   %d\n", $5
            printf "  Total:     %d\n", $7
        }'
    fi
    
    echo ""
    echo "KEY EVENTS SUMMARY"
    echo "────────────────────────────────────────────────────────────────"
    
    if [ -s "$EVENTS_LOG" ]; then
        echo "Birth events: $(grep -c -i birth "$EVENTS_LOG" || echo 0)"
        echo "Death events: $(grep -c -i died "$EVENTS_LOG" || echo 0)"
        echo "Hunting events: $(grep -c -i hunt "$EVENTS_LOG" || echo 0)"
        echo "Pack events: $(grep -c -i pack "$EVENTS_LOG" || echo 0)"
        echo "Mating events: $(grep -c -i mat "$EVENTS_LOG" || echo 0)"
        echo "Flee events: $(grep -c -i flee "$EVENTS_LOG" || echo 0)"
    fi
    
    echo ""
    echo "════════════════════════════════════════════════════════════════"
    
} | tee "$SUMMARY_FILE"

echo ""
echo "✅ Summary saved to: $SUMMARY_FILE"
echo ""
echo "Next steps:"
echo "  1. Review population_data.csv for trends"
echo "  2. Check events.log for ecosystem dynamics"
echo "  3. Analyze balance issues and adjust reproduction rates if needed"
echo ""
