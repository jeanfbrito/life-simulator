# Deer Herd System - Quick Reference

## At a Glance

**Status**: ✅ Fully Operational
**Tests**: 6/6 passing
**Integration**: Generic group infrastructure

## Configuration

```rust
// In spawn_deer (already implemented)
.insert(GroupFormationConfig::deer_herd())
```

**Parameters**:
- Group Type: `Herd`
- Min Size: `5` deer
- Max Size: `20` deer
- Formation Radius: `100` tiles
- Cohesion Radius: `200` tiles
- Check Interval: `300` ticks (~30s)
- Reformation Cooldown: `400` ticks (~40s)

## Behavior Bonus

**Herd Safety Bonus**: +10% utility to Graze and Rest actions

```rust
// Applied in plan_deer_actions (already implemented)
apply_group_behavior_bonuses(entity, &mut actions, world);
```

## How It Works

### Formation (Every 300 Ticks)
1. Scan for deer within 100 tiles
2. If 5+ deer found → form herd
3. One deer = PackLeader, others = PackMember
4. Tag as GroupType::Herd

### Cohesion (Every 300 Ticks)
1. Check each member distance from leader
2. Remove members >200 tiles away
3. If <5 members remain → dissolve herd

### Grazing Bonus (Every Planning Cycle)
1. Check if deer is in herd (PackLeader or PackMember)
2. If yes → boost Graze/Rest utility by +10%
3. Result: Deer prefer grazing when in herds

## Common Scenarios

### ✅ Herd Forms
- 8 deer spawn near each other (<100 tiles)
- After 300 ticks → 1 leader + 7 members
- All receive +10% graze bonus

### ✅ Herd Maintains
- Members stay within 200 tiles
- Coordinated grazing behavior
- Safety in numbers

### ✅ Herd Dissolves
- 2 members wander >200 tiles away
- Only 3 remain (< min of 5)
- Herd components removed

### ✅ Multiple Herds
- 20 deer across map
- Form 2-4 separate herds
- Each operates independently

## Code Locations

**Spawning**: `src/entities/entity_types.rs::spawn_deer()`
**Planning**: `src/entities/types/deer.rs::plan_deer_actions()`
**Behavior**: `src/ai/behaviors/herd_grazing.rs::apply_herd_safety_bonus()`
**Tests**: `tests/deer_herd_integration.rs`

## Architecture

```
Generic Systems (Shared with wolves, rabbits)
├── generic_group_formation_system  (forms herds)
├── generic_group_cohesion_system   (maintains herds)
└── process_member_removals         (dissolves herds)

Deer-Specific Behavior
└── apply_herd_safety_bonus         (+10% graze utility)
```

## Testing

```bash
# Run all deer herd tests
cargo test --test deer_herd_integration

# Run specific test
cargo test --test deer_herd_integration test_deer_form_herd_when_proximate
```

## Debug Tips

**Check if deer is in herd**:
```rust
if world.get::<PackLeader>(deer_entity).is_some() {
    // This deer is a herd leader
}
if world.get::<PackMember>(deer_entity).is_some() {
    // This deer is a herd member
}
```

**Query all herds**:
```rust
for (entity, leader) in query.iter() {
    if leader.group_type == GroupType::Herd {
        println!("Herd at {:?} with {} members", entity, leader.members.len());
    }
}
```

## Performance

**Impact**: Minimal (same systems as wolf packs)
**Scalability**: Tested with 20+ deer forming multiple herds
**Optimization**: Uses spatial queries and cached tick checks

## Known Issues

None - all tests passing ✅

## Future Enhancements

- Herd leader following behavior
- Coordinated predator flee response
- Herd split/merge for large groups
- Visual herd indicators in viewer
