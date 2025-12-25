# Performance Optimization Plan: Vegetation & Entity Systems

## Executive Summary

This document provides a comprehensive performance analysis and optimization roadmap for the life simulator's vegetation (ResourceGrid) and entity state management systems. Based on code analysis, we've identified key bottlenecks and optimization opportunities with an incremental implementation strategy.

**Current Performance Status:**
- ✅ **Vegetation System**: Event-driven ResourceGrid with ~145μs processing time (target: <2000μs)
- ✅ **Benchmark Infrastructure**: Comprehensive testing framework already in place
- ⚠️ **Entity State Access**: Query-heavy patterns with potential cache misses
- ⚠️ **Memory Efficiency**: HashMap-based storage with optimization potential

---

## 1. System Architecture Analysis

### 1.1 Vegetation System (ResourceGrid)

**Current Architecture (Phase 3: Event-Driven):**
```rust
// Sparse storage - only cells with biomass
cells: HashMap<IVec2, GrazingCell>

// Event scheduler for regrowth
event_scheduler: VegetationScheduler
  ├── event_queue: BinaryHeap<Reverse<ScheduledEvent>>
  └── random_tick_budget: 50 cells/tick
```

**Performance Characteristics:**
- **Sparse storage**: Only stores ~156 active cells out of potentially millions
- **Event-driven updates**: Processes only cells with pending events
- **Target budget**: 2000μs per tick (currently ~145μs ✅)
- **Memory footprint**: ~35-40 bytes per active cell

**Key Strengths:**
1. Event-driven approach eliminates per-tick loops
2. Sparse HashMap only stores cells with biomass
3. Priority queue ensures proper event ordering
4. Already meeting performance targets

**Optimization Opportunities:**
1. **Memory layout**: HashMap causes cache misses during spatial queries
2. **Random sampling**: 50 cells/tick could benefit from spatial locality
3. **Event batch processing**: Process related events together
4. **LOD system integration**: chunk_lod.rs exists but underutilized

---

### 1.2 Entity State Management

**Current Architecture:**
```rust
// Component-based queries
Query<(&TilePosition, &Hunger, &Thirst, &Energy), With<Species>>

// Multiple queries per planning cycle
- Position lookups: HashMap<Entity, IVec2>
- Stat access: Individual component queries
- Fear system: Separate proximity checks
```

**Performance Characteristics:**
- **Query overhead**: Multiple Bevy queries per entity per tick
- **Cache locality**: Components scattered in memory
- **Lookup patterns**: HashMap for position, then component access
- **Update frequency**: 10 TPS with ~150 entities

**Key Bottlenecks:**
1. **Multiple query iterations**: Planning system queries same entity data multiple times
2. **HashMap position lookups**: O(1) but with hash overhead per lookup
3. **Component fragmentation**: Stats spread across multiple component pools
4. **Emergency checks**: Repeated threshold calculations per entity

**Optimization Opportunities:**
1. **Cached entity state**: Bundle frequently accessed data
2. **Spatial indexing**: Grid-based lookup for proximity queries
3. **Batch processing**: Process entities by chunk/region
4. **Stat normalization caching**: Pre-compute urgency values

---

## 2. Performance Benchmarking

### 2.1 Existing Benchmark Infrastructure

**Already Implemented** (`src/vegetation/benchmark.rs`):
```rust
BenchmarkRunner {
  - Duration-based testing (5s quick, 15s comprehensive)
  - Real-time performance monitoring
  - Budget compliance tracking (1ms target)
  - Efficiency ratings (Excellent/Good/Fair/Poor)
}

PerformanceMonitor {
  - Ring buffer for tick/growth times
  - Statistical analysis (avg, max, stddev)
  - CPU utilization tracking
}
```

**Current Metrics:**
- ✅ Avg Growth Time: ~850μs (target: <1000μs)
- ✅ Budget Compliance: 95%+
- ✅ Efficiency Rating: Excellent

### 2.2 Benchmark Enhancements Needed

#### Entity State Benchmark Suite

**Create:** `tests/entity_state_benchmark.rs`

```rust
pub struct EntityStateBenchmark {
    // Test scenarios
    - entity_query_overhead()      // Measure Bevy query cost
    - position_lookup_performance() // HashMap vs spatial grid
    - stat_access_patterns()       // Component access overhead
    - planning_cycle_timing()      // Full AI planning cost

    // Optimization validation
    - cached_state_comparison()    // Before/after caching
    - spatial_grid_speedup()       // HashMap vs grid lookup
    - batch_processing_gains()     // Sequential vs batched
}
```

#### Integrated System Benchmark

**Create:** `tests/integrated_performance.rs`

```rust
pub struct IntegratedBenchmark {
    // Full simulation scenarios
    - idle_world_baseline()        // No entities, just vegetation
    - low_entity_load()            // 50 entities
    - medium_entity_load()         // 150 entities (current)
    - high_entity_load()           // 300+ entities
    - stress_test()                // 500+ entities

    // Specific workloads
    - heavy_grazing_scenario()     // Multiple herbivores per chunk
    - predator_chase_scenario()    // Fear system + movement
    - reproduction_peak()          // Birth + mating queries
}
```

### 2.3 Profiling Strategy

**Real-world Profiling:**
```bash
# CPU profiling with flamegraph
cargo flamegraph --bin life-simulator

# Memory profiling with heaptrack
heaptrack cargo run --bin life-simulator

# Sampling profiler for hotspots
cargo install samply
samply record cargo run --release --bin life-simulator
```

**Key Metrics to Track:**
1. **Per-tick timing**: Total, vegetation, AI, movement, physics
2. **Cache miss rate**: L1/L2/L3 cache efficiency
3. **Memory allocations**: HashMap resizing, vector growth
4. **Query iteration cost**: Bevy archetype iteration overhead

---

## 3. Optimization Opportunities

### 3.1 Vegetation System Optimizations

#### Opportunity 1: Spatial Grid for Queries

**Current (HashMap-based):**
```rust
// O(radius²) HashMap lookups for find_best_cell()
for dx in -radius..=radius {
    for dy in -radius..=radius {
        if let Some(cell) = self.get_cell(pos) {  // HashMap lookup each iteration
```

**Optimized (Spatial Grid):**
```rust
pub struct SpatialGrid {
    chunks: HashMap<IVec2, Vec<GrazingCell>>,  // Chunked storage
    chunk_size: i32,
}

impl SpatialGrid {
    pub fn find_cells_in_radius(&self, center: IVec2, radius: i32) -> Vec<&GrazingCell> {
        // Calculate affected chunks (O(1-4) chunk lookups)
        // Iterate only cells in those chunks
        // 10-50x faster for radius queries
    }
}
```

**Expected Gains:**
- **Radius queries**: 10-50x faster (especially for large radius)
- **Cache locality**: Better memory access patterns
- **Scaling**: Constant-time chunk lookup regardless of world size

**Implementation Complexity:** Medium (2-3 days)

---

#### Opportunity 2: Event Batch Processing

**Current (Individual Events):**
```rust
for event in due_events {
    match event {
        GrowthEvent::Regrow { location, .. } => {
            self.regrow_cell(location);  // Individual cell update
        }
    }
}
```

**Optimized (Batch by Chunk):**
```rust
// Group events by spatial proximity
let batches = group_events_by_chunk(due_events);

for (chunk, events) in batches {
    // Process all events in chunk together
    // Better cache locality
    // Vectorizable growth calculations
    process_chunk_batch(chunk, events);
}
```

**Expected Gains:**
- **Cache efficiency**: 2-3x better for clustered events
- **Branch prediction**: More predictable access patterns
- **SIMD potential**: Vectorize growth calculations

**Implementation Complexity:** Low (1 day)

---

#### Opportunity 3: LOD-Based Update Frequency

**Current (Uniform Updates):**
```rust
// All cells update at same frequency
calculate_regrowth_interval(biomass_fraction)  // 20-100 ticks
```

**Optimized (Distance-Based LOD):**
```rust
pub enum UpdateTier {
    Hot,   // Near entities, 20 tick interval
    Warm,  // Medium distance, 50 tick interval
    Cold,  // Far from entities, 100 tick interval
    Frozen // Very far, no updates (impostor data)
}

fn calculate_lod_interval(biomass_fraction: f32, distance_to_nearest_entity: f32) -> u64 {
    let base_interval = calculate_regrowth_interval(biomass_fraction);
    let lod_multiplier = match distance_to_nearest_entity {
        d if d < 20.0 => 1.0,   // Hot
        d if d < 50.0 => 2.0,   // Warm
        d if d < 100.0 => 5.0,  // Cold
        _ => f32::INFINITY,     // Frozen (no updates)
    };
    (base_interval as f32 * lod_multiplier) as u64
}
```

**Expected Gains:**
- **Processing reduction**: 60-80% fewer events for large worlds
- **Scalability**: Performance independent of world size
- **Visual quality**: No perceptible difference (distant cells rarely visible)

**Implementation Complexity:** Medium (2-3 days, integrates with existing chunk_lod.rs)

---

### 3.2 Entity State Optimizations

#### Opportunity 4: Cached Entity State Bundle

**Current (Multiple Queries):**
```rust
// AI planning system
for (entity, position, thirst, hunger, energy, ...) in query.iter() {
    // Position lookup
    let entity_pos = position_lookup.get(&entity);  // HashMap lookup

    // Multiple component accesses
    let hunger_urgency = hunger.urgency();  // Compute each time
    let thirst_urgency = thirst.urgency();
    let energy_urgency = energy.urgency();

    // Threshold checks
    let emergency = hunger.0.normalized() >= 0.85
                 || thirst.0.normalized() >= 0.85
                 || energy.0.normalized() <= 0.15;
}
```

**Optimized (Cached State):**
```rust
#[derive(Component)]
pub struct CachedEntityState {
    // Position (avoid HashMap lookup)
    pub tile: IVec2,

    // Pre-computed urgencies (avoid repeated calculations)
    pub hunger_urgency: f32,
    pub thirst_urgency: f32,
    pub energy_urgency: f32,

    // Pre-computed flags
    pub is_emergency: bool,
    pub is_juvenile: bool,
    pub can_mate: bool,

    // Dirty flag for invalidation
    pub dirty: bool,
    pub last_update_tick: u64,
}

// Update system (runs early in tick)
fn update_cached_entity_state(
    mut query: Query<(&TilePosition, &Hunger, &Thirst, &Energy, &mut CachedEntityState)>
) {
    for (pos, hunger, thirst, energy, mut cached) in query.iter_mut() {
        if cached.dirty {
            cached.tile = pos.tile;
            cached.hunger_urgency = hunger.urgency();
            cached.thirst_urgency = thirst.urgency();
            cached.energy_urgency = energy.urgency();
            cached.is_emergency = hunger.0.normalized() >= 0.85
                               || thirst.0.normalized() >= 0.85
                               || energy.0.normalized() <= 0.15;
            cached.dirty = false;
        }
    }
}

// AI planning uses cached data
fn plan_actions(query: Query<(&CachedEntityState, &BehaviorConfig)>) {
    for (cached, behavior) in query.iter() {
        if cached.is_emergency {
            // Use pre-computed values
            let urgency = cached.hunger_urgency.max(cached.thirst_urgency);
        }
    }
}
```

**Expected Gains:**
- **Query overhead**: 40-60% reduction (single component vs multiple)
- **Computation**: Eliminate repeated urgency() calculations
- **Cache locality**: All frequently-accessed data in one component
- **Branch prediction**: Pre-computed booleans reduce conditionals

**Implementation Complexity:** Medium (2-3 days)

**Trade-offs:**
- Memory: +24 bytes per entity (~4KB for 150 entities)
- Invalidation: Must mark dirty on stat changes
- Synchronization: Potential staleness if not updated properly

---

#### Opportunity 5: Spatial Entity Index

**Current (Linear Proximity Search):**
```rust
// Fear system proximity check
fn find_nearest_predator(entity_pos: IVec2, all_predators: &[(Entity, IVec2)]) -> Option<Entity> {
    // O(N) linear search through all predators
    all_predators.iter()
        .min_by_key(|(_, pred_pos)| {
            entity_pos.as_vec2().distance(pred_pos.as_vec2())
        })
}
```

**Optimized (Spatial Grid Index):**
```rust
#[derive(Resource)]
pub struct SpatialEntityIndex {
    grid: HashMap<IVec2, Vec<(Entity, EntityType)>>,
    chunk_size: i32,
}

impl SpatialEntityIndex {
    pub fn entities_in_radius(&self, center: IVec2, radius: i32, filter: EntityType)
        -> impl Iterator<Item = Entity>
    {
        // O(1) chunk lookup + O(k) where k = entities in nearby chunks
        let affected_chunks = self.get_chunks_in_radius(center, radius);
        affected_chunks.flat_map(|chunk| {
            self.grid.get(&chunk)
                .into_iter()
                .flatten()
                .filter(move |(_, ty)| *ty == filter)
                .map(|(e, _)| *e)
        })
    }
}
```

**Expected Gains:**
- **Proximity queries**: 10-100x faster (O(k) vs O(N))
- **Fear system**: Near-instant predator detection
- **Mate finding**: Fast potential partner searches
- **Scalability**: Performance independent of total entity count

**Implementation Complexity:** Medium-High (3-4 days)

**Maintenance:**
- Must update on entity movement
- Chunk transitions need special handling
- Memory overhead: ~8 bytes per entity per chunk

---

#### Opportunity 6: Batch Entity Processing

**Current (Random Iteration Order):**
```rust
// Entities processed in arbitrary archetype order
for (entity, position, stats...) in query.iter() {
    plan_actions(entity, position, stats);
}
```

**Optimized (Spatial Batching):**
```rust
// Group entities by chunk
let entity_batches = group_entities_by_chunk(&query);

for (chunk, entities) in entity_batches {
    // Process all entities in chunk together
    // Better cache locality for vegetation queries
    // Can reuse chunk data
    process_entity_batch(chunk, entities);
}
```

**Expected Gains:**
- **Cache efficiency**: 2-4x better for vegetation queries
- **Data reuse**: Load chunk data once for all entities
- **Memory bandwidth**: Fewer cache line loads

**Implementation Complexity:** Low-Medium (1-2 days)

---

## 4. Incremental Implementation Roadmap

### Phase 1: Quick Wins (Week 1)
**Goal**: 20-30% performance improvement with minimal risk

**Tasks:**
1. **Event Batch Processing** (1 day)
   - Group vegetation events by chunk
   - Process batches together for cache locality
   - Benchmark: Measure growth time reduction

2. **Cached Entity State** (2-3 days)
   - Implement CachedEntityState component
   - Add update system with dirty flag
   - Migrate AI planning to use cached data
   - Benchmark: Measure query overhead reduction

3. **Benchmark Suite Expansion** (1-2 days)
   - Create entity_state_benchmark.rs
   - Add planning cycle timing tests
   - Establish baseline metrics

**Validation:**
- Run existing vegetation benchmarks
- Verify no regression in budget compliance
- Measure cache miss rate improvements

---

### Phase 2: Spatial Optimizations (Week 2-3)
**Goal**: 40-60% improvement in spatial queries

**Tasks:**
1. **Spatial Entity Index** (3-4 days)
   - Implement grid-based entity index
   - Integrate with movement system
   - Update fear system to use index
   - Benchmark: Proximity query speedup

2. **Vegetation Spatial Grid** (2-3 days)
   - Convert ResourceGrid to chunked storage
   - Optimize find_best_cell() with spatial queries
   - Benchmark: Radius query performance

3. **Batch Entity Processing** (1-2 days)
   - Group entities by chunk for planning
   - Process batches sequentially
   - Benchmark: Cache efficiency gains

**Validation:**
- Integrated system benchmark with 300+ entities
- Memory profiling to verify overhead acceptable
- Visual testing to confirm behavior unchanged

---

### Phase 3: LOD & Advanced Optimizations (Week 4)
**Goal**: Scalability to 500+ entities and large worlds

**Tasks:**
1. **LOD-Based Update Frequency** (2-3 days)
   - Integrate with existing chunk_lod.rs
   - Distance-based update tiers
   - Frozen cells for distant regions
   - Benchmark: Event reduction metrics

2. **Memory Optimization** (1-2 days)
   - Profile memory usage patterns
   - Optimize HashMap configurations (capacity hints)
   - Consider u16 biomass storage (existing analysis in memory_optimization.rs)

3. **Comprehensive Profiling** (2 days)
   - Flamegraph analysis
   - Cache miss rate measurements
   - Memory allocation profiling
   - Generate optimization report

**Validation:**
- Stress test with 500+ entities
- Large world performance (100x100 chunks)
- Memory footprint analysis
- Final benchmark report

---

## 5. Benchmark Specifications

### 5.1 Entity State Benchmark

**Test File:** `tests/entity_state_benchmark.rs`

```rust
#[test]
fn benchmark_entity_query_overhead() {
    // Measures: Time to query all entity stats
    // Baseline: Current multi-component queries
    // Optimized: CachedEntityState queries
    // Expected: 40-60% reduction
}

#[test]
fn benchmark_position_lookup() {
    // Measures: Position lookup performance
    // Baseline: HashMap<Entity, IVec2>
    // Optimized: Direct component access or cached
    // Expected: 50-70% reduction
}

#[test]
fn benchmark_planning_cycle() {
    // Measures: Full AI planning cycle time
    // Baseline: Current implementation
    // Optimized: With all entity optimizations
    // Expected: 30-50% total reduction
}

#[test]
fn benchmark_proximity_queries() {
    // Measures: Find nearest predator/mate
    // Baseline: Linear O(N) search
    // Optimized: Spatial grid O(k) search
    // Expected: 10-100x improvement
}
```

### 5.2 Integrated System Benchmark

**Test File:** `tests/integrated_performance.rs`

```rust
#[test]
fn benchmark_idle_world() {
    // Scenario: 0 entities, vegetation only
    // Target: <500μs per tick
    // Measures: Vegetation baseline overhead
}

#[test]
fn benchmark_low_entity_load() {
    // Scenario: 50 entities, normal behavior
    // Target: <2000μs per tick
    // Measures: Entity + vegetation interaction
}

#[test]
fn benchmark_medium_entity_load() {
    // Scenario: 150 entities (current production)
    // Target: <3000μs per tick
    // Measures: Current performance baseline
}

#[test]
fn benchmark_high_entity_load() {
    // Scenario: 300 entities, heavy interaction
    // Target: <5000μs per tick
    // Measures: Scalability limits
}

#[test]
fn benchmark_stress_test() {
    // Scenario: 500+ entities, worst-case
    // Target: <10000μs per tick
    // Measures: Maximum capacity
}
```

### 5.3 Workload-Specific Benchmarks

```rust
#[test]
fn benchmark_heavy_grazing() {
    // Scenario: Multiple herbivores per vegetation chunk
    // Measures: Vegetation query + event generation overhead
    // Target: Graceful degradation under contention
}

#[test]
fn benchmark_predator_chase() {
    // Scenario: Predators pursuing prey with fear system
    // Measures: Proximity query + pathfinding overhead
    // Target: Smooth fear response at 10 TPS
}

#[test]
fn benchmark_reproduction_peak() {
    // Scenario: Multiple births + mate searching
    // Measures: Entity spawning + mate query overhead
    // Target: No frame drops during population boom
}
```

---

## 6. Success Metrics

### 6.1 Performance Targets

| System | Current | Target | Stretch Goal |
|--------|---------|--------|--------------|
| Vegetation (idle) | 145μs | <500μs | <200μs |
| Entity planning (150 entities) | ~2000μs | <1500μs | <1000μs |
| Proximity queries | O(N) | O(k) | Constant-time |
| Memory per entity | ~100 bytes | <150 bytes | <120 bytes |
| Max entities (10 TPS) | 150 | 300 | 500+ |

### 6.2 Quality Metrics

| Metric | Target |
|--------|--------|
| Budget compliance | >95% |
| Cache miss rate | <10% |
| Frame time variance | <2ms stddev |
| Memory growth rate | <1MB/min |
| Visual consistency | 100% (no behavior changes) |

---

## 7. Risk Mitigation

### 7.1 Technical Risks

**Risk 1: Cache Invalidation Bugs**
- *Mitigation*: Comprehensive testing of dirty flag propagation
- *Fallback*: Feature flag to disable caching

**Risk 2: Spatial Index Overhead**
- *Mitigation*: Incremental rollout, benchmark before full integration
- *Fallback*: Keep HashMap-based path available

**Risk 3: Regression in Existing Systems**
- *Mitigation*: Extensive regression testing, visual diff tests
- *Fallback*: Git revert with clear commit boundaries

### 7.2 Schedule Risks

**Risk**: Optimization takes longer than estimated
- *Mitigation*: Incremental phases with independent value
- *Contingency*: Each phase delivers measurable improvement

**Risk**: Unforeseen interactions between optimizations
- *Mitigation*: Benchmark after each optimization
- *Contingency*: Rollback individual optimizations if conflicts arise

---

## 8. Appendix: Code Examples

### A1. Spatial Grid Implementation Template

```rust
pub mod spatial_grid {
    use bevy::prelude::*;
    use std::collections::HashMap;

    const CHUNK_SIZE: i32 = 16;

    #[derive(Resource)]
    pub struct EntitySpatialIndex {
        chunks: HashMap<IVec2, Vec<(Entity, EntityType)>>,
    }

    impl EntitySpatialIndex {
        pub fn new() -> Self {
            Self {
                chunks: HashMap::with_capacity(1024),
            }
        }

        fn world_to_chunk(pos: IVec2) -> IVec2 {
            IVec2::new(pos.x.div_euclid(CHUNK_SIZE), pos.y.div_euclid(CHUNK_SIZE))
        }

        pub fn insert(&mut self, entity: Entity, pos: IVec2, entity_type: EntityType) {
            let chunk = Self::world_to_chunk(pos);
            self.chunks
                .entry(chunk)
                .or_insert_with(Vec::new)
                .push((entity, entity_type));
        }

        pub fn remove(&mut self, entity: Entity, old_pos: IVec2) {
            let chunk = Self::world_to_chunk(old_pos);
            if let Some(entities) = self.chunks.get_mut(&chunk) {
                entities.retain(|(e, _)| *e != entity);
            }
        }

        pub fn update(&mut self, entity: Entity, old_pos: IVec2, new_pos: IVec2, entity_type: EntityType) {
            let old_chunk = Self::world_to_chunk(old_pos);
            let new_chunk = Self::world_to_chunk(new_pos);

            if old_chunk != new_chunk {
                self.remove(entity, old_pos);
                self.insert(entity, new_pos, entity_type);
            }
        }

        pub fn entities_in_radius(
            &self,
            center: IVec2,
            radius: i32,
            filter: Option<EntityType>,
        ) -> Vec<Entity> {
            let chunk_radius = (radius + CHUNK_SIZE - 1) / CHUNK_SIZE;
            let center_chunk = Self::world_to_chunk(center);

            let mut results = Vec::new();

            for dx in -chunk_radius..=chunk_radius {
                for dy in -chunk_radius..=chunk_radius {
                    let chunk = center_chunk + IVec2::new(dx, dy);
                    if let Some(entities) = self.chunks.get(&chunk) {
                        for (entity, entity_type) in entities {
                            if filter.is_none() || Some(*entity_type) == filter {
                                results.push(*entity);
                            }
                        }
                    }
                }
            }

            results
        }
    }
}
```

### A2. Cached Entity State Template

```rust
pub mod cached_state {
    use bevy::prelude::*;

    #[derive(Component)]
    pub struct CachedEntityState {
        pub tile: IVec2,
        pub hunger_urgency: f32,
        pub thirst_urgency: f32,
        pub energy_urgency: f32,
        pub is_emergency: bool,
        pub dirty: bool,
        pub last_update_tick: u64,
    }

    impl Default for CachedEntityState {
        fn default() -> Self {
            Self {
                tile: IVec2::ZERO,
                hunger_urgency: 0.0,
                thirst_urgency: 0.0,
                energy_urgency: 0.0,
                is_emergency: false,
                dirty: true,
                last_update_tick: 0,
            }
        }
    }

    pub fn update_cached_state_system(
        mut query: Query<
            (&TilePosition, &Hunger, &Thirst, &Energy, &mut CachedEntityState),
            Changed<TilePosition> // Or use manual dirty flag
        >,
        tick: Res<SimulationTick>,
    ) {
        for (pos, hunger, thirst, energy, mut cached) in query.iter_mut() {
            cached.tile = pos.tile;
            cached.hunger_urgency = hunger.urgency();
            cached.thirst_urgency = thirst.urgency();
            cached.energy_urgency = energy.urgency();

            let hunger_norm = hunger.0.normalized();
            let thirst_norm = thirst.0.normalized();
            let energy_norm = energy.0.normalized();

            cached.is_emergency = hunger_norm >= 0.85
                               || thirst_norm >= 0.85
                               || energy_norm <= 0.15;

            cached.dirty = false;
            cached.last_update_tick = tick.0;
        }
    }
}
```

---

## 9. Next Steps

### Immediate Actions (This Week)

1. **Review & Approval**
   - Share this document with team
   - Discuss priorities and timeline
   - Confirm success metrics

2. **Establish Baselines**
   - Run existing vegetation benchmarks
   - Document current performance numbers
   - Create baseline report

3. **Set Up Phase 1**
   - Create feature branch: `perf/entity-vegetation-opt`
   - Set up benchmark infrastructure
   - Configure profiling tools

### Long-term Monitoring

1. **Continuous Performance Tracking**
   - Integrate benchmarks into CI/CD
   - Weekly performance reports
   - Regression detection

2. **Future Optimizations**
   - SIMD vectorization for batch operations
   - Multi-threading for independent entity batches
   - GPU compute for massive entity counts (future consideration)

---

**Document Version:** 1.0
**Last Updated:** 2025-12-25
**Status:** Awaiting Review
**Owner:** Performance Optimization Team
