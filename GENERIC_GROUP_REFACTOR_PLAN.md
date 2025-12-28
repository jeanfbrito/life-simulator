# Generic Group System Refactoring Plan

## Problem
Current implementation is wolf-specific when it should be generic:
- `wolf_pack_formation.rs` - Can't be reused for deer herds, rabbit warrens, bird flocks
- Hardcoded wolf queries instead of generic species markers
- Violates DRY and modularity principles

## Solution: Generic + Configurable + Modular

### 1. Rename Components (More Generic)

```rust
// BEFORE (wolf-specific naming)
PackLeader → Generic enough, keep it
PackMember → Generic enough, keep it

// OR even better:
GroupLeader { group_type: GroupType, ... }
GroupMember { group_type: GroupType, ... }

enum GroupType {
    Pack,      // Wolves
    Herd,      // Deer, cattle
    Warren,    // Rabbits
    Flock,     // Birds
    Colony,    // Ants, bees
    School,    // Fish
}
```

### 2. Species Group Config (Component)

```rust
/// Configuration for group formation behavior per species
#[derive(Component, Clone)]
pub struct GroupFormationConfig {
    pub enabled: bool,
    pub group_type: GroupType,
    pub min_group_size: usize,        // 3 for wolves, 5 for deer herds
    pub formation_radius: f32,         // 50.0 for wolves, 100.0 for deer
    pub cohesion_radius: f32,          // 150.0 for wolves, 200.0 for herds
    pub check_interval_ticks: u64,     // How often to check formation
}

// Add to species at spawn time:
commands.spawn((
    Wolf,
    GroupFormationConfig {
        enabled: true,
        group_type: GroupType::Pack,
        min_group_size: 3,
        formation_radius: 50.0,
        cohesion_radius: 150.0,
        check_interval_ticks: 300,
    },
    // ... other wolf components
));
```

### 3. Generic Group Formation System

```rust
// BEFORE: wolf_pack_formation_system (Wolf-specific)
pub fn wolf_pack_formation_system(
    lone_wolves: Query<(Entity, &TilePosition), (With<Wolf>, Without<PackLeader>, Without<PackMember>)>,
    // ^^ HARDCODED WOLF
)

// AFTER: generic_group_formation_system (Any species)
pub fn generic_group_formation_system(
    mut commands: Commands,
    tick: Res<SimulationTick>,
    // Generic query: any entity with GroupFormationConfig
    candidates: Query<(Entity, &TilePosition, &GroupFormationConfig), Without<GroupLeader>>,
    world: &World,
) {
    // Group candidates by species/config
    // Form groups based on their config parameters
    // Works for wolves, deer, rabbits, etc.
}
```

### 4. Generic Group Tactics (Behavior Traits)

```rust
/// Trait for species-specific group behaviors
pub trait GroupBehavior {
    /// Bonus to apply for group actions
    fn group_action_bonus(&self, action_type: &ActionType, group_size: usize) -> f32;

    /// Should this species coordinate this action type?
    fn coordinates_action(&self, action_type: &ActionType) -> bool;
}

// Wolf implementation
impl GroupBehavior for Wolf {
    fn group_action_bonus(&self, action_type: &ActionType, group_size: usize) -> f32 {
        match action_type {
            ActionType::Hunt => 0.15 * (group_size as f32 - 1.0).min(3.0), // Up to 3 pack members
            _ => 0.0,
        }
    }

    fn coordinates_action(&self, action_type: &ActionType) -> bool {
        matches!(action_type, ActionType::Hunt)
    }
}

// Deer implementation
impl GroupBehavior for Deer {
    fn group_action_bonus(&self, action_type: &ActionType, group_size: usize) -> f32 {
        match action_type {
            ActionType::Graze => 0.05 * group_size.min(8) as f32, // Herd safety
            ActionType::Rest => 0.10 * group_size.min(5) as f32,  // Shared lookout
            _ => 0.0,
        }
    }

    fn coordinates_action(&self, action_type: &ActionType) -> bool {
        matches!(action_type, ActionType::Graze | ActionType::Rest)
    }
}
```

### 5. Modular Group System Architecture

```
src/ai/
├── group_formation.rs          # Generic formation logic (any species)
├── group_cohesion.rs           # Generic cohesion maintenance
├── group_coordination.rs       # Generic action coordination
└── behaviors/
    ├── pack_hunting.rs         # Wolf-specific: pack hunting tactics
    ├── herd_grazing.rs         # Deer-specific: herd grazing bonuses
    └── warren_management.rs    # Rabbit-specific: warren defense
```

### 6. Configuration-Driven Species Setup

```rust
// src/entities/types/wolf.rs
impl Wolf {
    pub fn default_group_config() -> GroupFormationConfig {
        GroupFormationConfig {
            enabled: true,
            group_type: GroupType::Pack,
            min_group_size: 3,
            formation_radius: 50.0,
            cohesion_radius: 150.0,
            check_interval_ticks: 300,
        }
    }
}

// src/entities/types/deer.rs
impl Deer {
    pub fn default_group_config() -> GroupFormationConfig {
        GroupFormationConfig {
            enabled: true,
            group_type: GroupType::Herd,
            min_group_size: 5,        // Deer form larger herds
            formation_radius: 100.0,   // Wider formation radius
            cohesion_radius: 200.0,    // Herds stay together longer
            check_interval_ticks: 300,
        }
    }
}
```

## Benefits of Refactor

### 1. Reusability
- ✅ Same formation code for wolves, deer, rabbits, birds
- ✅ Add new species → just provide config
- ✅ No code duplication

### 2. Modularity
- ✅ Generic formation system
- ✅ Species-specific behaviors via traits/modules
- ✅ Easy to extend with new group types

### 3. Configurability
- ✅ Tune per species without code changes
- ✅ Data-driven design
- ✅ Easy balancing

### 4. Testability
- ✅ Test generic system once
- ✅ Test species configs separately
- ✅ Mock GroupFormationConfig for edge cases

## Migration Path

### Phase 1: Add Generic Infrastructure
1. Create `GroupFormationConfig` component
2. Create `generic_group_formation_system`
3. Add config to Wolf species

### Phase 2: Migrate Wolves
1. Replace `wolf_pack_formation_system` with generic version
2. Keep wolf-specific hunting tactics separate
3. Validate wolf pack behavior unchanged

### Phase 3: Extend to Other Species
1. Add `GroupFormationConfig` to Deer (herds)
2. Add `GroupFormationConfig` to Rabbit (warrens)
3. Implement species-specific group behaviors

### Phase 4: Cleanup
1. Delete `wolf_pack_formation.rs`
2. Rename `wolf_pack_tactics.rs` → `pack_hunting_tactics.rs`
3. Create `herd_grazing_tactics.rs` for deer
4. Update documentation

## Code Estimate

### Generic System (Replaces wolf-specific)
- `src/ai/group_formation.rs` (~200 lines) - Generic formation logic
- `src/ai/group_cohesion.rs` (~150 lines) - Generic cohesion maintenance
- `src/ai/group_coordination.rs` (~100 lines) - Generic action bonuses
- `src/entities/group_config.rs` (~80 lines) - Config component

### Species Behaviors (Modular)
- `src/ai/behaviors/pack_hunting.rs` (~120 lines) - Wolf tactics
- `src/ai/behaviors/herd_grazing.rs` (~100 lines) - Deer tactics
- `src/ai/behaviors/warren_defense.rs` (~80 lines) - Rabbit tactics

**Total: ~830 lines** (vs current 312 lines wolf-specific)
**But supports unlimited species!**

## Testing Strategy

```rust
#[test]
fn test_generic_formation_wolves() {
    let config = GroupFormationConfig { /* wolf config */ };
    // Test with wolf entities
}

#[test]
fn test_generic_formation_deer() {
    let config = GroupFormationConfig { /* deer config */ };
    // Same formation logic, different config
}

#[test]
fn test_pack_hunting_bonus() {
    // Wolf-specific behavior test
}

#[test]
fn test_herd_grazing_bonus() {
    // Deer-specific behavior test
}
```

## Recommendation

**DO THE REFACTOR** - The current wolf-specific approach is technical debt.

### Why?
1. **Future-proof**: Adding bird flocks, fish schools, ant colonies is trivial
2. **Maintainable**: One formation system instead of N species-specific ones
3. **Testable**: Test generic system once, configs separately
4. **Professional**: Proper separation of concerns (generic + species-specific)

### Effort Estimate
- **Time**: ~4-6 hours
- **Risk**: Low (well-understood refactor)
- **Tests**: High confidence with TDD approach

---

**Decision**: Refactor to generic group system now before adding more species.
