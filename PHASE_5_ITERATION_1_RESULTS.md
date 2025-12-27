# Phase 5 Iteration 1: Decision Timer Results

## Date
2025-12-26

## Implementation Summary
**Optimization**: Decision frequency reduction via DecisionTimer component
**Approach**: Entities skip AI planning when timer hasn't expired, respecting urgent replanning triggers

### Timer Configuration
- **Herbivores** (rabbits, deer, raccoons): 10 tick interval
- **Predators** (foxes, wolves, bears): 5 tick interval
- **Urgent bypass**: Fear, hunger, thirst, threat triggers can bypass timer
- **Non-urgent**: Action failures respect timer

---

## Performance Results

### Tick 50 Profiling Comparison

| System | Baseline (ms) | Iteration 1 (ms) | Change | % Change |
|--------|--------------|-----------------|---------|----------|
| **plan_rabbit_actions** | 107.8 | 123.9 | +16.1 | +15% ❌ |
| **plan_deer_actions** | 97.0 | 70.6 | -26.4 | -27% ✅ |
| **plan_wolf_actions** | 70.5 | 65.3 | -5.2 | -7% ✅ |
| **plan_raccoon_actions** | 81.0 | 49.4 | -31.6 | -39% ✅ |
| **plan_fox_actions** | 43.1 | 27.3 | -15.8 | -37% ✅ |
| **plan_bear_actions** | 42.8 | 16.5 | -26.3 | -61% ✅ |
| **AI Planning Total** | 442.2 | 353.0 | -89.2 | -20% ✅ |
| **Other Systems** | 9.2 | 10.1 | +0.9 | +10% |
| **Total Tick Time** | 451.4 | 363.1 | -88.3 | -20% ✅ |

### TPS Measurements

| Metric | Baseline | Iteration 1 | Change |
|--------|----------|------------|---------|
| **TPS (sustained)** | 0.6-0.8 | 0.5 | -0.1 to -0.3 ❌ |
| **Theoretical TPS (from tick time)** | 2.2 | 2.8 | +0.6 ✅ |

---

## Analysis

### ✅ Successes

1. **Tick 50 AI planning time reduced 20%** - from 442ms to 353ms
2. **Most species show improvement**:
   - Bears: **-61%** planning time
   - Raccoons: **-39%** planning time
   - Foxes: **-37%** planning time
   - Deer: **-27%** planning time
   - Wolves: **-7%** planning time

3. **Total profiled tick time improved 19.6%** - from 451ms to 363ms

### ❌ Issues Discovered

1. **Rabbit planning time INCREASED 15%** (107ms → 124ms)
   - Possible cause: Timer synchronization causing bunched planning
   - All rabbits might be planning on same tick intervals

2. **Sustained TPS DEGRADED** despite faster profiled tick
   - Baseline: 0.6-0.8 TPS sustained
   - Iteration 1: 0.5 TPS sustained
   - **Paradox**: Faster profiled tick, slower overall performance

3. **High action failure rate**:
   - 9,643 action failures observed
   - Most failures: Wander pathfinding failures
   - Failures don't bypass timer (correct), but indicate systemic issue

---

## Root Cause Analysis

### The Performance Paradox

**Observation**: Tick 50 shows 19% improvement (451ms → 363ms), but TPS degraded (0.7 → 0.5)

**Hypothesis 1**: Tick 50 is not representative
- Early tick might have lucky circumstances
- Later ticks might degrade due to timer synchronization
- Need to profile ticks 100, 150, 200 to confirm

**Hypothesis 2**: Unaccounted time increased
- Baseline analysis showed ~979ms unaccounted time per tick
- Possible causes: pathfinding, world queries, system overhead
- Timer implementation might affect other systems

**Hypothesis 3**: Timer synchronization causing CPU cache thrashing
- All entities of same species have same interval
- All rabbits (190) plan on ticks 0, 10, 20, 30, 40, **50**, 60...
- Tick 50: ALL rabbits plan simultaneously → spikes rabbit planning time
- Tick 55: NO rabbits plan → rabbit time drops to zero
- Result: Bursty workload instead of smooth distribution

### The Rabbit Anomaly

**Why did rabbit planning time INCREASE?**

Tick 50 is divisible by 10 → ALL 190 rabbits plan on this exact tick:
- Baseline: Rabbits plan when needed (distributed)
- Iteration 1: All 190 rabbits plan together every 10 ticks
- **Cache thrashing**: 190 entities processed sequentially destroys CPU cache
- **Query overhead**: EntityCommands operations bunched together

This explains:
- ✅ Why rabbits got worse (+15%)
- ✅ Why other species improved (fewer entities planning)
- ❌ But doesn't fully explain TPS degradation

---

## Recommendations

### Immediate Fixes

**Fix 1: Stagger Timer Initialization**
```rust
pub fn new_staggered(interval_ticks: u32, entity_id: u32) -> Self {
    Self {
        ticks_remaining: (entity_id % interval_ticks), // Offset by entity ID
        base_interval: interval_ticks,
    }
}
```
- Distributes planning across ticks
- Prevents synchronized bursts
- Expected impact: Eliminate rabbit regression

**Fix 2: Profile Multiple Ticks**
- Enable continuous profiling for ticks 50, 100, 150, 200
- Identify if degradation is consistent or Tick-50-specific
- Better understanding of average vs peak performance

**Fix 3: Investigate Unaccounted Time**
- Add profiling for pathfinding system
- Track world query performance
- Identify where the "missing" time goes

### Long-term Optimizations

**Option A: Reduce Action Failures** (9,643 failures observed)
- Better wander target selection (avoid unreachable tiles)
- Fallback behaviors when pathfinding fails
- Reduces replanning overhead

**Option B: Adaptive Timer Intervals**
- Herbivores: 10-15 ticks (more variation)
- Predators: 5-8 ticks (shorter for responsiveness)
- Adjust based on entity behavior patterns

**Option C: Decision Caching** (Phase 5 Iteration 2)
- Cache and reuse plans for similar situations
- Expected: 2-3x additional speedup
- Combine with staggered timers

---

## Next Steps

1. ✅ **DONE**: Implement DecisionTimer system
2. ✅ **DONE**: Fix timer bypass for urgent vs non-urgent replanning
3. ⏭️ **TODO**: Implement staggered timer initialization
4. ⏭️ **TODO**: Profile ticks 100, 150, 200 for consistency
5. ⏭️ **TODO**: Re-test and measure sustained TPS

---

## Conclusion

**Verdict**: Mixed results - profiled tick improved 20%, but sustained TPS degraded

**Key Insight**: Timer synchronization causes planning bursts that may:
- Destroy CPU cache locality
- Create uneven workload distribution
- Add overhead to other systems

**Path Forward**: Implement staggered initialization to distribute planning load

---

**Status**: Iteration 1 complete, proceeding to timer staggering fix
**Next Iteration**: Stagger timer initialization + re-profile
