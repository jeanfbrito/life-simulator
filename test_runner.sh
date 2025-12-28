#!/bin/bash
echo "🧪 Running life-simulator with panic backtrace..."
export RUST_BACKTRACE=full
export RUST_LOG=debug

timeout 5 ./target/debug/life-simulator 2>&1 | grep -E "(💓|🔍 Frame|panic|error|WARN)" || echo "No heartbeat/frame logs found - Update schedule NOT running"
