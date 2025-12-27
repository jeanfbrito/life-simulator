# System Sets Quick Reference

**Phase**: 6 - System Organization and Parallelism
**Status**: COMPLETE ✅
**TPS**: 10.0 (maintained)

---

## System Set Enum

```rust
use crate::simulation::SimulationSet;

pub enum SimulationSet {
    Planning,         // AI decision making (parallel)
    ActionExecution,  // Execute actions (sequential, World access)
    Movement,         // Execute movement (parallel)
    Stats,           // Update stats (parallel)
    Reproduction,    // Mate matching, births (parallel)
    Cleanup,         // Death, cleanup (sequential, must run last)
}
```

---

## Execution Order

```
Planning → ActionExecution → Movement → Stats/Reproduction → Cleanup
```

---

## Adding Systems to Sets

### Example: Add new species planner
```rust
.add_systems(Update, (
    plan_new_species_actions,
).in_set(SimulationSet::Planning).run_if(should_run_tick_systems))
```

### Example: Add new reproduction system
```rust
.add_systems(Update, (
    new_species_mate_matching_system,
    new_species_birth_system,
).in_set(SimulationSet::Reproduction).after(SimulationSet::Movement).run_if(should_run_tick_systems))
```

### Example: Add cleanup system
```rust
.add_systems(Update, (
    cleanup_expired_resources,
).in_set(SimulationSet::Cleanup).after(SimulationSet::Stats).after(SimulationSet::Reproduction).run_if(should_run_tick_systems))
```

---

## System Categories

### Planning Set (6 systems, parallel)
- `plan_rabbit_actions`
- `plan_deer_actions`
- `plan_raccoon_actions`
- `plan_bear_actions`
- `plan_fox_actions`
- `plan_wolf_actions`

### ActionExecution Set (1 system, sequential)
- `execute_queued_actions`

### Movement Set (2 systems, parallel)
- `tick_movement_system`
- `execute_movement_component`

### Stats Set (2 systems, parallel)
- `tick_stats_system`
- `auto_eat_system`

### Reproduction Set (14 systems, parallel)
- `update_age_and_wellfed_system`
- `tick_reproduction_timers_system`
- 6x `mate_matching_system` (one per species)
- 6x `birth_system` (one per species)

### Cleanup Set (2 systems, sequential)
- `death_system`
- `tick_carcasses`

---

## Rules for System Organization

1. **Planning systems**: Read-only queries, decision making → Planning set
2. **Action execution**: Needs World access, modifies multiple entities → ActionExecution set
3. **Movement**: Modifies position/movement state → Movement set
4. **Stats updates**: Modifies health/hunger/energy → Stats set
5. **Reproduction**: Mate matching, spawning → Reproduction set
6. **Cleanup**: Entity removal, resource cleanup → Cleanup set (must run last)

---

## Dependencies

```rust
// ActionExecution must run after Planning
.after(SimulationSet::Planning)

// Movement must run after ActionExecution
.after(SimulationSet::ActionExecution)

// Stats/Reproduction must run after Movement
.after(SimulationSet::Movement)

// Cleanup must run after both Stats and Reproduction
.after(SimulationSet::Stats)
.after(SimulationSet::Reproduction)
```

---

## Tests

**Location**: `tests/system_sets_test.rs`

**Run**: `cargo test --test system_sets_test`

**Coverage**:
- SimulationSet enum exists
- SystemSet trait implementation
- Execution ordering
- Parallel execution within set
- Run conditions with sets

---

## Files

**Definition**: `src/simulation/system_sets.rs`
**Export**: `src/simulation/mod.rs`
**Usage**: `src/entities/mod.rs`, `src/ai/mod.rs`
**Tests**: `tests/system_sets_test.rs`

---

## Performance

**TPS**: 10.0 (maintained, not exceeded)
**Parallelism**: 22/24 systems can run in parallel
**CPU Utilization**: Better multi-core distribution
**Thread Safety**: Enforced by Bevy's system sets

---

## Quick Commands

```bash
# Run all tests
cargo test

# Run system set tests only
cargo test --test system_sets_test

# Build release
cargo build --release

# Run simulation
cargo run --release --bin life-simulator
```

---

## Troubleshooting

**Issue**: Systems running in wrong order
**Fix**: Check `.after()` dependencies match execution requirements

**Issue**: System not running
**Fix**: Verify run condition `should_run_tick_systems` is attached

**Issue**: Race condition detected
**Fix**: Move system to later set or add explicit ordering

**Issue**: Performance regression
**Fix**: Check if system is in correct set (parallel vs sequential)

---

**Reference**: `PHASE6_SYSTEM_SETS_DELIVERY.md` for full implementation details
