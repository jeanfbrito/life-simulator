#!/bin/bash
# Development helper script for life-simulator
# Usage: ./scripts/dev.sh [command]

set -e

API_BASE="http://127.0.0.1:54321"

show_help() {
    echo "Life Simulator Development Helper"
    echo ""
    echo "Usage: ./scripts/dev.sh [command]"
    echo ""
    echo "Commands:"
    echo "  start      Build and start the simulation"
    echo "  stop       Stop the simulation"
    echo "  restart    Stop and start the simulation"
    echo "  status     Check simulation status"
    echo "  logs       Show simulation logs"
    echo "  health     Show health check"
    echo "  entities   List all entities"
    echo "  tps        Show TPS metrics"
    echo "  verify     Run behavior verification"
    echo "  build      Build release binary only"
    echo "  test       Run cargo tests"
    echo "  watch      Watch entity positions (updates every 2s)"
    echo "  help       Show this help"
    echo ""
}

do_build() {
    echo "Building release binary..."
    source "$HOME/.cargo/env" 2>/dev/null || true
    cargo build --release
    echo "Build complete."
}

do_start() {
    if pgrep -f "life-simulator" > /dev/null; then
        echo "Simulation already running. Use 'restart' to restart."
        return 1
    fi
    
    do_build
    
    echo "Starting simulation..."
    cd "$(dirname "$0")/.."
    nohup ./target/release/life-simulator > /tmp/life-sim.log 2>&1 &
    
    echo "Waiting for server to start..."
    for i in {1..30}; do
        if curl -s "$API_BASE/api/debug/health" > /dev/null 2>&1; then
            echo "Simulation started! (PID: $(pgrep -f life-simulator))"
            echo "Logs: /tmp/life-sim.log"
            echo "Viewer: $API_BASE/viewer.html"
            return 0
        fi
        sleep 1
    done
    
    echo "ERROR: Simulation failed to start within 30 seconds"
    return 1
}

do_stop() {
    if pkill -f "life-simulator"; then
        echo "Simulation stopped."
    else
        echo "No simulation running."
    fi
}

do_restart() {
    do_stop
    sleep 2
    do_start
}

do_status() {
    if ! pgrep -f "life-simulator" > /dev/null; then
        echo "Status: NOT RUNNING"
        return 1
    fi
    
    echo "Status: RUNNING (PID: $(pgrep -f life-simulator))"
    echo ""
    
    HEALTH=$(curl -s "$API_BASE/api/debug/health" 2>/dev/null)
    if [ -n "$HEALTH" ]; then
        echo "Health: $(echo "$HEALTH" | python3 -c "import sys,json; d=json.load(sys.stdin); print('HEALTHY' if d['is_healthy'] else 'UNHEALTHY')")"
        echo "TPS: $(echo "$HEALTH" | python3 -c "import sys,json; print(f\"{json.load(sys.stdin)['current_tps']:.1f}\")")"
    else
        echo "Health: Unable to connect"
    fi
}

do_logs() {
    if [ -f /tmp/life-sim.log ]; then
        tail -f /tmp/life-sim.log
    else
        echo "No log file found at /tmp/life-sim.log"
    fi
}

do_health() {
    curl -s "$API_BASE/api/debug/health" | python3 -m json.tool
}

do_entities() {
    curl -s "$API_BASE/api/entities" | python3 -c "
import sys, json
d = json.load(sys.stdin)
entities = d.get('entities', [])
print(f'Total: {len(entities)} entities')
print('')
print(f'{\"ID\":<6} {\"Name\":<12} {\"Type\":<10} {\"Action\":<12} {\"Hunger\":<8} {\"Thirst\":<8}')
print('-' * 60)
for e in entities:
    print(f\"{e.get('id', '?'):<6} {e.get('name', '?')[:11]:<12} {e.get('entity_type', '?')[:9]:<10} {e.get('current_action', '?')[:11]:<12} {e.get('hunger', 0):>6.1f}% {e.get('thirst', 0):>6.1f}%\")
"
}

do_tps() {
    curl -s "$API_BASE/api/debug/tps" | python3 -m json.tool
}

do_verify() {
    "$(dirname "$0")/verify_behaviors.sh"
}

do_watch() {
    echo "Watching entity positions (Ctrl+C to stop)..."
    while true; do
        clear
        echo "=== Entity Positions ($(date +%H:%M:%S)) ==="
        curl -s "$API_BASE/api/entities" 2>/dev/null | python3 -c "
import sys, json
d = json.load(sys.stdin)
for e in d.get('entities', []):
    pos = e.get('position', {})
    print(f\"{e.get('name', '?'):<12} ({pos.get('x', 0):>4}, {pos.get('y', 0):>4}) - {e.get('current_action', '?')}\")
" 2>/dev/null || echo "Simulation not running"
        sleep 2
    done
}

do_test() {
    source "$HOME/.cargo/env" 2>/dev/null || true
    cargo test "$@"
}

# Main
case "${1:-help}" in
    start)   do_start ;;
    stop)    do_stop ;;
    restart) do_restart ;;
    status)  do_status ;;
    logs)    do_logs ;;
    health)  do_health ;;
    entities) do_entities ;;
    tps)     do_tps ;;
    verify)  do_verify ;;
    build)   do_build ;;
    test)    shift; do_test "$@" ;;
    watch)   do_watch ;;
    help|*)  show_help ;;
esac
