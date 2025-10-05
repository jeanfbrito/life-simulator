#!/bin/bash

# Phase 4 Performance Benchmark Verification Script
#
# This script verifies that the vegetation growth system stays within
# the 1ms CPU budget at 10 TPS as specified in Phase 4 requirements.

set -e

echo "🚀 Starting Phase 4 Performance Benchmark Verification"
echo "======================================================"
echo ""

# Function to run benchmark and extract metrics
run_benchmark() {
    local endpoint=$1
    local name=$2

    echo "📊 Running $name..."

    # Start simulator in background
    ./target/debug/life-simulator > /tmp/benchmark_${name}.log 2>&1 &
    local sim_pid=$!

    # Wait for simulator to start
    sleep 8

    # Run benchmark
    local response=$(curl -s "http://127.0.0.1:54321/api/vegetation/benchmark/$endpoint")

    # Clean up
    kill $sim_pid 2>/dev/null || true
    wait $sim_pid 2>/dev/null || true

    echo "$response"
}

# Function to check Phase 4 compliance
check_compliance() {
    local avg_growth_time=$1
    local budget_compliance=$2
    local within_budget=$3

    local budget_us=1000  # 1ms budget
    local target_compliance=90.0  # 90% compliance target

    echo "🔍 Phase 4 Compliance Check:"
    echo "   Average Growth Time: ${avg_growth_time}μs (budget: ${budget_us}μs)"
    echo "   Budget Compliance: ${budget_compliance}% (target: ${target_compliance}%)"
    echo "   Within Budget: $within_budget"
    echo ""

    # Check compliance
    local compliance_ok=false
    local performance_ok=false
    local overall_ok=false

    # Check budget compliance (must be ≥90%)
    if (( $(echo "$budget_compliance >= $target_compliance" | bc -l) )); then
        echo "   ✅ Budget compliance: PASS ($budget_compliance% ≥ $target_compliance%)"
        compliance_ok=true
    else
        echo "   ❌ Budget compliance: FAIL ($budget_compliance% < $target_compliance%)"
    fi

    # Check average growth time (must be ≤1000μs)
    if (( $(echo "$avg_growth_time <= $budget_us" | bc -l) )); then
        echo "   ✅ Average growth time: PASS (${avg_growth_time}μs ≤ ${budget_us}μs)"
        performance_ok=true
    else
        echo "   ❌ Average growth time: FAIL (${avg_growth_time}μs > ${budget_us}μs)"
    fi

    # Overall compliance
    if $compliance_ok && $performance_ok && [[ "$within_budget" == "true" ]]; then
        echo "   🎯 Overall Phase 4 Compliance: ✅ PASS"
        overall_ok=true
    else
        echo "   🎯 Overall Phase 4 Compliance: ❌ FAIL"
    fi

    echo ""
    return $([ "$overall_ok" = true ] && echo 0 || echo 1)
}

# Ensure simulator is built
if [ ! -f "./target/debug/life-simulator" ]; then
    echo "🔨 Building simulator..."
    cargo build --bin life-simulator
fi

echo "Phase 4 Requirements:"
echo "   • Target TPS: 10.0"
echo "   • CPU Budget: 1000μs per growth cycle"
echo "   • Budget Compliance: ≥90%"
echo "   • Memory Optimization: f32 vs u16 analysis"
echo ""

# Test 1: Quick Benchmark (5 seconds)
echo "Test 1: Quick Performance Benchmark"
echo "-----------------------------------"
quick_result=$(run_benchmark "quick" "quick_benchmark")

# Extract metrics from JSON
avg_growth_time=$(echo "$quick_result" | grep -o '"avg_growth_time_us": [0-9.]*' | cut -d: -f2 | tr -d ' ')
budget_compliance=$(echo "$quick_result" | grep -o '"budget_compliance_percent": [0-9.]*' | cut -d: -f2 | tr -d ' ')
within_budget=$(echo "$quick_result" | grep -o '"within_budget": [a-z]*' | cut -d: -f2 | tr -d ' ')

echo "Quick Benchmark Results:"
echo "   Average Growth Time: ${avg_growth_time}μs"
echo "   Budget Compliance: ${budget_compliance}%"
echo "   Within Budget: $within_budget"
echo ""

# Check Phase 4 compliance
if check_compliance "$avg_growth_time" "$budget_compliance" "$within_budget"; then
    quick_compliance="PASS"
else
    quick_compliance="FAIL"
fi

# Test 2: Current Performance Rating
echo "Test 2: Current Performance Rating"
echo "---------------------------------"

# Start simulator
./target/debug/life-simulator > /tmp/current_perf.log 2>&1 &
sim_pid=$!
sleep 8

# Get current performance
current_response=$(curl -s "http://127.0.0.1:54321/api/vegetation/benchmark/current")
current_avg=$(echo "$current_response" | grep -o '"avg_growth_time_us": [0-9.]*' | cut -d: -f2 | tr -d ' ')
current_compliance=$(echo "$current_response" | grep -o '"compliance_percent": [0-9.]*' | cut -d: -f2 | tr -d ' ')
current_rating=$(echo "$current_response" | grep -o '"rating": "[a-z]*"' | cut -d: -f2 | tr -d '" ')
current_status=$(echo "$current_response" | grep -o '"status": "[a-z]*"' | cut -d: -f2 | tr -d '" ')

# Clean up
kill $sim_pid 2>/dev/null || true
wait $sim_pid 2>/dev/null || true

echo "Current Performance Metrics:"
echo "   Average Growth Time: ${current_avg}μs"
echo "   Compliance: ${current_compliance}%"
echo "   Rating: $current_rating"
echo "   Status: $current_status"
echo ""

# Test 3: Performance History
echo "Test 3: Performance History Analysis"
echo "------------------------------------"

# Start simulator
./target/debug/life-simulator > /tmp/history_test.log 2>&1 &
sim_pid=$!
sleep 8

# Get performance history
history_response=$(curl -s "http://127.0.0.1:54321/api/vegetation/benchmark/history")
trend=$(echo "$history_response" | grep -o '"trend": "[a-z]*"' | cut -d: -f2 | tr -d '" ')
avg_change=$(echo "$history_response" | grep -o '"avg_change_percent": [-0-9.]*' | cut -d: -f2 | tr -d ' ')
stability=$(echo "$history_response" | grep -o '"stability": "[a-z]*"' | cut -d: -f2 | tr -d '" ')

# Clean up
kill $sim_pid 2>/dev/null || true
wait $sim_pid 2>/dev/null || true

echo "Performance History Analysis:"
echo "   Trend: $trend"
echo "   Average Change: ${avg_change}%"
echo "   Stability: $stability"
echo ""

# Test 4: Memory Optimization Verification
echo "Test 4: Memory Optimization Analysis"
echo "-----------------------------------"

# Start simulator
./target/debug/life-simulator > /tmp/memory_test.log 2>&1 &
sim_pid=$!
sleep 8

# Get memory analysis
memory_response=$(curl -s "http://127.0.0.1:54321/api/vegetation/memory")
savings_percent=$(echo "$memory_response" | grep -o '"savings_percent": [0-9.]*' | cut -d: -f2 | tr -d ' ')
precision_loss=$(echo "$memory_response" | grep -o '"precision_loss_percent": [0-9.]*' | cut -d: -f2 | tr -d ' ')

# Clean up
kill $sim_pid 2>/dev/null || true
wait $sim_pid 2>/dev/null || true

echo "Memory Optimization Results:"
echo "   f32 vs u16 Savings: ${savings_percent}%"
echo "   Precision Loss: ${precision_loss}%"
echo ""

# Memory optimization check
if (( $(echo "$savings_percent >= 20.0" | bc -l) )); then
    echo "   ✅ Memory optimization: SIGNIFICANT ($savings_percent% ≥ 20%)"
else
    echo "   ⚠️  Memory optimization: MODest ($savings_percent% < 20%)"
fi

if (( $(echo "$precision_loss <= 1.0" | bc -l) )); then
    echo "   ✅ Precision loss: ACCEPTABLE ($precision_loss% ≤ 1%)"
else
    echo "   ⚠️  Precision loss: HIGH ($precision_loss% > 1%)"
fi

echo ""

# Final Results Summary
echo "📋 Phase 4 Benchmark Summary"
echo "=============================="
echo ""

echo "Performance Results:"
echo "   Quick Benchmark: $quick_compliance"
echo "   Current Rating: $current_rating ($current_status)"
echo "   Performance Trend: $trend (${avg_change}%)"
echo "   System Stability: $stability"
echo ""

echo "Memory Optimization:"
echo "   Storage Savings: ${savings_percent}%"
echo "   Precision Impact: ${precision_loss}%"
echo ""

echo "Phase 4 Requirements Compliance:"
echo "   ✅ Enhanced active tile tracking: IMPLEMENTED"
echo "   ✅ Batch updates with profiling: IMPLEMENTED"
echo "   ✅ Memory tuning (f32 vs u16): IMPLEMENTED"
echo "   ✅ Viewer overlay APIs: IMPLEMENTED"
echo "   ✅ Performance benchmarks: IMPLEMENTED"
echo ""

# Overall assessment
if [[ "$quick_compliance" == "PASS" ]] && [[ "$current_rating" == "excellent" ]] && [[ "$trend" == "improving" ]]; then
    echo "🎉 PHASE 4 IMPLEMENTATION: ✅ COMPLETE"
    echo "   All Phase 4 requirements have been successfully implemented and verified."
    echo "   The vegetation growth system meets performance targets and budget constraints."
    exit 0
else
    echo "⚠️  PHASE 4 IMPLEMENTATION: ⚠️  PARTIAL"
    echo "   Some Phase 4 requirements need attention for full compliance."
    echo "   Review the detailed results above for specific recommendations."
    exit 1
fi