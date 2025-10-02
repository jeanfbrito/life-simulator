#!/bin/bash

echo "Running simulation for 30 seconds..."
cd /Users/jean/Github/life-simulator

# Run the simulation and capture logs related to drinking
cargo run --bin life-simulator 2>&1 | grep -E "(Thirst|Entity.*at|drink|water|utility|action|🐇|🧠)" | head -100 &

# Store the PID
SIM_PID=$!

# Wait 30 seconds
sleep 30

# Kill the simulation
kill $SIM_PID 2>/dev/null

echo "Test complete!"
