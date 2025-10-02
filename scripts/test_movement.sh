#!/bin/bash
# Movement verification test script
# Tests the tick-based movement system by tracking entity positions over time

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=== Life Simulator Movement Test ===${NC}"
echo ""

# Kill any existing simulation
echo "Cleaning up existing processes..."
pkill -f "target/debug/life-simulator" 2>/dev/null || true
pkill -f "target/release/life-simulator" 2>/dev/null || true
sleep 1

# Build the project
echo "Building project..."
cargo build --quiet 2>&1 | head -n 20 || true

# Start the simulation in background
echo "Starting simulation..."
cargo run --bin life-simulator > /tmp/life-simulator.log 2>&1 &
SIM_PID=$!

# Wait for startup
echo "Waiting for simulation to start..."
sleep 5

# Check if simulation is running
if ! ps -p $SIM_PID > /dev/null 2>&1; then
    echo -e "${RED}❌ Simulation failed to start!${NC}"
    echo "Check logs at: /tmp/life-simulator.log"
    tail -n 20 /tmp/life-simulator.log
    exit 1
fi

# Check if API is responding
echo "Checking API..."
if ! curl -s -f http://127.0.0.1:54321/api/entities > /dev/null; then
    echo -e "${RED}❌ API not responding!${NC}"
    echo "Check logs at: /tmp/life-simulator.log"
    kill $SIM_PID 2>/dev/null || true
    exit 1
fi

echo -e "${GREEN}✓ Simulation started successfully${NC}"
echo ""

# Track entity movement
echo -e "${YELLOW}Tracking Human_0 movement for 30 seconds...${NC}"
echo ""
printf "%-8s | %-8s | %-12s | %s\n" "Time" "Elapsed" "Position" "Notes"
echo "---------|----------|--------------|---------------------------"

# Store positions to detect movement
declare -a positions
movement_count=0
last_position=""

for i in {1..15}; do
    elapsed=$((i * 2))
    timestamp=$(date +"%H:%M:%S")
    
    # Get position from API
    position=$(curl -s http://127.0.0.1:54321/api/entities 2>/dev/null | \
               jq -r '.entities[] | select(.name == "Human_0") | "\(.position.x),\(.position.y)"' 2>/dev/null)
    
    if [ -z "$position" ]; then
        position="N/A"
        note="${RED}No data${NC}"
    else
        positions+=("$position")
        
        # Check if position changed
        if [ -n "$last_position" ] && [ "$position" != "$last_position" ]; then
            movement_count=$((movement_count + 1))
            note="${GREEN}Moved!${NC}"
        else
            note=""
        fi
        
        last_position="$position"
    fi
    
    printf "%-8s | %5ds   | %-12s | %s\n" "$timestamp" "$elapsed" "$position" "$note"
    
    sleep 2
done

echo ""
echo -e "${YELLOW}=== Test Results ===${NC}"
echo ""

# Analysis
if [ ${#positions[@]} -eq 0 ]; then
    echo -e "${RED}❌ FAILED: No position data collected${NC}"
    echo "   The API might not be working correctly."
    result=1
elif [ $movement_count -eq 0 ]; then
    echo -e "${RED}❌ FAILED: No movement detected${NC}"
    echo "   Entities are not moving. Possible issues:"
    echo "   - Simulation might be paused"
    echo "   - Tick system not working"
    echo "   - Movement speed too slow"
    echo "   - Wandering AI not functioning"
    result=1
else
    echo -e "${GREEN}✅ SUCCESS: Movement detected!${NC}"
    echo "   - Total movements: $movement_count"
    echo "   - Average movement interval: $((30 / movement_count))s"
    echo "   - Expected interval: ~3s (with 30 ticks per tile at 10 TPS)"
    echo ""
    
    # Check if movement rate is reasonable
    if [ $movement_count -ge 6 ]; then
        echo -e "${GREEN}   Movement rate looks good!${NC}"
        result=0
    elif [ $movement_count -ge 3 ]; then
        echo -e "${YELLOW}   Movement rate is slower than expected.${NC}"
        echo "   This might be normal due to AI decision making."
        result=0
    else
        echo -e "${YELLOW}   Movement rate is quite slow.${NC}"
        echo "   Consider checking movement speed configuration."
        result=0
    fi
fi

echo ""
echo "Simulation log: /tmp/life-simulator.log"
echo ""

# Cleanup
echo "Stopping simulation..."
kill $SIM_PID 2>/dev/null || true
sleep 1

# Show recent tick info from logs
echo -e "${YELLOW}Recent tick activity:${NC}"
grep -E "(Tick #|wanderer|movement)" /tmp/life-simulator.log | tail -n 10 || echo "No tick log entries found"

echo ""
if [ $result -eq 0 ]; then
    echo -e "${GREEN}=== Test Passed ===${NC}"
else
    echo -e "${RED}=== Test Failed ===${NC}"
    echo "Review the output above and check /tmp/life-simulator.log for details"
fi

exit $result
