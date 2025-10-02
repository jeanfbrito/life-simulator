#!/bin/bash

# Test actual movement timing of humans

echo "🕐 Starting movement timing test..."
echo "Monitoring Human_0 position every 2 seconds for 30 seconds..."
echo ""

for i in {1..15}; do
    timestamp=$(date +"%H:%M:%S")
    position=$(curl -s http://127.0.0.1:54321/api/entities | jq -r '.entities[] | select(.name == "Human_0") | "\(.position.x),\(.position.y)"')
    echo "[$timestamp] (${i}x2s = $((i*2))s) Human_0 at: $position"
    sleep 2
done

echo ""
echo "✅ Test complete! Check if position changed after ~10 seconds"
