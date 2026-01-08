# Behavior Verification Guide

**CRITICAL**: Code compiling does NOT mean the simulation works correctly. This guide ensures animal behaviors are actually functioning.

## Quick Health Check (30 seconds)

```bash
# 1. Start simulation
cargo build --release && ./target/release/life-simulator &

# 2. Wait for startup
sleep 15

# 3. Run health check
curl -s http://127.0.0.1:54321/api/debug/health | python3 -c "
import sys, json
d = json.load(sys.stdin)
print('=== HEALTH CHECK ===')
print(f\"Status: {d['status']}\")
print(f\"Healthy: {d['is_healthy']}\")
print(f\"TPS: {d['current_tps']:.1f}\")
print(f\"Stuck entities: {d['alerts']['entities_stuck']}\")
print(f\"AI loops: {d['alerts']['ai_loops']}\")
"
```

**Expected**: `is_healthy: true`, TPS ~10, stuck entities = 0

## Complete Verification Checklist

### 1. Entities Exist and Have Data

```bash
curl -s http://127.0.0.1:54321/api/entities | python3 -c "
import sys, json
d = json.load(sys.stdin)
entities = d.get('entities', [])
print(f'Total entities: {len(entities)}')
for e in entities[:5]:
    print(f\"  {e.get('name')} ({e.get('entity_type')}): {e.get('current_action')}\")
"
```

**Expected**: 5+ entities with names, types, and actions

### 2. Movement Working

```bash
# Check positions change over time
for i in 1 2 3; do
  curl -s http://127.0.0.1:54321/api/entities | python3 -c "
import sys, json
d = json.load(sys.stdin)
for e in d.get('entities', [])[:3]:
    pos = e.get('position', {})
    print(f\"{e.get('name')}: ({pos.get('x')}, {pos.get('y')})\")
"
  sleep 5
done
```

**Expected**: Positions change between samples (not all 0,0)

### 3. Eating (Grazing) Working

```bash
# Check logs for grazing completions
grep "Hunger reduced" /tmp/life-sim.log | tail -5
```

**Expected**: Messages like `Entity X completed grazing! Hunger reduced by 25.0`

Or via API - hunger should stay low:
```bash
curl -s http://127.0.0.1:54321/api/entities | python3 -c "
import sys, json
for e in json.load(sys.stdin).get('entities', []):
    h = e.get('hunger', 0)
    if h > 50: print(f\"WARNING: {e.get('name')} very hungry ({h}%)\")
"
```

**Expected**: No warnings (hunger stays controlled)

### 4. Drinking Working

Drinking triggers at high thirst thresholds:
- Rabbits: 75% thirst
- Deer: 65% thirst  
- Raccoons: 55% thirst

```bash
# Check if any entity is critically thirsty but not drinking
curl -s http://127.0.0.1:54321/api/entities | python3 -c "
import sys, json
for e in json.load(sys.stdin).get('entities', []):
    t = e.get('thirst', 0)
    action = e.get('current_action', '')
    if t > 80 and 'Drink' not in action:
        print(f\"ISSUE: {e.get('name')} at {t}% thirst but action is {action}\")
"
```

**Expected**: No issues (high thirst entities should be drinking)

### 5. Reproduction Working

```bash
# Check for mating pair formations
grep -E "Pair formed" /tmp/life-sim.log | tail -5
```

**Expected**: Messages like `Pair formed: female X with male Y -> rendezvous at Z`

Check via API:
```bash
curl -s http://127.0.0.1:54321/api/entities | python3 -c "
import sys, json
for e in json.load(sys.stdin).get('entities', []):
    if e.get('current_action') == 'Mate':
        print(f\"{e.get('name')} is mating (eligible: {e.get('eligible_to_mate')})\")
"
```

### 6. Fear System Active

```bash
# Check fear trigger system is running
grep "trigger_fear" /tmp/life-sim.log | head -3
```

**Expected**: System appears in tick performance logs (even if 0.0ms when no predators nearby)

### 7. No Stuck Entities

```bash
curl -s http://127.0.0.1:54321/api/debug/health | jq '.alerts.entities_stuck'
```

**Expected**: `0`

## Automated Full Verification Script

Save as `scripts/verify_behaviors.sh`:

```bash
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
```

## Common Issues & Solutions

### Issue: All entities at position (0, 0)

**Cause**: Entity tracker not syncing positions  
**Fix**: Check `sync_entity_tracker` system is running

### Issue: Entities stuck with high hunger

**Cause**: Vegetation not available or AI not planning  
**Fix**: 
1. Check vegetation exists: `curl http://127.0.0.1:54321/api/collectables/stats`
2. Check AI planning: look for `queuing action` in logs

### Issue: No mating despite eligible entities

**Cause**: Missing compatible partners or cooldown active  
**Fix**: Check `reproduction_cooldown_ticks` in entity data

### Issue: TPS below 5

**Cause**: Performance bottleneck  
**Fix**: Check tick performance logs for slow systems

## API Endpoints Reference

| Endpoint | Purpose |
|----------|---------|
| `/api/debug/health` | Overall health status and alerts |
| `/api/debug/tps` | Current TPS metrics |
| `/api/entities` | All entity data with stats |
| `/api/collectables/stats` | Vegetation and resource stats |
| `/api/vegetation/metrics` | Detailed vegetation metrics |

## When to Run Verification

1. **After any AI system changes** - behavior may silently break
2. **After entity component changes** - required components may be missing
3. **After pathfinding changes** - movement may fail
4. **Before PR merge** - ensure no regressions
5. **After vegetation system changes** - food availability affects behavior

---

**Remember**: A successful `cargo check` only means syntax is correct. Always verify behaviors are functioning with actual runtime tests!
