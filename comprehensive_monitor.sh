#!/bin/bash
# Comprehensive behavior monitor - watches all rabbit behaviors

echo "🔬 COMPREHENSIVE BEHAVIOR MONITOR"
echo "=================================="
echo "Monitoring: eating, drinking, resting, movement"
echo ""

PREV_ACTION=""
PREV_POS=""
CHECK_NUM=1

while true; do
  # Get current state
  RESPONSE=$(curl -s http://127.0.0.1:54321/api/entities)

  if [ $? -ne 0 ]; then
    echo "[$CHECK_NUM] ⚠️  API call failed, retrying..."
    sleep 2
    CHECK_NUM=$((CHECK_NUM+1))
    continue
  fi

  # Parse entity data
  ACTION=$(echo "$RESPONSE" | jq -r '.entities[0].current_action // "NONE"')
  POS=$(echo "$RESPONSE" | jq -r '.entities[0].position | "(\(.x),\(.y))"')
  HUNGER=$(echo "$RESPONSE" | jq -r '.entities[0].hunger // 0')
  THIRST=$(echo "$RESPONSE" | jq -r '.entities[0].thirst // 0')
  ENERGY=$(echo "$RESPONSE" | jq -r '.entities[0].energy // 0')
  NAME=$(echo "$RESPONSE" | jq -r '.entities[0].name // "Unknown"')

  # Detect changes
  TIMESTAMP=$(date +"%H:%M:%S")

  # Always show status every 10 checks
  if [ $((CHECK_NUM % 10)) -eq 0 ]; then
    echo "[$CHECK_NUM @ $TIMESTAMP] $NAME at $POS | $ACTION | H:${HUNGER}% T:${THIRST}% E:${ENERGY}%"
  fi

  # Highlight behavior changes
  if [ "$ACTION" != "$PREV_ACTION" ]; then
    echo ""
    echo "[$CHECK_NUM @ $TIMESTAMP] 🔔 ACTION CHANGE: $PREV_ACTION → $ACTION"
    echo "                      Position: $POS | H:${HUNGER}% T:${THIRST}% E:${ENERGY}%"
    PREV_ACTION="$ACTION"
  fi

  # Highlight movement
  if [ "$POS" != "$PREV_POS" ] && [ -n "$PREV_POS" ]; then
    echo "[$CHECK_NUM @ $TIMESTAMP] 🚶 MOVED: $PREV_POS → $POS | $ACTION"
    PREV_POS="$POS"
  fi

  # Initialize prev_pos if empty
  if [ -z "$PREV_POS" ]; then
    PREV_POS="$POS"
    PREV_ACTION="$ACTION"
  fi

  # Check logs for eating/drinking/resting events
  RECENT_LOGS=$(tail -100 /tmp/sim_test.log | grep -E "(ate vegetation|drinking|resting|🐇)" | tail -1)
  if [ -n "$RECENT_LOGS" ]; then
    # Only show if it's a new log line (crude check)
    if [ $((CHECK_NUM % 5)) -eq 0 ]; then
      echo "[$CHECK_NUM] 📋 Recent: $RECENT_LOGS" | cut -c1-120
    fi
  fi

  sleep 2
  CHECK_NUM=$((CHECK_NUM+1))
done
