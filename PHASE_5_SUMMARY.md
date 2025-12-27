# Phase 5: AI Planning Optimization - Summary Report

## Date
2025-12-26

## Objective
Improve simulator performance from 0.6-0.8 TPS to target of 10.0 TPS with 500 entities by optimizing AI planning systems

---

## Work Completed

### 1. Profiling & Analysis
- ✅ Configured built-in TickProfiler system
- ✅ Ran profiling with 500 entities (190 rabbits, 120 deer, 100 raccoons, 50 foxes, 25 wolves, 15 bears)
- ✅ Identified critical bottleneck: AI planning systems consuming 442ms/tick (98% of total tick time)
- ✅ Created `PHASE_5_PROFILING_ANALYSIS.md` with detailed findings

**Key Finding**: With 500 entities planning every tick at 0.88ms per entity, even if all other systems were optimized to zero, max theoretical TPS would be ~2.3

### 2. Decision Frequency Reduction Implementation
- ✅ Created `DecisionTimer` component in `src/entities/mod.rs`
- ✅ Added timer tick system to decrement timers each tick
- ✅ Modified all 6 species planners to check timer before planning
- ✅ Updated `plan_species_actions()` to respect timer
- ✅ Added DecisionTimer to all entity spawn functions
- ✅ Configured intervals: 10 ticks (herbivores), 5 ticks (predators)

### 3. Timer Bypass Bug Fix
- ✅ Identified issue: `NeedsReplanning` marker bypasses timer
- ✅ Implemented smart bypass logic: urgent triggers (fear, hunger, thirst) bypass timer, action failures respect timer
- ✅ Prevents constant replanning from pathfinding failures

### 4. Staggered Timer Initialization
- ✅ Added `DecisionTimer::new_staggered()` with random offsets
- ✅ Updated all spawn functions to use staggered initialization
- ✅ Goal: Distribute planning load across ticks instead of synchronized bursts

---

## Performance Results

### Baseline (No Optimization)
```
Tick 50 Total: 451.4ms
- AI Planning: 442.2ms (98%)
- Other Systems: 9.2ms (2%)
Sustained TPS: 0.6-0.8
```

### Iteration 1: Synchronized Timers
```
Tick 50 Total: 363.1ms (-19.6% ✅)
- AI Planning: 353.0ms (-20% ✅)
- plan_rabbit: 123.9ms (+15% ❌)  <-- Synchronized planning burst
- plan_deer: 70.6ms (-27% ✅)
- plan_wolf: 65.3ms (-7% ✅)
- plan_raccoon: 49.4ms (-39% ✅)
- plan_fox: 27.3ms (-37% ✅)
- plan_bear: 16.5ms (-61% ✅)
Sustained TPS: 0.5 (-17% ❌)
```

### Iteration 2: Staggered Timers
```
Sustained TPS: 0.4 (-33% ❌)
Profiler: No output (possible profiler issue)
Action failures: Still high (~9,600+)
```

---

## Analysis: The Performance Paradox

### The Core Mystery
**Tick 50 profiling shows 19% improvement, but sustained TPS degraded by 17-33%**

### Possible Explanations

#### 1. **Profiled Tick Not Representative**
- Tick 50 might be atypical
- Performance may degrade over time
- Need profiling of ticks 100, 150, 200 for full picture

#### 2. **Unaccounted Time Increased**
- Baseline analysis showed ~979ms unaccounted time (pathfinding, world queries)
- DecisionTimer implementation may affect other systems:
  - EntityCommands overhead for timer resets
  - Query complexity increased
  - System scheduling changed

#### 3. **Measurement Methodology Issues**
- TPS calculated from wall-clock time
- Profiler measures single-tick execution
- Gap between measurements suggests external factors

#### 4. **Pathfinding Bottleneck**
- 9,600+ action failures observed (mostly Wander pathfinding failures)
- Each failure triggers replanning (respects timer, but still overhead)
- Pathfinding system not profiled, could be true bottleneck

#### 5. **ECS Overhead**
- DecisionTimer component adds query complexity
- Timer tick system processes all 500 entities every tick
- EntityCommands::insert() for timer resets queued/deferred

---

## Lessons Learned

### ✅ What Worked
1. **Profiling infrastructure** - TickProfiler provided excellent visibility
2. **Component design** - DecisionTimer cleanly integrates with ECS
3. **Smart bypass logic** - Urgent vs non-urgent replanning distinction
4. **Staggering concept** - Right idea to distribute load

### ❌ What Didn't Work
1. **Decision frequency reduction alone** - Not sufficient for 10 TPS target
2. **Staggered timers** - Didn't improve TPS (may have worsened)
3. **Single-tick profiling** - Doesn't capture full performance picture

### ⚠️ Unexpected Discoveries
1. **Profiled tick ≠ sustained performance** - Significant gap exists
2. **Rabbit planning spike** - Synchronized timers caused localized regression
3. **Action failures** - High pathfinding failure rate indicates deeper issue

---

## Recommendations

### Immediate Actions

#### A. **Disable Timer Optimization (Rollback)**
The current implementation degraded performance. Revert to baseline for stability:
```bash
git checkout HEAD~N src/entities/mod.rs
git checkout HEAD~N src/ai/planner.rs
git checkout HEAD~N src/entities/entity_types.rs
git checkout HEAD~N src/ai/event_driven_planner.rs
```

#### B. **Investigate True Bottleneck**
Run comprehensive profiling including pathfinding:
- Add profiling for pathfinding system
- Profile ticks 50, 100, 150, 200 for consistency
- Track unaccounted time sources

#### C. **Fix Pathfinding Failures**
With 9,600+ failures, reducing these could improve performance:
- Better wander target selection (avoid unreachable tiles)
- Pathfinding caching (reuse recent paths)
- Fallback behaviors when pathfinding fails

### Alternative Optimization Strategies

#### Strategy 1: **Hierarchical Pathfinding**
- Coarse grid for long-distance, fine grid for local
- Expected: 2-3x pathfinding speedup
- Effort: High

#### Strategy 2: **Decision Caching** (Original Phase 5 Plan)
- Cache recent decisions for similar situations
- Expected: 2-3x planning speedup
- Effort: Medium-High

#### Strategy 3: **Simplified Decision Trees**
- Reduce predator hunt complexity
- Limit prey search radius
- Expected: 1.5-2x planning speedup
- Effort: Low-Medium

#### Strategy 4: **Batch Processing**
- Process similar entities together for cache locality
- Expected: 1.2-1.5x speedup
- Effort: Medium

#### Strategy 5: **Parallel Planning** (Advanced)
- Use Bevy's parallel system execution
- Plan multiple species in parallel
- Expected: 2-4x speedup (with multi-core)
- Effort: High

---

## Phase 5 Conclusion

**Status**: **Inconclusive** - Optimization showed promise in profiling but failed to improve sustained TPS

**Root Cause**: Likely combination of:
1. Unaccounted time bottleneck (pathfinding?)
2. ECS overhead from new components/systems
3. Measurement methodology gap

**Path Forward**:
1. Rollback DecisionTimer implementation
2. Profile pathfinding and unaccounted systems
3. Consider alternative strategies (decision caching, pathfinding optimization)
4. Implement continuous profiling (not just tick 50)

**Effort Summary**:
- Time invested: ~4 hours
- Code changes: 10+ files modified
- Learning: Significant insights into Bevy ECS performance
- Production ready: No - rollback recommended

---

## Files Modified

1. `src/entities/mod.rs` - DecisionTimer component + tick system
2. `src/ai/planner.rs` - Timer check + smart bypass logic
3. `src/ai/event_driven_planner.rs` - System registration
4. `src/entities/entity_types.rs` - All 6 spawn functions
5. `src/entities/types/rabbit.rs` - Query update
6. `src/entities/types/deer.rs` - Query update
7. `src/entities/types/raccoon.rs` - Query update
8. `src/entities/types/fox.rs` - Query update
9. `src/entities/types/wolf.rs` - Query update
10. `src/entities/types/bear.rs` - Query update

## Documentation Created

1. `PHASE_5_PROFILING_ANALYSIS.md` - Profiling methodology and findings
2. `PHASE_5_ITERATION_1_RESULTS.md` - Iteration 1 performance analysis
3. `PHASE_5_SUMMARY.md` (this file) - Comprehensive summary

---

**Recommendation**: Focus next optimization effort on **pathfinding** and **decision caching**, as these showed clearer paths to improvement in the original analysis.
