#!/bin/bash

# Test script to verify predator fear system works
# This script spawns a wolf and rabbits to observe fear-based behavior modification

echo "🔬 Testing Predator Fear System"
echo "================================"
echo ""

# Clean up any existing processes
pkill -f "target/debug/life-simulator" 2>/dev/null || true
sleep 2

# Start the simulation with debug logging
echo "🚀 Starting life simulator with debug logging..."
RUST_LOG=debug cargo run --bin life-simulator > /tmp/fear_test.log 2>&1 &
SIMULATOR_PID=$!

# Wait for startup
sleep 5

echo "✅ Simulator started (PID: $SIMULATOR_PID)"
echo ""

# Check initial world info
echo "📋 Checking world status..."
curl -s http://127.0.0.1:54321/api/world_info | jq '.' > /tmp/world_info.json
echo "World loaded successfully"

# Get initial entities
echo "🐰 Checking initial entities..."
curl -s http://127.0.0.1:54321/api/entities | jq '.entities | length' > /tmp/initial_count.txt
INITIAL_COUNT=$(cat /tmp/initial_count.txt)
echo "Initial entity count: $INITIAL_COUNT"

# Spawn a wolf (predator)
echo "🐺 Spawning wolf at position (10, 10)..."
curl -s -X POST http://127.0.0.1:54321/api/spawn \
  -H "Content-Type: application/json" \
  -d '{
    "entity_type": "Wolf",
    "name": "TestWolf",
    "position": {"x": 10, "y": 10}
  }' > /tmp/wolf_spawn.json

# Spawn some rabbits (prey)
echo "🐇 Spawning rabbits near the wolf..."
curl -s -X POST http://127.0.0.1:54321/api/spawn \
  -H "Content-Type: application/json" \
  -d '{
    "entity_type": "Rabbit",
    "name": "ScaredRabbit1",
    "position": {"x": 15, "y": 15}
  }' > /tmp/rabbit1_spawn.json

curl -s -X POST http://127.0.0.1:54321/api/spawn \
  -H "Content-Type: application/json" \
  -d '{
    "entity_type": "Rabbit",
    "name": "ScaredRabbit2",
    "position": {"x": 25, "y": 12}
  }' > /tmp/rabbit2_spawn.json

curl -s -X POST http://127.0.0.1:54321/api/spawn \
  -H "Content-Type: application/json" \
  -d '{
    "entity_type": "Rabbit",
    "name": "SafeRabbit",
    "position": {"x": 50, "y": 50}
  }' > /tmp/rabbit3_spawn.json

# Wait for entities to be processed
sleep 3

# Check current entities
echo "📊 Current entity count after spawning..."
curl -s http://127.0.0.1:54321/api/entities | jq '.entities | length'

# Monitor fear behavior for 30 seconds
echo ""
echo "👀 Monitoring fear behavior for 30 seconds..."
echo "Distance calculations:"
echo "- Wolf at (10,10)"
echo "- ScaredRabbit1 at (15,15): distance = $((echo "sqrt((15-10)^2 + (15-10)^2)" | bc)) ≈ 7.1 tiles (WITHIN fear radius)"
echo "- ScaredRabbit2 at (25,12): distance = $((echo "sqrt((25-10)^2 + (12-10)^2)" | bc)) ≈ 15.1 tiles (WITHIN fear radius)"
echo "- SafeRabbit at (50,50): distance = $((echo "sqrt((50-10)^2 + (50-10)^2)" | bc)) ≈ 56.6 tiles (OUTSIDE fear radius)"
echo ""

for i in {1..6}; do
    echo "=== Check $i/6 (5 second intervals) ==="

    # Get entity positions
    ENTITIES_JSON=$(curl -s http://127.0.0.1:54321/api/entities)

    echo "Wolf position:"
    echo "$ENTITIES_JSON" | jq -r '.entities[] | select(.name == "TestWolf") | "  \(.name): (\(.position.x), \(.position.y))"'

    echo "Rabbit positions:"
    echo "$ENTITIES_JSON" | jq -r '.entities[] | select(.name | startswith("ScaredRabbit") or .name == "SafeRabbit") | "  \(.name): (\(.position.x), \(.position.y))"'

    echo ""

    # Check for fear-related logs
    echo "🦊 Fear-related logs (last 5 seconds):"
    tail -n 50 /tmp/fear_test.log | grep -E "(🦊|🏃|fear|predator)" || echo "  No fear logs detected in this interval"

    echo "----------------------------------------"
    sleep 5
done

echo ""
echo "📈 Test Summary:"
echo "- Wolf spawned at (10,10)"
echo "- 3 rabbits spawned at different distances"
echo "- 2 rabbits within fear radius (20 tiles)"
echo "- 1 rabbit outside fear radius"
echo ""
echo "Expected behavior:"
echo "- Rabbits within fear radius should show reduced feeding utility"
echo "- Rabbits within fear radius should move faster when fearful"
echo "- Safe rabbit should behave normally"
echo ""

# Clean up
echo "🧹 Cleaning up..."
kill $SIMULATOR_PID 2>/dev/null || true
sleep 2

echo "✅ Predator fear test completed!"
echo ""
echo "📝 To analyze detailed behavior, check:"
echo "- /tmp/fear_test.log (full simulation logs)"
echo "- Look for 🦊, 🏃 emojis indicating fear detection and speed boosts"
echo "- Check for 'Fear modified action utility' messages"
echo "- Look for 'Entity X detects predator' messages"