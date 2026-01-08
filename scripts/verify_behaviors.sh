#!/bin/bash
set -e

echo "=== Life Simulator Behavior Verification ==="
echo ""

# Check if server is running
if ! curl -s http://127.0.0.1:54321/api/debug/health > /dev/null 2>&1; then
    echo "ERROR: Simulation not running on port 54321"
    echo "Start with: cargo run --release --bin life-simulator"
    exit 1
fi

# Health check
echo "1. Health Check"
HEALTH=$(curl -s http://127.0.0.1:54321/api/debug/health)
IS_HEALTHY=$(echo $HEALTH | python3 -c "import sys,json; print(json.load(sys.stdin)['is_healthy'])")
TPS=$(echo $HEALTH | python3 -c "import sys,json; print(f\"{json.load(sys.stdin)['current_tps']:.1f}\")")
STUCK=$(echo $HEALTH | python3 -c "import sys,json; print(json.load(sys.stdin)['alerts']['entities_stuck'])")

if [ "$IS_HEALTHY" = "True" ]; then
    echo "   [PASS] System healthy"
else
    echo "   [FAIL] System unhealthy"
fi
echo "   TPS: $TPS, Stuck: $STUCK"

# Entity count
echo ""
echo "2. Entity Population"
ENTITIES=$(curl -s http://127.0.0.1:54321/api/entities)
COUNT=$(echo $ENTITIES | python3 -c "import sys,json; print(len(json.load(sys.stdin).get('entities', [])))")
echo "   Total entities: $COUNT"
if [ "$COUNT" -gt 0 ]; then
    echo "   [PASS] Entities exist"
else
    echo "   [FAIL] No entities found"
fi

# Actions check
echo ""
echo "3. Current Actions"
echo $ENTITIES | python3 -c "
import sys, json
from collections import Counter
entities = json.load(sys.stdin).get('entities', [])
actions = Counter(e.get('current_action', 'Unknown') for e in entities)
for action, count in actions.most_common():
    print(f'   {action}: {count}')
"

# Grazing check
echo ""
echo "4. Eating (Grazing)"
if grep -q "Hunger reduced" /tmp/life-sim.log 2>/dev/null; then
    GRAZE_COUNT=$(grep -c "Hunger reduced" /tmp/life-sim.log 2>/dev/null || echo "0")
    echo "   [PASS] $GRAZE_COUNT grazing completions logged"
else
    echo "   [WARN] No grazing completions in log (may need more time)"
fi

# Mating check
echo ""
echo "5. Reproduction"
if grep -q "Pair formed" /tmp/life-sim.log 2>/dev/null; then
    PAIR_COUNT=$(grep -c "Pair formed" /tmp/life-sim.log 2>/dev/null || echo "0")
    echo "   [PASS] $PAIR_COUNT mating pairs formed"
else
    echo "   [INFO] No mating pairs yet (normal for early simulation)"
fi

# Summary
echo ""
echo "=== Verification Complete ==="
if [ "$IS_HEALTHY" = "True" ] && [ "$COUNT" -gt 0 ] && [ "$STUCK" = "0" ]; then
    echo "Result: PASS - Core behaviors working"
    exit 0
else
    echo "Result: ISSUES DETECTED - Review above"
    exit 1
fi
