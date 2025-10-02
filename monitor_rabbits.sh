#!/bin/bash

echo "🐇 Monitoring rabbit drinking behavior..."
echo "Running simulation and watching for drinking actions..."
echo ""

cd /Users/jean/Github/life-simulator

# Run simulation and watch for key events
cargo run --bin life-simulator 2>&1 | \
    grep --line-buffered -E "(Spawned rabbit|Entity.*Thirst|drank water|DrinkWater|utility|queuing action)" | \
    head -50

echo ""
echo "✅ Test complete! Check above for drinking events."
