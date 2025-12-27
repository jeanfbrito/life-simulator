# UltraThink Quick Reference

## Phase 1: Core Queue Infrastructure - COMPLETE

### Quick Start

```bash
# Run all ultrathink tests
cargo test ultrathink

# Run integration tests only
cargo test --test ultrathink_queue_test

# Build and run with test harness
ULTRATHINK_TEST=1 RUST_LOG=info cargo run --bin life-simulator

# Build normally
cargo build --bin life-simulator
```

### Core Types

```rust
// Priority levels
pub enum ThinkPriority {
    Urgent,  // 1-2 tick latency - fear, critical needs
    Normal,  // 5-10 tick latency - moderate needs, actions
    Low,     // 20-50 tick latency - idle, exploration
}

// Think reasons (13 variants)
pub enum ThinkReason {
    // Urgent
    FearTriggered,
    HungerCritical,
    ThirstCritical,
    Threatened,

    // Normal
    HungerModerate,
    ThirstModerate,
    ActionCompleted,
    ActionFailed,
    ReproductionReady,

    // Low
    Idle,
    WanderTargetNeeded,
    ExplorationDesired,
    SocialInteraction,
}

// Think request
pub struct ThinkRequest {
    pub entity: Entity,
    pub reason: ThinkReason,
    pub scheduled_tick: u64,
    pub priority: ThinkPriority,
}
```

### ThinkQueue API

```rust
// Get the queue resource
let mut think_queue = world.resource_mut::<ThinkQueue>();

// Schedule requests
think_queue.schedule_urgent(entity, ThinkReason::FearTriggered, tick);
think_queue.schedule_normal(entity, ThinkReason::ActionCompleted, tick);
think_queue.schedule_low(entity, ThinkReason::Idle, tick);

// Process requests (called by ultrathink_system)
let requests = think_queue.drain(50); // Process up to 50

// Query queue state
let (urgent, normal, low) = think_queue.queue_sizes();
let total = think_queue.total_queued();
let processed = think_queue.total_processed();
let is_queued = think_queue.contains(entity);
```

### System Integration

```rust
// Plugin is auto-registered in TQUAIPlugin
impl Plugin for TQUAIPlugin {
    fn build(&self, app: &mut App) {
        // ...
        .add_plugins(UltraThinkPlugin::default())  // ← Added in Phase 1
        // ...
    }
}

// System runs automatically
fn ultrathink_system(
    mut think_queue: ResMut<ThinkQueue>,
    tick: Res<SimulationTick>,
) {
    // Drains up to budget per tick
    // Logs debug per request
    // Logs info every 50 ticks
}
```

### Configuration

```rust
// Default configuration (50 thinks/tick)
.add_plugins(UltraThinkPlugin::default())

// Custom budget
.add_plugins(UltraThinkPlugin {
    thinks_per_tick: 100,
})
```

### Logging

```bash
# See per-request processing
RUST_LOG=debug cargo run --bin life-simulator

# See queue metrics only (every 50 ticks)
RUST_LOG=info cargo run --bin life-simulator
```

Expected output:
```
INFO  🧠 UltraThink Plugin initialized with 50 thinks per tick budget
DEBUG 🧠 Processing think request: entity=Entity(123), reason=FearTriggered, priority=Urgent, wait_time=2 ticks
INFO  🧠 ThinkQueue depth: 12 urgent, 45 normal, 120 low | Processed 50/50 | Total processed: 2500
```

### Test Coverage

```
✅ Unit Tests (2)
  - test_queue_creation
  - test_priority_ordering

✅ Integration Tests (5)
  - test_schedule_requests_to_correct_priority_queues
  - test_drain_respects_priority_order
  - test_drain_respects_budget_limit
  - test_contains_detects_queued_entities
  - test_queue_empty_returns_empty_vec
```

### Files Created

```
src/ai/ultrathink/
├── mod.rs           - Plugin and exports
├── queue.rs         - ThinkQueue resource and system
├── request.rs       - Request types and priorities
└── test_harness.rs  - Manual testing system

tests/
└── ultrathink_queue_test.rs - Integration tests

Documentation:
├── ULTRATHINK_PHASE1_DELIVERY.md - Full delivery report
└── ULTRATHINK_QUICK_REFERENCE.md - This file
```

### Next Steps: Phase 2

**Automatic Scheduling Integration**

1. Fear trigger: `FearState` changes → `schedule_urgent(FearTriggered)`
2. Hunger/Thirst: Critical (<20) → `schedule_urgent()`, Moderate (<50) → `schedule_normal()`
3. Action completion: `ActionQueue` events → `schedule_normal()`
4. Idle scheduler: Every 20 ticks → `schedule_low(Idle)` for idle entities

### Common Patterns

```rust
// Example: Fear trigger integration (Phase 2)
fn fear_trigger_system(
    mut think_queue: ResMut<ThinkQueue>,
    query: Query<(Entity, &FearState), Changed<FearState>>,
    tick: Res<SimulationTick>,
) {
    for (entity, fear) in query.iter() {
        if fear.is_fearful() {
            think_queue.schedule_urgent(entity, ThinkReason::FearTriggered, tick.0);
        }
    }
}
```

### Performance Characteristics

```
Memory: O(N) where N = queued entities (typical: 10-50KB)
Time:
  - schedule_*: O(1) amortized
  - drain(N): O(N)
  - contains: O(1)

Throughput:
  - 50 thinks/tick @ 10 TPS = 500 thinks/sec
  - Sufficient for 500-1000 entities
```

### Troubleshooting

**Q: No logging output?**
A: Ensure `RUST_LOG=info` or `RUST_LOG=debug` is set

**Q: Queue not processing?**
A: Check that simulation is ticking (`should_tick` must be true)

**Q: Entities not being scheduled?**
A: Phase 1 only has infrastructure - Phase 2 adds automatic scheduling

**Q: How to test manually?**
A: Set `ULTRATHINK_TEST=1` to enable test harness

---

**Status**: Phase 1 Complete ✅
**Tests**: 7 passing (2 unit + 5 integration)
**Next**: Phase 2 - Automatic Scheduling Integration
