# Generic Group Formation - Quick Reference

## TL;DR - 30 Second Overview

Generic group formation infrastructure is ready. Add one component to any species to enable group behavior:

```rust
// Wolf pack (3-8 members)
.insert(GroupFormationConfig::wolf_pack())

// Deer herd (5-20 members)
.insert(GroupFormationConfig::deer_herd())

// Rabbit warren (4-15 members)
.insert(GroupFormationConfig::rabbit_warren())
```

Systems automatically handle formation, cohesion, and dissolution. No species-specific code needed.

---

## Factory Methods

### Wolf Pack
```rust
GroupFormationConfig::wolf_pack()
```
- Group Type: Pack
- Min Size: 3 members
- Max Size: 8 members
- Formation Radius: 50 tiles
- Cohesion Radius: 150 tiles
- Check Interval: 300 ticks
- Reformation Cooldown: 600 ticks

### Deer Herd
```rust
GroupFormationConfig::deer_herd()
```
- Group Type: Herd
- Min Size: 5 members
- Max Size: 20 members
- Formation Radius: 100 tiles
- Cohesion Radius: 200 tiles
- Check Interval: 300 ticks
- Reformation Cooldown: 400 ticks

### Rabbit Warren
```rust
GroupFormationConfig::rabbit_warren()
```
- Group Type: Warren
- Min Size: 4 members
- Max Size: 15 members
- Formation Radius: 30 tiles
- Cohesion Radius: 100 tiles
- Check Interval: 200 ticks
- Reformation Cooldown: 300 ticks

---

## Group Types

```rust
pub enum GroupType {
    Pack,      // Wolves: Coordinated hunting
    Herd,      // Deer/Cattle: Safety in numbers
    Flock,     // Birds: Coordinated flight
    Warren,    // Rabbits: Shared burrow defense
    Colony,    // Ants/Bees: Shared nest
    School,    // Fish: Coordinated swimming
}
```

---

## Systems

### Formation System
```rust
generic_group_formation_system(
    commands: Commands,
    tick: Res<SimulationTick>,
    candidates: Query<(Entity, &TilePosition, &GroupFormationConfig),
        (Without<PackLeader>, Without<PackMember>)>,
    world: &World,
)
```
**What it does**: Finds clusters of nearby entities and forms groups

**When it runs**: Every `check_interval_ticks` (species-specific)

**Output**: Inserts `PackLeader` and `PackMember` components

### Cohesion System
```rust
generic_group_cohesion_system(
    commands: Commands,
    tick: Res<SimulationTick>,
    groups: Query<(Entity, &PackLeader, &TilePosition, &GroupFormationConfig)>,
    members: Query<(Entity, &PackMember, &TilePosition)>,
)
```
**What it does**: Checks member distances, removes distant members, dissolves small groups

**When it runs**: Every `check_interval_ticks` (species-specific)

**Output**: Removes members beyond `cohesion_radius`, dissolves groups below `min_group_size`

### Member Removal System
```rust
process_member_removals(
    commands: Commands,
    leaders: Query<&mut PackLeader>,
    markers: Query<(Entity, &RemoveMemberMarker)>,
)
```
**What it does**: Processes deferred member removals

**When it runs**: After cohesion system (same tick)

**Output**: Updates leader member lists, removes markers

### Coordination System
```rust
apply_group_behavior_bonuses(
    entity: Entity,
    actions: &mut Vec<UtilityScore>,
    world: &World,
)
```
**What it does**: Applies species-specific behavior bonuses based on group type

**When it runs**: During planning phase (per entity)

**Output**: Modifies action utilities based on group membership

---

## Custom Configuration

Create your own group configuration:

```rust
GroupFormationConfig {
    enabled: true,
    group_type: GroupType::Flock,
    min_group_size: 6,
    max_group_size: 30,
    formation_radius: 75.0,
    cohesion_radius: 120.0,
    check_interval_ticks: 250,
    reformation_cooldown_ticks: 500,
}
```

---

## Migration Checklist

### Migrating Wolves to Generic System

1. **Add config to wolf spawns**:
```rust
commands.spawn((
    Wolf,
    TilePosition::from_tile(pos),
    GroupFormationConfig::wolf_pack(), // ADD THIS
    // ... other components
));
```

2. **Register systems** (in TQUAIPlugin or EntitiesPlugin):
```rust
.add_systems(Update, (
    generic_group_formation_system,
    generic_group_cohesion_system,
    process_member_removals,
).chain())
```

3. **Test wolf pack formation** works identically

4. **Remove old systems**:
- Delete `wolf_pack_formation_system`
- Delete `wolf_pack_cohesion_system`
- Move bonuses to `behaviors/pack_hunting.rs`

### Adding New Species

1. **Create config** (or use existing):
```rust
let config = GroupFormationConfig::deer_herd();
```

2. **Add to spawns**:
```rust
commands.spawn((
    Deer,
    TilePosition::from_tile(pos),
    config, // ADD THIS
    // ... other components
));
```

3. **Implement behavior bonuses** (optional):
```rust
// In src/ai/behaviors/herd_grazing.rs
pub fn apply_herd_safety_bonus(
    entity: Entity,
    actions: &mut Vec<UtilityScore>,
    leader: Entity,
    members: Vec<Entity>,
    world: &World,
) {
    const HERD_SAFETY_BONUS: f32 = 0.10;

    for action in actions.iter_mut() {
        if matches!(action.action_type, ActionType::Graze | ActionType::Rest) {
            action.utility += HERD_SAFETY_BONUS;
        }
    }
}
```

4. **Wire up in coordination.rs**:
```rust
match group_type {
    GroupType::Herd => apply_herd_safety_bonus(entity, actions, leader, members, world),
    // ... other types
}
```

---

## Debugging

### Check if entity is in a group
```rust
// As leader
let is_leader = world.get::<PackLeader>(entity).is_some();

// As member
let is_member = world.get::<PackMember>(entity).is_some();

// Get group info
if let Some(leader_comp) = world.get::<PackLeader>(entity) {
    println!("Leader with {} members, type: {:?}",
        leader_comp.members.len(),
        leader_comp.group_type);
}
```

### Common Issues

**Groups not forming?**
- Check `enabled` is true
- Verify tick divisible by `check_interval_ticks`
- Ensure entities within `formation_radius`
- Check entity count >= `min_group_size`

**Groups dissolving unexpectedly?**
- Check members within `cohesion_radius`
- Verify group size >= `min_group_size - 1` (leader excluded)
- Check for despawned members

**Behavior bonuses not applying?**
- Verify group type matches in `apply_group_behavior_bonuses`
- Check behavior function is called
- Ensure action types match bonus conditions

---

## File Locations

### Configuration
- `src/entities/group_config.rs` - GroupFormationConfig component
- `src/entities/pack_relationships.rs` - GroupType enum, PackLeader/PackMember

### Systems
- `src/ai/group_formation.rs` - Formation system
- `src/ai/group_cohesion.rs` - Cohesion system
- `src/ai/group_coordination.rs` - Behavior coordination

### Behaviors (Future)
- `src/ai/behaviors/pack_hunting.rs` - Wolf pack bonuses
- `src/ai/behaviors/herd_grazing.rs` - Deer herd bonuses
- `src/ai/behaviors/warren_defense.rs` - Rabbit warren bonuses

### Tests
- Unit tests in each module file
- Integration tests in `tests/generic_group_formation_integration.rs`

---

## Performance Notes

- **Spatial clustering**: O(n²) worst case, optimize with spatial grid if needed
- **Check intervals**: Reduce frequency for performance (higher `check_interval_ticks`)
- **Formation radius**: Smaller radius = faster clustering
- **Group size limits**: Larger `max_group_size` = more iterations

---

## Quick Commands

### Run all tests
```bash
cargo test --lib 'group' --test generic_group_formation_integration
```

### Run specific module tests
```bash
cargo test --lib group_config
cargo test --lib group_formation
cargo test --lib group_cohesion
cargo test --lib group_coordination
```

### Run integration tests
```bash
cargo test --test generic_group_formation_integration
```

---

**Questions? See GENERIC_GROUP_INFRASTRUCTURE_DELIVERY.md for full implementation details.**
