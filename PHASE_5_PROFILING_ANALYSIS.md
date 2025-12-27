# Phase 5: Profiling Analysis & Optimization Plan

## Date
2025-12-26

## Objective
Identify performance bottlenecks through profiling and create targeted optimization plan to improve TPS from 0.6-0.8 to target of 10.0.

---

## Profiling Method

**Tool**: Built-in TickProfiler (src/simulation/profiler.rs)
**Configuration**: 500 entities (190 rabbits, 120 deer, 100 raccoons, 50 foxes, 25 wolves, 15 bears)
**Sample**: Tick 50 profiling data
**Total Tick Duration**: 451.4ms

---

## Profiling Results

### System Performance Breakdown (Tick 50)

| System | Duration (ms) | % of Total | Entity Count | ms per Entity |
|--------|--------------|------------|--------------|---------------|
| plan_rabbit_actions | 107.8 | 24% | 190 | 0.57 |
| plan_deer_actions | 97.0 | 21% | 120 | 0.81 |
| plan_raccoon_actions | 81.0 | 18% | 100 | 0.81 |
| plan_wolf_actions | 70.5 | 16% | 25 | 2.82 |
| plan_fox_actions | 43.1 | 10% | 50 | 0.86 |
| plan_bear_actions | 42.8 | 9% | 15 | 2.85 |
| **AI Planning Total** | **442.2** | **98%** | **500** | **0.88** |
| ai_actions | 4.4 | 1% | - | - |
| chunk_aggregation | 3.2 | 1% | - | - |
| heatmap_snapshot | 0.9 | 0% | - | - |
| chunk_lod | 0.6 | 0% | - | - |
| All others | ~0.0 | 0% | - | - |

### Key Findings

#### Critical Bottleneck: AI Planning Systems (98% of tick time)

**AI planning consumes 442.2ms out of 451.4ms total tick time.**

The six species-specific planning systems account for nearly all computational load:
- Combined: 442.2ms (98% of tick time)
- Per-entity average: 0.88ms per entity per tick
- With 500 entities: 0.88ms × 500 = 440ms minimum tick time

**Implication**: Even if we optimized everything else to zero, we'd still be limited to ~2.3 TPS (1000ms / 440ms) by AI planning alone.

#### Per-Entity Cost Analysis

**Low-cost entities** (herbivores):
- Rabbits: 0.57ms per entity
- Deer: 0.81ms per entity
- Raccoons: 0.81ms per entity

**High-cost entities** (predators):
- Wolves: 2.82ms per entity
- Bears: 2.85ms per entity
- Foxes: 0.86ms per entity

**Observation**: Predator AI is ~3-5x more expensive than herbivore AI, likely due to:
- Hunt behavior complexity
- Prey tracking and selection
- Pack coordination (wolves)
- More complex decision trees

#### Other Systems (2% of tick time)

- `ai_actions`: 4.4ms - Action execution (separate from planning)
- `chunk_aggregation`: 3.2ms - World chunking
- `heatmap_snapshot`: 0.9ms - Visualization data
- `chunk_lod`: 0.6ms - Level of detail
- Everything else: ~0.0ms (optimized in Phases 1-3)

**Status**: Non-AI systems are highly optimized. Further optimization here would yield minimal gains.

---

## Root Cause Analysis

### Why AI Planning is So Expensive

Based on the codebase architecture (event-driven planner):

1. **Per-Tick Decision Making**
   - Every entity runs planning logic every tick
   - No decision caching or reuse
   - Full state evaluation each time

2. **Event-Driven Planner Complexity**
   - Multiple trigger systems
   - State evaluation per trigger
   - Action queue processing
   - Plan generation and validation

3. **Predator-Specific Overhead**
   - Prey detection and tracking
   - Hunt coordination
   - Pack behavior (wolves)
   - More complex goal hierarchies

4. **No Batching or Throttling**
   - All 500 entities plan every tick
   - No staggered updates
   - No decision frequency reduction

---

## Performance Gap Calculation

### Current State
- **Current TPS**: 0.6-0.8 (average 0.7)
- **Current tick time**: ~1430ms
- **AI planning time**: 442ms
- **Other systems time**: 9ms
- **Unaccounted time**: ~979ms (likely pathfinding, world queries, system overhead)

### Target State
- **Target TPS**: 10.0
- **Target tick time**: 100ms
- **Required speedup**: 14.3x

### AI Planning Optimization Requirements

To reach 10 TPS (100ms per tick):
- **Current AI time**: 442ms
- **Target AI budget**: ~40ms (assuming 60ms for other systems)
- **Required AI speedup**: 11x

---

## Optimization Strategy

### Phase 5A: AI Planning Optimization (Primary Focus)

#### 1. Decision Frequency Reduction (Estimated: 5-10x speedup)

**Concept**: Not every entity needs to replan every tick.

**Implementation**:
```rust
// Add decision timer to entities
pub struct DecisionTimer {
    ticks_until_replan: u32,
    replan_interval: u32,  // e.g., 10 ticks
}

// Only replan when timer expires
fn plan_entity_actions(
    mut query: Query<(&mut DecisionTimer, &Creature, ...)>,
) {
    for (mut timer, creature, ...) in query.iter_mut() {
        if timer.ticks_until_replan > 0 {
            timer.ticks_until_replan -= 1;
            continue;  // Skip planning this tick
        }

        // Run planning
        let plan = generate_plan(creature, ...);

        // Reset timer based on plan duration
        timer.ticks_until_replan = plan.expected_duration;
    }
}
```

**Expected Impact**:
- If entities replan every 10 ticks instead of every tick: **10x reduction**
- With dynamic intervals (longer for long-term plans): **5-15x reduction**
- 442ms → 44-88ms

**Effort**: Medium (modify each species planner)

#### 2. Decision Caching (Estimated: 2-3x additional speedup)

**Concept**: Cache and reuse recent decisions for similar situations.

**Implementation**:
```rust
pub struct DecisionCache {
    last_state: EntityState,
    last_decision: Action,
    cache_validity: u32,  // ticks remaining
}

fn plan_with_cache(
    state: &EntityState,
    cache: &mut DecisionCache,
) -> Action {
    // Check if cached decision is still valid
    if cache.cache_validity > 0 && state.similar_to(&cache.last_state) {
        cache.cache_validity -= 1;
        return cache.last_decision.clone();
    }

    // Generate new decision
    let decision = expensive_planning(state);
    cache.last_state = state.clone();
    cache.last_decision = decision.clone();
    cache.cache_validity = 20;  // Valid for 20 ticks

    decision
}
```

**Expected Impact**:
- Avoid redundant planning for repetitive behaviors (grazing, wandering)
- Combined with frequency reduction: **2-3x additional speedup**

**Effort**: Medium-High (requires state similarity detection)

#### 3. Simplified Decision Trees (Estimated: 1.5-2x speedup)

**Concept**: Reduce complexity of decision-making logic.

**Targets**:
- **Predators**: Simplify hunt target selection (currently expensive)
- **All species**: Reduce number of triggers evaluated
- **State evaluation**: Cache frequently accessed state

**Implementation**:
- Limit prey search radius
- Reduce number of candidate targets evaluated
- Skip low-priority triggers when high-priority action is available

**Expected Impact**:
- Per-entity cost reduction: **1.5-2x**
- Especially effective for predators (2.8ms → 1.4-1.9ms)

**Effort**: Low-Medium (simplify existing logic)

#### 4. Batch Processing (Estimated: 1.2-1.5x speedup)

**Concept**: Process similar entities together to improve cache locality.

**Implementation**:
```rust
// Instead of 6 separate planning systems, use one with batching
fn plan_all_species_batch(
    rabbits: Query<..., With<Rabbit>>,
    deer: Query<..., With<Deer>>,
    // ...
) {
    // Process all rabbits together (cache friendly)
    for rabbit in rabbits.iter() {
        plan_herbivore(rabbit, HerbivoreType::Rabbit);
    }

    // Process all deer together
    for deer in deer.iter() {
        plan_herbivore(deer, HerbivoreType::Deer);
    }

    // etc.
}
```

**Expected Impact**:
- Better CPU cache utilization: **1.2-1.5x**
- Reduced system scheduling overhead

**Effort**: Medium (consolidate planner systems)

### Combined AI Optimization Potential

Conservative estimate (multiplicative):
- Decision frequency reduction: 5x
- Decision caching: 2x (additional)
- Simplified logic: 1.5x (additional)
- Batch processing: 1.2x (additional)

**Total: 5 × 2 × 1.5 × 1.2 = 18x speedup**

With 18x speedup:
- Current: 442ms
- Optimized: 24.6ms ✅ (well within 40ms budget)

### Phase 5B: Secondary Optimizations (If Needed)

#### 1. Pathfinding Optimization

**Current Status**: Not showing in profiler (likely counted in unaccounted time)

**Potential Approaches**:
- Path caching (reuse recent paths)
- Hierarchical pathfinding (coarse + fine grid)
- Limit pathfinding frequency

**Expected Impact**: 2-3x speedup on pathfinding
**Effort**: High

#### 2. Action Queue Optimization

**Current**: 4.4ms (1% of tick time) - already efficient

**If needed**:
- Batch similar actions
- Priority-based processing

**Expected Impact**: 2x speedup → 2.2ms
**Effort**: Low

---

## Recommended Implementation Plan

### Iteration 1: Quick Wins (Highest ROI)

**Focus**: Decision frequency reduction + Simplified logic

**Steps**:
1. Add DecisionTimer component to all entities
2. Modify each species planner to check timer before planning
3. Set conservative replan intervals (10 ticks for herbivores, 5 for predators)
4. Simplify predator target selection (limit candidates)
5. Run performance test

**Expected Improvement**: 5-7x speedup → 63-88ms tick time → 11-16 TPS ✅

**Effort**: 2-3 hours
**Test**: Run 500 entity load test, verify TPS > 10

### Iteration 2: Refinement (If Needed)

**Focus**: Decision caching

**Steps**:
1. Add DecisionCache component
2. Implement state similarity detection
3. Integrate caching into planners
4. Tune cache validity periods

**Expected Improvement**: Additional 2x → 31-44ms tick time → 23-32 TPS ✅

**Effort**: 3-4 hours
**Test**: Run 500 entity load test, verify sustained TPS > 10

### Iteration 3: Polish (Optional)

**Focus**: Batch processing

**Steps**:
1. Consolidate species planners
2. Implement batched processing
3. Profile and verify improvements

**Expected Improvement**: Additional 1.2x
**Effort**: 2-3 hours

---

## Success Criteria

### Minimum Viable Success
- ✅ Achieve 10 TPS sustained with 500 entities
- ✅ All entities continue to behave correctly
- ✅ No regressions in existing functionality
- ✅ Tests passing

### Stretch Goals
- 🎯 Achieve 15+ TPS with 500 entities
- 🎯 Support 750-1000 entities at 10 TPS
- 🎯 Reduce predator AI cost to match herbivores

---

## Risk Assessment

### Low Risk
- **Decision frequency reduction**: Simple, well-understood technique
- **Simplified logic**: Reduces complexity, easier to debug
- **All changes localized to AI systems**: Won't affect phases 1-3 optimizations

### Medium Risk
- **Decision caching**: Requires careful state comparison
- **Batch processing**: May complicate system architecture

### Mitigation
- Implement incrementally with performance tests after each change
- Maintain comprehensive test suite
- Profile after each optimization to verify gains
- Keep git commits atomic for easy rollback

---

## Next Steps

1. ✅ **Create this profiling analysis document** - DONE
2. ⏭️ **Implement Iteration 1** (Decision frequency + Simplified logic)
3. ⏭️ **Test and measure** performance improvement
4. ⏭️ **Iterate** if target not met
5. ⏭️ **Document** final optimizations

---

## Appendix: Raw Profiling Data

### Tick 50 Full Report
```
🔧 TICK PERFORMANCE - Tick 50 | Total: 451.4ms
├── plan_rabbit_actions:  107.8ms ( 24%)
├── plan_deer_actions:   97.0ms ( 21%)
├── plan_raccoon_actions:   81.0ms ( 18%)
├── plan_wolf_actions:   70.5ms ( 16%)
├── plan_fox_actions:   43.1ms ( 10%)
├── plan_bear_actions:   42.8ms (  9%)
├── ai_actions     :    4.4ms (  1%)
├── chunk_aggregation:    3.2ms (  1%)
├── heatmap_snapshot:    0.9ms (  0%)
├── chunk_lod      :    0.6ms (  0%)
└── AVG TOTAL: 444.3ms over 21 systems
```

### TPS Measurements
```
Tick 18: 0.8 TPS, 500 entities
Tick 20: 0.7 TPS, 500 entities
Tick 23: 0.6 TPS, 500 entities
Tick 50: 0.4 TPS, 500 entities (profiling overhead)
Tick 63: ~0.7 TPS, 500 entities
```

---

**Analysis Complete**: 2025-12-26
**Phase 5 Status**: Profiling complete, optimization plan ready
**Next**: Implement Iteration 1 optimizations
