# Pack Dynamics Quick Reference

## Components

### PackLeader
```rust
use life_simulator::entities::PackLeader;

// Create new pack leader
let pack = PackLeader::new(tick);

// Manage members
pack.add_member(member_entity);
pack.remove_member(member_entity);
pack.member_count();
pack.has_member(member_entity);
```

### PackMember
```rust
use life_simulator::entities::PackMember;

// Create new pack member
let member = PackMember::new(leader_entity, tick);

// Members track their leader
let leader = member.leader;
let joined_at = member.joined_tick;
```

## System Functions

### Pack Management
```rust
use life_simulator::ai::{
    establish_pack_leadership, add_to_pack, remove_from_pack, dissolve_pack
};

// Form new pack
establish_pack_leadership(leader_entity, current_tick, &mut commands);

// Add member to pack
add_to_pack(member, leader, current_tick, &mut commands, &world);

// Remove member from pack
remove_from_pack(member, &mut commands, &world);

// Dissolve entire pack
dissolve_pack(leader, &mut commands, &world);
```

### Pack Queries
```rust
use life_simulator::ai::{
    get_pack_members, get_pack_leader, get_pack_size,
    is_pack_leader, is_pack_member, is_in_pack, are_in_same_pack
};

// Get pack information
let members = get_pack_members(leader, &world);
let leader = get_pack_leader(member, &world);
let size = get_pack_size(leader, &world);  // leader + members

// Check pack status
is_pack_leader(wolf, &world);      // Is this wolf a pack leader?
is_pack_member(wolf, &world);      // Is this wolf in a pack?
is_in_pack(wolf, &world);          // Is this wolf in any pack?
are_in_same_pack(wolf1, wolf2, &world);  // Are they packmates?
```

## Typical Usage Pattern

### Wolf Planning Integration

```rust
// In plan_wolf_actions
use life_simulator::ai::{
    establish_pack_leadership, add_to_pack, get_pack_members,
    are_in_same_pack
};

// Check for nearby packmates
let nearby_wolves = find_nearby_wolves(my_wolf, radius);

// Form or join pack
if nearby_wolves.len() >= 2 && !is_pack_member(my_wolf, world) {
    establish_pack_leadership(my_wolf, tick, &mut commands);
    for wolf in nearby_wolves {
        add_to_pack(wolf, my_wolf, tick, &mut commands, &world);
    }
}

// Coordinate with pack
if let Some(leader) = get_pack_leader(my_wolf, &world) {
    let packmates = get_pack_members(leader, &world);
    // Use packmates for coordinated hunting

    // Hunt together
    for packmate in packmates {
        if let Some(prey) = find_nearby_prey() {
            // Coordinate attack
        }
    }
}
```

### Cleanup Automatic

```rust
// No manual cleanup needed - stale relationships cleaned automatically
// cleanup_stale_pack_relationships system runs in SimulationSet::Cleanup
```

## Important Notes

1. **Member Uniqueness**: PackLeader won't add the same member twice
2. **Empty Pack Cleanup**: Packs with no members are automatically removed
3. **Bidirectional Updates**: Always update both leader and members when managing relationships
4. **Timing**: Use current simulation tick when forming/joining packs for duration tracking
5. **Cleanup Phase**: Cleanup runs after entity despawn, so check packs before entities die

## Pack Size Considerations

- **Solo wolf**: No pack components
- **Pair hunting**: 2 wolves (leader + 1 member)
- **Small pack**: 3-4 wolves (typical)
- **Large pack**: 5-7 wolves (uncommon, consider split)

## Performance Tips

1. Query packs early in planning phase
2. Cache pack membership during single tick
3. Use `are_in_same_pack` instead of comparing leaders
4. Pack cleanup runs every 100 ticks, so stale relationships removed periodically
5. Dissolve packs when separated to reduce query overhead

## Testing

```bash
# Run pack relationship tests
cargo test --lib pack

# Run all pack tests
cargo test pack_

# Integration tests (requires parent_child_relationship_system fixes)
cargo test --test pack_dynamics_integration_test
```

## See Also

- `PACK_DYNAMICS_DELIVERY.md` - Full implementation details
- `src/entities/pack_relationships.rs` - Component definitions
- `src/ai/pack_relationship_system.rs` - System functions
- `tests/pack_dynamics_integration_test.rs` - Integration examples
- `src/ai/hunting_relationship_system.rs` - Similar pattern for hunting

## Example: Complete Pack Scenario

```rust
// Step 1: Wolf meets other wolves
let wolf_a = /* ... */;
let wolf_b = /* ... */;
let wolf_c = /* ... */;

// Step 2: Form pack (wolf_a is leader)
establish_pack_leadership(wolf_a, tick, &mut commands);
add_to_pack(wolf_b, wolf_a, tick, &mut commands, &world);
add_to_pack(wolf_c, wolf_a, tick, &mut commands, &world);

// Step 3: Check pack status
let pack_size = get_pack_size(wolf_a, &world);  // Returns 3
let is_leader = is_pack_leader(wolf_a, &world); // Returns true
let members = get_pack_members(wolf_a, &world); // Returns [wolf_b, wolf_c]

// Step 4: Coordinate hunting
if are_in_same_pack(wolf_b, wolf_c, &world) {
    // Execute coordinated hunt
}

// Step 5: Wolf leaves pack
remove_from_pack(wolf_b, &mut commands, &world);

// Step 6: Remaining pack
let pack_size = get_pack_size(wolf_a, &world);  // Returns 2

// Step 7: Dissolve pack
dissolve_pack(wolf_a, &mut commands, &world);
```

---

**Last Updated**: 2025-12-27
**Version**: 1.0 (Stable)
