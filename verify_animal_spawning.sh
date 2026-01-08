#!/bin/bash
# Animal Spawning Verification Script - Subtask 10-2
# Verifies that animals spawn and behave correctly on Map Generator 2.0 maps

set -e

echo "=== Animal Spawn Verification - Map Generator 2.0 ==="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Change to project directory
cd "$(dirname "$0")"

# Step 1: Generate map
echo -e "${BLUE}Step 1: Generating test map...${NC}"
cargo run --bin map_generator generate animal_spawn_test 'Animal Spawn Test' 42424

if [ -f "./saves/animal_spawn_test.ron" ]; then
    echo -e "${GREEN}✓ Map generated successfully${NC}"
    ls -lh ./saves/animal_spawn_test.ron
else
    echo -e "${RED}✗ Map generation failed${NC}"
    exit 1
fi

echo ""

# Step 2: Start simulation in background
echo -e "${BLUE}Step 2: Starting simulation...${NC}"
cargo run --release --bin life-simulator > simulation.log 2>&1 &
SIM_PID=$!
echo "Simulation PID: $SIM_PID"

# Wait for startup
echo "Waiting for simulation to start (10 seconds)..."
sleep 10

# Check if simulation is still running
if ! kill -0 $SIM_PID 2>/dev/null; then
    echo -e "${RED}✗ Simulation crashed during startup${NC}"
    echo "Last 20 lines of log:"
    tail -20 simulation.log
    exit 1
fi

echo -e "${GREEN}✓ Simulation started${NC}"
echo ""

# Step 3: Wait for entity spawning
echo -e "${BLUE}Step 3: Waiting for entity spawning (30 seconds)...${NC}"
sleep 30

# Step 4: Check entity count
echo -e "${BLUE}Step 4: Checking entity count via API...${NC}"
ENTITY_COUNT=$(curl -s http://localhost:54321/api/entities | jq -r '.total // 0')

if [ "$ENTITY_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓ Entities spawned: $ENTITY_COUNT${NC}"
else
    echo -e "${RED}✗ No entities spawned (count: $ENTITY_COUNT)${NC}"
    echo "Check simulation log for errors:"
    tail -50 simulation.log
    kill $SIM_PID 2>/dev/null || true
    exit 1
fi

echo ""

# Step 5: Verify entity positions
echo -e "${BLUE}Step 5: Sampling entity positions...${NC}"
curl -s http://localhost:54321/api/entities | jq '.entities[0:5] | .[] | {species, position}' > entity_positions.json
echo "Sample entity positions saved to entity_positions.json"
cat entity_positions.json

echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}MANUAL VERIFICATION REQUIRED:${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo "1. Open http://localhost:54321/viewer.html in your browser"
echo "2. Navigate to the entity positions listed above"
echo "3. Verify entities are on LAND (grass/forest/dirt/sand)"
echo "4. Verify entities are NOT in WATER (deep or shallow)"
echo "5. Observe entity behavior for 2-3 minutes:"
echo "   - Animals should move around"
echo "   - Herbivores should eat vegetation"
echo "   - Animals should drink from water edges"
echo "   - Animals should NOT walk into deep water"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
read -p "Press ENTER when manual verification is complete..."

echo ""

# Step 6: Wait and check population growth
echo -e "${BLUE}Step 6: Monitoring population growth (60 seconds)...${NC}"
INITIAL_COUNT=$ENTITY_COUNT
sleep 60
FINAL_COUNT=$(curl -s http://localhost:54321/api/entities | jq -r '.total // 0')

echo "Initial count: $INITIAL_COUNT"
echo "Final count: $FINAL_COUNT"

if [ "$FINAL_COUNT" -ge "$INITIAL_COUNT" ]; then
    echo -e "${GREEN}✓ Population stable or growing${NC}"
else
    echo -e "${YELLOW}⚠ Population declining (may be normal if predation active)${NC}"
fi

echo ""

# Step 7: Species diversity
echo -e "${BLUE}Step 7: Checking species diversity...${NC}"
curl -s http://localhost:54321/api/entities | jq '{
  total: .total,
  species: (.entities | group_by(.species) | map({species: .[0].species, count: length}))
}' > population_stats.json

echo "Population statistics saved to population_stats.json"
cat population_stats.json

echo ""

# Step 8: Check for spawn errors in logs
echo -e "${BLUE}Step 8: Checking for spawn errors in simulation log...${NC}"
if grep -i "spawn.*error\|spawn.*fail\|invalid.*spawn" simulation.log > /dev/null; then
    echo -e "${RED}⚠ Found potential spawn errors in log:${NC}"
    grep -i "spawn.*error\|spawn.*fail\|invalid.*spawn" simulation.log | tail -10
else
    echo -e "${GREEN}✓ No spawn errors detected${NC}"
fi

echo ""

# Cleanup
echo "Stopping simulation (PID: $SIM_PID)..."
kill $SIM_PID 2>/dev/null || true
sleep 2

echo ""
echo "=== VERIFICATION SUMMARY ==="
echo -e "${GREEN}✓ Map generation: Success${NC}"
echo -e "${GREEN}✓ Simulation startup: Success${NC}"
echo -e "${GREEN}✓ Entity spawning: $ENTITY_COUNT entities${NC}"
echo -e "${GREEN}✓ Population monitoring: $INITIAL_COUNT → $FINAL_COUNT${NC}"
echo ""
echo "Files created:"
echo "  - entity_positions.json (sample entity locations)"
echo "  - population_stats.json (species distribution)"
echo "  - simulation.log (full simulation output)"
echo ""
echo -e "${YELLOW}FINAL CHECKLIST:${NC}"
echo "  [ ] Entities spawn within 30 seconds"
echo "  [ ] All entities are on walkable terrain (verified in viewer)"
echo "  [ ] No entities spawn in water (verified in viewer)"
echo "  [ ] Animals move around normally"
echo "  [ ] Animals interact with resources (eat/drink)"
echo "  [ ] Population is stable or growing"
echo "  [ ] Multiple species present"
echo "  [ ] No spawn errors in simulation.log"
echo ""
echo "If all checks pass, mark subtask-10-2 as COMPLETED"
echo ""
