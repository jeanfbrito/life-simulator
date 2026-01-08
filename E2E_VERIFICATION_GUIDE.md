# End-to-End Verification Guide - Map Generator 2.0

## Overview
This guide provides step-by-step instructions to verify the Map Generator 2.0 implementation works correctly end-to-end.

## Prerequisites
- Rust toolchain installed
- Project built successfully
- Terminal access to project root directory

## Verification Steps

### Step 1: Generate Test Map

Run the map generator with the test seed:

```bash
cd /Users/jean/Github/life-simulator/.worktrees/001-map-generator-2-0

cargo run --bin map_generator generate mapgen2_test 'MapGen2 Test' 12345
```

**Expected Output:**
- Map generation completes without errors
- File created: `saves/mapgen2_test.ron`
- Console shows terrain statistics (if verbose mode enabled)

**Verification Points:**
- ✅ Map file created in saves/ directory
- ✅ File size is reasonable (100KB - 200KB)
- ✅ No error messages or panics

### Step 2: Start Simulation Server

Start the life simulator with the generated map:

```bash
cargo run --release --bin life-simulator
```

**Expected Output:**
- Server starts on port 54321
- Console shows "Listening on http://127.0.0.1:54321"
- No errors loading the map
- Entities begin spawning

**Verification Points:**
- ✅ Server starts without errors
- ✅ Map loads successfully
- ✅ API responds at http://localhost:54321/api/entities

### Step 3: Open Web Viewer

Open the web viewer in your browser:

```
http://localhost:54321/viewer.html
```

**Expected Output:**
- Viewer loads and displays the map
- Terrain tiles render correctly
- Camera controls work (pan, zoom)

**Verification Points:**
- ✅ Map renders without visual errors
- ✅ Water tiles visible around perimeter
- ✅ Land tiles (grass/forest) in interior
- ✅ Entities spawn and move on walkable terrain

### Step 4: Verify Map Quality Criteria

Inspect the generated map visually and via API:

#### Visual Verification (in viewer)
1. **Perimeter Boundary:**
   - Outermost edge should be deep water (dark blue)
   - Next layer should be shallow water (lighter blue)
   - Next layer should be sand (beige/tan)
   - Then grass or forest begins

2. **Interior Water Bodies:**
   - Water spots should be distributed across the map
   - No abrupt deep water → land transitions
   - Shallow water buffer visible around lakes

3. **Terrain Distribution:**
   - Majority of land should be grass (green) or forest (dark green)
   - Minimal rocky/mountain terrain
   - Good balance of open areas and forested areas

#### API Verification

Check entity spawning:
```bash
curl http://localhost:54321/api/entities | jq '.total'
```

**Expected:** Number should increase over time as entities spawn.

Check that entities are on walkable terrain:
```bash
curl http://localhost:54321/api/entities | jq '.entities[0:5] | .[] | {species, position}'
```

**Expected:** Entity positions should be on land tiles, not in water.

### Step 5: Test Map Regeneration with Different Seeds

Generate additional maps to verify consistency:

```bash
# Test with different seed
cargo run --bin map_generator generate mapgen2_test2 'Test Map 2' 67890

# Test with verbose output
cargo run --bin map_generator generate mapgen2_verbose 'Verbose Test' 11111 --verbose

# Test with custom density (if implemented)
cargo run --bin map_generator generate mapgen2_custom 'Custom Density' 22222 --water-density 0.2 --forest-density 0.5
```

**Verification Points:**
- ✅ All maps generate successfully
- ✅ Different seeds produce different layouts
- ✅ All maps meet quality criteria (perimeter boundaries, land coverage)
- ✅ Verbose output shows boundary rules and validation stats

## Success Criteria Checklist

Map must satisfy ALL of these criteria:

- [ ] **Perimeter Boundaries:** Deep water → shallow water → sand at all edges
- [ ] **Interior Water Transitions:** Deep water → shallow water buffer (no abrupt deep→land)
- [ ] **Land Coverage:** ≥ 60% of total map area is walkable land
- [ ] **Green Coverage:** ≥ 50% of land tiles are grass or forest
- [ ] **Water Distribution:** Water spots appear in all quadrants
- [ ] **Entity Spawning:** Animals spawn only on walkable terrain
- [ ] **Foraging Resources:** Berry bushes and fruits spawn in appropriate biomes
- [ ] **No Errors:** Map generation, simulation startup, and viewer all work without errors

## Troubleshooting

### Map Generation Fails
- Check that saves/ directory exists and is writable
- Verify seed is a valid number
- Check console for error messages

### Simulation Won't Start
- Ensure only one instance is running (port 54321 not already in use)
- Check that map file exists in saves/ directory
- Look for errors in console output

### Viewer Shows Blank/Broken Map
- Verify server is running and API is accessible
- Check browser console for JavaScript errors
- Try refreshing the page (Ctrl+R or Cmd+R)
- Ensure map file is not corrupted

### Entities Not Spawning
- Wait 30-60 seconds for initial spawn cycle
- Check API: `curl http://localhost:54321/api/entities`
- Verify map has sufficient walkable terrain

## Performance Benchmarks

Expected performance characteristics:

- **Map Generation Time:** 2-5 seconds for standard map (depends on world size)
- **Simulation Startup:** < 5 seconds to load map and initialize
- **Viewer Load Time:** < 2 seconds to render initial view
- **Entity Spawn Rate:** 5-10 entities per minute (configurable)

## Comparison to Pre-Implementation State

To compare with the snapshot state before Map Generator 2.0:

```bash
# Switch to pre-implementation tag
git checkout pre-mapgen2.0

# Generate map with old system
cargo run --bin map_generator generate old_map 'Old Map' 12345

# Start simulation
cargo run --release --bin life-simulator

# Compare in viewer - old map should:
# - Have less consistent boundaries
# - Possibly more water/less green coverage
# - Less strategic water placement

# Return to current implementation
git checkout -
```

## Automated Verification (Future)

For CI/CD integration, consider adding:

```bash
#!/bin/bash
# e2e_test.sh

# Generate test map
cargo run --bin map_generator generate e2e_test 'E2E Test' 12345
if [ ! -f "saves/e2e_test.ron" ]; then
    echo "ERROR: Map generation failed"
    exit 1
fi

# Start simulation in background
cargo run --release --bin life-simulator &
SIM_PID=$!
sleep 10  # Wait for startup

# Test API
ENTITY_COUNT=$(curl -s http://localhost:54321/api/entities | jq '.total')
if [ -z "$ENTITY_COUNT" ]; then
    echo "ERROR: API not responding"
    kill $SIM_PID
    exit 1
fi

echo "SUCCESS: E2E test passed"
kill $SIM_PID
exit 0
```

## Verification Status

**Implementation Complete:** ✅ All phases 1-9 completed
**Cargo Restriction:** ⚠️ Direct cargo execution blocked in auto-claude environment
**Manual Verification Required:** ✅ This guide provides complete verification procedure

To complete verification, the user should:
1. Run the commands in this guide in a standard terminal
2. Verify all success criteria are met
3. Confirm no regressions from pre-implementation snapshot

## Related Files

- Implementation Plan: `./.auto-claude/specs/001-map-generator-2-0/implementation_plan.json`
- Build Progress: `./.auto-claude/specs/001-map-generator-2-0/build-progress.txt`
- Specification: `./.auto-claude/specs/001-map-generator-2-0/spec.md`
