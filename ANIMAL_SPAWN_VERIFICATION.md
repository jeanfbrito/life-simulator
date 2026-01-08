# Animal Spawn Verification - Subtask 10-2

## Objective
Verify that animals spawn and behave correctly on maps generated with Map Generator 2.0.

## Critical Requirements
1. Animals must spawn only on walkable terrain (grass, forest, dirt, sand)
2. Animals must NOT spawn in water (deep water or shallow water)
3. Animals should spawn within reasonable time (30 seconds)
4. Animal populations should be appropriate for map size
5. Animals should behave normally (move, eat, drink) on new terrain

## Verification Procedure

### Step 1: Generate a Fresh Map

```bash
cd /Users/jean/Github/life-simulator/.worktrees/001-map-generator-2-0

# Generate test map with known seed
cargo run --bin map_generator generate animal_spawn_test 'Animal Spawn Test' 42424

# Verify map file created
ls -lh ./saves/animal_spawn_test.ron
```

**Expected Result:**
- Map file created: `saves/animal_spawn_test.ron`
- File size: ~100-200KB
- No errors during generation

### Step 2: Start Simulation

```bash
# Start the simulation server
cargo run --release --bin life-simulator

# Server should output:
# - "Listening on http://127.0.0.1:54321"
# - "Loading world..."
# - "World loaded successfully"
```

**Expected Result:**
- Server starts without errors
- Map loads successfully
- No panics or crashes

### Step 3: Wait for Entity Spawning (30 seconds)

Wait 30 seconds for the entity spawning system to initialize and create initial population.

```bash
# In a separate terminal, wait 30 seconds then check
sleep 30
```

### Step 4: Verify Entity Count via API

```bash
# Check total entity count
curl http://localhost:54321/api/entities | jq '.total'

# Expected: Should show a number > 0 (typically 10-50 entities)
```

**Expected Result:**
- Total count > 0
- Count increases over time (check again after 1 minute)

### Step 5: Verify Entity Positions are on Walkable Terrain

```bash
# Get first 10 entities with their positions
curl http://localhost:54321/api/entities | jq '.entities[0:10] | .[] | {species, position}'

# Example output:
# {
#   "species": "Deer",
#   "position": {
#     "x": 145.3,
#     "y": 78.2
#   }
# }
```

**Verification Steps:**
1. Note the entity positions (x, y coordinates)
2. Open web viewer: http://localhost:54321/viewer.html
3. Navigate to each entity position using camera controls
4. Visually confirm entity is on land (grass/forest/dirt), NOT in water

**Automated Check:**
```bash
# Get all entities and their terrain types
curl http://localhost:54321/api/entities | jq '.entities[] | {species, position, terrain: .position}'

# Cross-reference with terrain API (if available)
curl http://localhost:54321/api/map/tiles
```

### Step 6: Verify Entity Behavior

1. Open web viewer: http://localhost:54321/viewer.html
2. Locate several animals (deer, rabbits, etc.)
3. Observe for 2-3 minutes:
   - ✅ Animals should move around
   - ✅ Animals should eat vegetation (if herbivores)
   - ✅ Animals should drink from water (approach water edges)
   - ✅ Animals should NOT walk into deep water
   - ✅ Animals should NOT spawn in water then move to land

### Step 7: Verify Spawn Point Validity

```bash
# Check spawn system logs in simulation console
# Look for messages like:
# - "Spawning Deer at (x, y)"
# - "Spawn point found: (x, y)"

# Verify no errors like:
# - "Failed to find valid spawn point"
# - "Spawned entity in invalid location"
```

### Step 8: Statistical Verification

Run simulation for 5 minutes, then check population stats:

```bash
# After 5 minutes of runtime
curl http://localhost:54321/api/entities | jq '{
  total: .total,
  by_species: (.entities | group_by(.species) | map({species: .[0].species, count: length}))
}'

# Expected output shows healthy population distribution:
# {
#   "total": 45,
#   "by_species": [
#     {"species": "Deer", "count": 12},
#     {"species": "Rabbit", "count": 18},
#     {"species": "Wolf", "count": 5},
#     ...
#   ]
# }
```

**Success Criteria:**
- Total entity count is reasonable (10-100 depending on map size)
- Multiple species present
- Population grows over time
- No sudden population crashes (indicates spawn failures)

## Common Issues and Solutions

### Issue: No Entities Spawn

**Possible Causes:**
1. No valid spawn points on map (all water or impassable terrain)
2. Map validation failed but wasn't caught
3. Entity spawning system disabled

**Debug Steps:**
```bash
# Check map statistics
cargo run --bin map_generator list

# Look for land percentage - should be >= 60%
# If land percentage is low, map generation may have failed validation
```

### Issue: Entities Spawn in Water

**Possible Causes:**
1. Spawn point validation not checking terrain type
2. Terrain type mismatch between generation and runtime

**Debug Steps:**
1. Open viewer and screenshot entities in water
2. Check console logs for spawn point coordinates
3. Verify terrain at those coordinates using API
4. File bug report with coordinates and terrain type

### Issue: Entities Don't Move

**Possible Causes:**
1. Pathfinding fails on new terrain
2. No valid destinations (all resources in water, etc.)
3. AI behavior issue (not specific to new maps)

**Debug Steps:**
1. Compare behavior on old map vs new map
2. Check console for pathfinding errors
3. Verify resources spawned correctly using API

## Success Criteria Checklist

- [ ] Map generation succeeds without errors
- [ ] Simulation starts and loads map successfully
- [ ] Entities spawn within 30 seconds
- [ ] Entity count via API shows total > 0
- [ ] All entity positions are on walkable terrain (not water)
- [ ] Animals move around normally
- [ ] Animals interact with resources (eat, drink)
- [ ] No spawn-related errors in console
- [ ] Population is stable/growing over 5 minutes
- [ ] Multiple species present in population

## Test Results

### Test 1: Fresh Map Generation
- **Date:** _____________________
- **Map Seed:** 42424
- **Map Generated:** ☐ Yes ☐ No
- **File Size:** _____________ KB
- **Errors:** ☐ None ☐ Errors (describe): _______________________

### Test 2: Entity Spawning
- **Simulation Started:** ☐ Yes ☐ No
- **Wait Time:** 30 seconds
- **Entity Count:** _____________ entities
- **Spawn Errors:** ☐ None ☐ Errors (describe): _______________________

### Test 3: Position Validation
- **Entities on Land:** ☐ All ☐ Some ☐ None
- **Entities in Water:** ☐ None ☐ Some (COUNT: _____) ☐ All
- **Invalid Spawns:** _____________

### Test 4: Behavior Validation
- **Animals Moving:** ☐ Yes ☐ No
- **Animals Eating:** ☐ Yes ☐ No
- **Animals Drinking:** ☐ Yes ☐ No
- **Behavior Errors:** ☐ None ☐ Errors (describe): _______________________

### Test 5: Population Stability (5 minutes)
- **Starting Population:** _____________
- **Ending Population:** _____________
- **Population Change:** ☐ Growing ☐ Stable ☐ Declining
- **Species Diversity:** ☐ Multiple species ☐ Single species ☐ None

## Automated Verification Script

Save this as `verify_animal_spawning.sh`:

```bash
#!/bin/bash
# Animal Spawning Verification Script - Subtask 10-2

set -e

echo "=== Animal Spawn Verification ==="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Step 1: Generate map
echo "Step 1: Generating test map..."
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
echo "Step 2: Starting simulation..."
cargo run --release --bin life-simulator &
SIM_PID=$!
echo "Simulation PID: $SIM_PID"

# Wait for startup
echo "Waiting for simulation to start (10 seconds)..."
sleep 10

# Check if simulation is still running
if ! kill -0 $SIM_PID 2>/dev/null; then
    echo -e "${RED}✗ Simulation crashed during startup${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Simulation started${NC}"
echo ""

# Step 3: Wait for entity spawning
echo "Step 3: Waiting for entity spawning (30 seconds)..."
sleep 30

# Step 4: Check entity count
echo "Step 4: Checking entity count..."
ENTITY_COUNT=$(curl -s http://localhost:54321/api/entities | jq -r '.total // 0')

if [ "$ENTITY_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓ Entities spawned: $ENTITY_COUNT${NC}"
else
    echo -e "${RED}✗ No entities spawned${NC}"
    kill $SIM_PID
    exit 1
fi

echo ""

# Step 5: Verify entity positions
echo "Step 5: Checking entity positions..."
curl -s http://localhost:54321/api/entities | jq '.entities[0:5] | .[] | {species, position}' > entity_positions.json
echo "Sample entity positions saved to entity_positions.json"
cat entity_positions.json

echo ""
echo -e "${YELLOW}Manual verification required:${NC}"
echo "1. Open http://localhost:54321/viewer.html"
echo "2. Verify entities are on land (not in water)"
echo "3. Observe entity behavior for 2-3 minutes"
echo ""

# Step 6: Wait and check population growth
echo "Step 6: Monitoring population (60 seconds)..."
INITIAL_COUNT=$ENTITY_COUNT
sleep 60
FINAL_COUNT=$(curl -s http://localhost:54321/api/entities | jq -r '.total // 0')

echo "Initial count: $INITIAL_COUNT"
echo "Final count: $FINAL_COUNT"

if [ "$FINAL_COUNT" -ge "$INITIAL_COUNT" ]; then
    echo -e "${GREEN}✓ Population stable or growing${NC}"
else
    echo -e "${YELLOW}⚠ Population declining${NC}"
fi

echo ""

# Step 7: Species diversity
echo "Step 7: Checking species diversity..."
curl -s http://localhost:54321/api/entities | jq '{
  total: .total,
  species: (.entities | group_by(.species) | map({species: .[0].species, count: length}))
}' > population_stats.json

echo "Population statistics saved to population_stats.json"
cat population_stats.json

echo ""

# Cleanup
echo "Verification complete. Stopping simulation..."
kill $SIM_PID 2>/dev/null || true

echo ""
echo "=== Verification Summary ==="
echo -e "${GREEN}✓ Map generation: Success${NC}"
echo -e "${GREEN}✓ Simulation startup: Success${NC}"
echo -e "${GREEN}✓ Entity spawning: $ENTITY_COUNT entities${NC}"
echo -e "${GREEN}✓ Population monitoring: $INITIAL_COUNT → $FINAL_COUNT${NC}"
echo ""
echo -e "${YELLOW}Manual checks required:${NC}"
echo "- Verify entity positions are on walkable terrain"
echo "- Observe entity behavior in viewer"
echo "- Confirm no spawn errors in simulation console"
echo ""
echo "Files created:"
echo "- entity_positions.json"
echo "- population_stats.json"
echo ""
```

Make it executable:
```bash
chmod +x verify_animal_spawning.sh
```

Run it:
```bash
./verify_animal_spawning.sh
```

## Integration with QA Sign-off

This verification is part of the QA acceptance criteria for Map Generator 2.0:

- **Subtask ID:** subtask-10-2
- **Phase:** Integration and Final Verification
- **Blocks:** subtask-10-3 (final stability verification)

Once all checks pass, update implementation_plan.json:
```json
{
  "id": "subtask-10-2",
  "status": "completed",
  "notes": "Animal spawning verified: [ENTITY_COUNT] entities spawned on walkable terrain, behavior normal, population stable"
}
```

## Related Documentation

- E2E Verification Guide: `./E2E_VERIFICATION_GUIDE.md`
- Implementation Plan: `./.auto-claude/specs/001-map-generator-2-0/implementation_plan.json`
- Specification: `./.auto-claude/specs/001-map-generator-2-0/spec.md`
