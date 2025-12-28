# Wolf Pack AI - Quick Reference Guide

## System Status: ✅ ACTIVE

Wolf pack AI is now fully integrated and operational in the simulation.

---

## How It Works

### Automatic Pack Formation
Wolves with `GroupFormationConfig::wolf_pack()` automatically:
1. **Detect nearby wolves** within 50 tile radius
2. **Form pack** when 3+ wolves are proximate
3. **Assign roles**: First wolf becomes leader, others become members
4. **Apply bonuses**: Hunt actions get +15% utility boost

### Pack Maintenance
Every 300 ticks (30 seconds), the system:
1. **Checks cohesion**: Are members within 150 tiles of leader?
2. **Removes stragglers**: Members beyond cohesion radius
3. **Dissolves packs**: If pack falls below 3 members

### Pack Hunting
When hunting in a pack:
- Base hunt utility: 0.4-0.85 (varies by hunger/distance)
- Pack coordination bonus: +0.15 utility
- Scales with nearby pack members (within 80 tiles)
- Coordinated attacks more likely than solo hunting

---

## Configuration (Wolf Packs)

```rust
GroupFormationConfig::wolf_pack()
    min_group_size: 3           // Minimum wolves to form pack
    max_group_size: 8           // Maximum wolves in one pack
    formation_radius: 50.0      // Detection range for pack formation
    cohesion_radius: 150.0      // Max distance before member removed
    check_interval_ticks: 300   // How often to check (30 seconds)
```

---

## System Architecture

### Registered Systems (TQUAIPlugin)
```rust
SimulationSet::Planning (runs on tick):
  ├─ generic_group_formation_system
  ├─ generic_group_cohesion_system
  └─ process_member_removals

SimulationSet::ActionExecution:
  └─ execute_queued_actions (with pack bonuses)

SimulationSet::Cleanup:
  └─ cleanup_stale_pack_relationships
```

### Key Files
| File | Purpose |
|------|---------|
| `src/ai/mod.rs` | System registration in TQUAIPlugin |
| `src/ai/group_formation.rs` | Pack formation logic |
| `src/ai/group_cohesion.rs` | Pack maintenance logic |
| `src/ai/behaviors/pack_hunting.rs` | Hunt utility bonuses |
| `src/ai/predator_toolkit.rs` | Wolf action evaluation with bonuses |
| `src/entities/pack_relationships.rs` | Pack data structures |
| `src/entities/group_config.rs` | Configuration presets |
| `src/entities/entity_types.rs` | Wolf spawning with config |

---

## Component Reference

### PackLeader
```rust
PackLeader {
    members: Vec<Entity>,           // Pack member entities
    formed_tick: u64,               // When pack was formed
    group_type: GroupType::Pack,    // Type of group
}
```

### PackMember
```rust
PackMember {
    leader: Entity,                 // Which leader to follow
    joined_tick: u64,               // When joined pack
    group_type: GroupType::Pack,    // Type of group
}
```

### GroupFormationConfig
```rust
GroupFormationConfig {
    enabled: true,                  // Is formation active?
    group_type: GroupType::Pack,    // What kind of group
    min_group_size: 3,              // Minimum entities
    max_group_size: 8,              // Maximum entities
    formation_radius: 50.0,         // Detection range
    cohesion_radius: 150.0,         // Maintenance range
    check_interval_ticks: 300,      // Update frequency
    reformation_cooldown_ticks: 600 // Cooldown after dissolve
}
```

---

## Test Coverage

### Integration Tests (5 tests)
```
✅ test_wolves_form_pack_when_proximate
✅ test_wolves_dont_form_pack_when_distant
✅ test_pack_dissolves_when_members_drift
✅ test_pack_cohesion_maintained
✅ test_pack_hunting_bonus_applied
```

### Unit Tests (31 tests total)
- Pack relationship tests: 10 tests
- Group formation tests: 7 tests
- Group cohesion tests: 7 tests
- Group coordination tests: 7 tests

**All tests passing**: ✅

---

## API Functions

### Pack Relationship Helpers
```rust
// Query functions (read-only)
is_in_pack(wolf: Entity, world: &World) -> bool
is_pack_leader(wolf: Entity, world: &World) -> bool
is_pack_member(wolf: Entity, world: &World) -> bool
get_pack_members(leader: Entity, world: &World) -> Vec<Entity>
get_pack_leader(member: Entity, world: &World) -> Option<Entity>
get_pack_size(leader: Entity, world: &World) -> usize
are_in_same_pack(wolf1: Entity, wolf2: Entity, world: &World) -> bool

// Mutation functions (use Commands)
establish_pack_leadership(leader: Entity, tick: u64, commands: &mut Commands)
add_to_pack(member: Entity, leader: Entity, tick: u64, commands: &mut Commands, world: &World)
remove_from_pack(member: Entity, commands: &mut Commands, world: &World)
dissolve_pack(leader: Entity, commands: &mut Commands, world: &World)
```

---

## Example Usage Patterns

### Checking Pack Status
```rust
if is_in_pack(wolf_entity, world) {
    if is_pack_leader(wolf_entity, world) {
        let members = get_pack_members(wolf_entity, world);
        println!("Leading pack of {} wolves", members.len() + 1);
    } else {
        let leader = get_pack_leader(wolf_entity, world).unwrap();
        println!("Following pack leader {:?}", leader);
    }
}
```

### Pack Coordination in Actions
```rust
// In evaluate_wolf_actions (predator_toolkit.rs)
let mut actions = evaluate_core_actions(...);

// Automatically applies pack bonuses to Hunt actions
apply_group_behavior_bonuses(entity, &mut actions, world);
```

---

## Behavior Examples

### Scenario 1: Pack Formation
```
Initial State:
  Wolf A at (100, 100)
  Wolf B at (110, 100)
  Wolf C at (120, 100)
  Wolf D at (110, 110)

Tick 300 (formation check):
  generic_group_formation_system detects 4 wolves within 50 tiles

Result:
  Wolf A becomes PackLeader { members: [B, C, D] }
  Wolf B becomes PackMember { leader: A }
  Wolf C becomes PackMember { leader: A }
  Wolf D becomes PackMember { leader: A }
```

### Scenario 2: Pack Hunting
```
Pack Status:
  Leader: Wolf A
  Members: Wolf B, Wolf C, Wolf D
  All within 80 tiles of each other

Hunt Action Evaluation:
  Base utility: 0.65 (moderate hunger, prey nearby)
  Pack coordination bonus: +0.15
  Final utility: 0.80 (much more likely to hunt)
```

### Scenario 3: Pack Dissolution
```
Initial Pack:
  Leader: Wolf A at (100, 100)
  Member: Wolf B at (110, 100)
  Member: Wolf C at (120, 100)

Wolf C moves to (300, 100) - beyond cohesion radius

Tick 600 (cohesion check):
  Wolf C distance from A: 200 tiles > 150 cohesion radius
  Wolf C removed from pack
  Pack now has only 2 members (below min_group_size of 3)

Result:
  Pack dissolved
  All wolves lose PackLeader/PackMember components
  Can reform if they come together again
```

---

## Performance Notes

- **Formation checks**: Every 300 ticks (30 seconds)
- **Cohesion checks**: Every 300 ticks (30 seconds)
- **Computational cost**: O(n²) for formation, O(m) for cohesion (m = pack members)
- **Optimized**: Only checks entities with GroupFormationConfig
- **Scalable**: Works for any number of packs simultaneously

---

## Extending Pack Behavior

### Adding New Group Types
```rust
// In GroupFormationConfig
pub fn deer_herd() -> Self {
    Self {
        enabled: true,
        group_type: GroupType::Herd,
        min_group_size: 5,
        max_group_size: 20,
        formation_radius: 100.0,
        cohesion_radius: 200.0,
        check_interval_ticks: 300,
        reformation_cooldown_ticks: 400,
    }
}

// In spawn_deer
.insert(GroupFormationConfig::deer_herd())
```

### Custom Behavior Bonuses
```rust
// In src/ai/behaviors/herd_grazing.rs
pub fn apply_herd_safety_bonus(
    entity: Entity,
    actions: &mut Vec<UtilityScore>,
    leader: Entity,
    members: Vec<Entity>,
    world: &World,
) {
    // Custom herd behavior logic
}

// In group_coordination.rs
GroupType::Herd => apply_herd_safety_bonus(entity, actions, leader, members, world),
```

---

## Debugging Tips

### Enable Pack Logging
```bash
RUST_LOG=life_simulator::ai::group_formation=debug cargo run
```

### Check Pack Status in Code
```rust
fn debug_pack_status(wolf: Entity, world: &World) {
    if let Some(leader) = world.get::<PackLeader>(wolf) {
        info!("Wolf {:?} is pack leader with {} members",
              wolf, leader.members.len());
    } else if let Some(member) = world.get::<PackMember>(wolf) {
        info!("Wolf {:?} is pack member following {:?}",
              wolf, member.leader);
    } else {
        info!("Wolf {:?} is lone wolf", wolf);
    }
}
```

---

## TDD Implementation Summary

| Metric | Value |
|--------|-------|
| Tests Written First | 5 integration tests |
| Implementation Files Modified | 2 files |
| Infrastructure Reused | 100% (all systems pre-built) |
| Build Status | ✅ Success |
| Test Results | ✅ 31/31 passing |
| Breaking Changes | 0 |

---

**Implementation Status**: ✅ **COMPLETE AND TESTED**

Wolf pack AI is production-ready and automatically active in all simulations.
