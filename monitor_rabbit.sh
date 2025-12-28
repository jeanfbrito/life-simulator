#!/bin/bash
# Monitor single rabbit behavior

for i in {1..15}; do
  echo ""
  echo "=== Check $i/15 ($(date +%H:%M:%S)) ==="
  curl -s http://127.0.0.1:54321/api/entities | jq -r '.entities[] | "🐇 \(.name) @ (\(.position.x),\(.position.y)) | Action: \(.current_action) | Hunger:\(.hunger)% Thirst:\(.thirst)% Energy:\(.energy)%"'

  # Check recent action logs
  echo "Recent actions:"
  grep -E "(💧|🐇|😴|Entity.*action)" /tmp/sim_test.log | tail -3

  sleep 3
done
