#!/bin/bash
# Debugging script for life simulator
# Monitors entity behaviors, actions, and vital stats

set -e

echo "🔧 Life Simulator Debug Monitor"
echo "================================"

# Kill any existing simulation
pkill -f life-simulator || true
sleep 1

# Start simulation in background with logging
echo "🚀 Starting simulation..."
RUST_LOG=info,life_simulator::ai=debug,life_simulator::entities=debug cargo run --bin life-simulator > /tmp/sim.log 2>&1 &
SIM_PID=$!

echo "📊 Simulation PID: $SIM_PID"
echo "📝 Log file: /tmp/sim.log"
echo ""
echo "⏱️  Waiting for simulation to initialize..."
sleep 5

# Monitor function
monitor() {
    echo ""
    echo "=== ENTITY STATUS ==="
    curl -s http://127.0.0.1:54321/api/entities | jq -r '.entities[] | "Entity: \(.id) | Species: \(.species) | Pos: (\(.x),\(.y)) | Action: \(.current_action // "none") | Hunger: \(.hunger)% | Thirst: \(.thirst)% | Energy: \(.energy)%"' 2>/dev/null || echo "No entities yet"

    echo ""
    echo "=== RECENT LOGS (Actions) ==="
    grep -E "(💧|🐇|😴|🎯|Entity.*action)" /tmp/sim.log | tail -10

    echo ""
    echo "=== PATHFINDING STATUS ==="
    grep -E "(pathfinding|Path|NeedPath)" /tmp/sim.log | tail -5
}

# Monitor loop
echo "📡 Starting monitoring (Ctrl+C to stop)..."
for i in {1..30}; do
    sleep 5
    echo ""
    echo "╔═══════════════════════════════════════════╗"
    echo "║  Check $i/30 ($(date +%H:%M:%S))                  ║"
    echo "╚═══════════════════════════════════════════╝"
    monitor
done

# Cleanup
echo ""
echo "🛑 Stopping simulation..."
kill $SIM_PID 2>/dev/null || true

echo ""
echo "✅ Debug session complete. Full log at /tmp/sim.log"
