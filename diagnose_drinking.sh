#!/bin/bash
# Diagnostic script to understand why rabbits aren't drinking water
# despite being near water sources

echo "=== Rabbit Drinking Diagnostic ==="
echo ""
echo "This will monitor:"
echo "1. Rabbit positions and thirst levels"
echo "2. AI planning decisions (what actions are evaluated)"
echo "3. Action queue status (what's being executed)"
echo "4. Pathfinding requests and results"
echo "5. Water drinking events"
echo ""
echo "Starting simulation with detailed logging..."
echo ""

# Kill any existing simulation
pkill -f "target/debug/life-simulator" 2>/dev/null || true
sleep 1

# Start simulation with detailed logging
RUST_LOG=info cargo run --bin life-simulator 2>&1 | tee /tmp/rabbit_diagnosis.log &
SIM_PID=$!

sleep 5  # Wait for startup

echo "Simulation started. Monitoring for 30 seconds..."
echo ""

# Function to get entity data
get_entity_data() {
    curl -s http://127.0.0.1:54321/api/entities 2>/dev/null | jq -r '.entities[] | select(.name | startswith("Rabbit")) | "\(.name): pos=(\(.position.x),\(.position.y)) thirst=\(.thirst)%"' 2>/dev/null || echo "API not ready"
}

# Monitor entity positions and thirst every 3 seconds
echo "=== Entity Status Monitor ==="
for i in {1..10}; do
    timestamp=$(date +"%H:%M:%S")
    echo "[$timestamp]"
    get_entity_data
    echo ""
    sleep 3
done

echo ""
echo "=== Analyzing Logs ==="
echo ""

# Check if rabbits are spawning
echo "1. Rabbit Spawning:"
grep -i "spawned.*rabbit" /tmp/rabbit_diagnosis.log | tail -5 || echo "   ❌ No rabbit spawns found"
echo ""

# Check AI planning
echo "2. AI Planning (last 10 actions):"
grep "🧠.*Rabbit.*Evaluated.*actions" /tmp/rabbit_diagnosis.log | tail -10 || echo "   ❌ No AI planning logs found"
echo ""

# Check what actions are being evaluated
echo "3. Drink Water Actions Evaluated:"
grep "DrinkWater.*utility" /tmp/rabbit_diagnosis.log | tail -10 || echo "   ❌ No DrinkWater actions evaluated"
echo ""

# Check what actions are being queued
echo "4. Actions Queued:"
grep "✅.*queuing action" /tmp/rabbit_diagnosis.log | tail -10 || echo "   ❌ No actions queued"
echo ""

# Check path requests
echo "5. Pathfinding Requests:"
grep "PATH.*request" /tmp/rabbit_diagnosis.log | tail -10 || echo "   ❌ No pathfinding requests"
echo ""

# Check pathfinding failures
echo "6. Pathfinding Failures:"
grep "PATH.*FAIL\|Pathfinding failed" /tmp/rabbit_diagnosis.log | tail -10 || echo "   ✅ No pathfinding failures (good!)"
echo ""

# Check for actual drinking events
echo "7. Water Drinking Events:"
grep "drank water\|drinking water" /tmp/rabbit_diagnosis.log | tail -10 || echo "   ❌ No drinking events found"
echo ""

# Check action execution
echo "8. Action Execution:"
grep "executing action\|InProgress\|Success\|Failed" /tmp/rabbit_diagnosis.log | tail -15 || echo "   ❌ No action execution logs"
echo ""

# Check wandering
echo "9. Wandering Actions:"
grep "Wander.*utility" /tmp/rabbit_diagnosis.log | tail -10 || echo "   ❌ No wandering actions"
echo ""

# Summary
echo ""
echo "=== DIAGNOSIS SUMMARY ==="
echo ""

# Count key events
planning_count=$(grep -c "🧠.*Rabbit.*Evaluated" /tmp/rabbit_diagnosis.log 2>/dev/null || echo 0)
drink_evaluated=$(grep -c "DrinkWater.*utility" /tmp/rabbit_diagnosis.log 2>/dev/null || echo 0)
drink_queued=$(grep -c "DrinkWater.*queuing" /tmp/rabbit_diagnosis.log 2>/dev/null || echo 0)
drink_executed=$(grep -c "drank water" /tmp/rabbit_diagnosis.log 2>/dev/null || echo 0)
wander_queued=$(grep -c "Wander.*queuing" /tmp/rabbit_diagnosis.log 2>/dev/null || echo 0)

echo "Planning Events:           $planning_count"
echo "DrinkWater Evaluated:      $drink_evaluated times"
echo "DrinkWater Queued:         $drink_queued times"
echo "DrinkWater Executed:       $drink_executed times"
echo "Wander Queued:             $wander_queued times"
echo ""

if [ "$drink_evaluated" -eq 0 ]; then
    echo "🔴 PROBLEM: DrinkWater action never evaluated!"
    echo "   → AI planner may not be running or water not found"
elif [ "$drink_queued" -eq 0 ]; then
    echo "🟡 PROBLEM: DrinkWater evaluated but never queued!"
    echo "   → Utility score may be too low"
    echo "   → Check utility threshold or thirst levels"
elif [ "$drink_executed" -eq 0 ]; then
    echo "🟠 PROBLEM: DrinkWater queued but never executed!"
    echo "   → Action execution system may have issues"
    echo "   → Check action queue processing"
else
    echo "✅ Drinking is working!"
fi

echo ""
echo "Full logs available at: /tmp/rabbit_diagnosis.log"
echo "Hint: grep specific entity: grep 'Entity(0v1)' /tmp/rabbit_diagnosis.log"

# Cleanup
kill $SIM_PID 2>/dev/null
wait $SIM_PID 2>/dev/null
