#!/bin/bash
echo "🧪 FINAL TEST - Monitoring rabbit for 2 minutes"
echo "=============================================="
for i in $(seq 1 60); do
  echo "[$i/60] $(curl -s http://127.0.0.1:54321/api/entities | jq -r '.entities[] | "Pos(\(.position.x),\(.position.y)) \(.current_action) H:\(.hunger)% T:\(.thirst)% E:\(.energy)%"')"
  sleep 2
done
echo ""
echo "✅ Test complete! Checking logs for all behaviors..."
echo "Drinking:" && grep "💧.*drinking" /tmp/sim_test.log | tail -3
echo "Grazing:" && grep "🐇.*grazing\|Entity.*Graze" /tmp/sim_test.log | tail -3
echo "Resting:" && grep "😴.*resting" /tmp/sim_test.log | tail -3
