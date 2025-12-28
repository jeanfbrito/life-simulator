# Proper Generic Group Architecture - Final Design

## Design Principles

1. **Separation of Concerns**: Generic formation ≠ Species behavior
2. **Data-Driven**: Configuration, not code duplication
3. **Open/Closed**: Open for extension (new species), closed for modification (core system)
4. **Single Responsibility**: Each module does ONE thing well
5. **DRY**: Write once, configure per species

## Layer Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 4: Species Integration                                │
│ - Wolf spawns with GroupFormationConfig (Pack, 3, 50)       │
│ - Deer spawns with GroupFormationConfig (Herd, 5, 100)      │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 3: Species-Specific Behaviors (Modular Traits)        │
│ - PackHuntingBehavior: +15% hunt utility for wolves         │
│ - HerdGrazingBehavior: +10% safety for deer in herds        │
│ - WarrenDefenseBehavior: +20% flee for rabbits near warren  │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 2: Generic Group Systems                              │
│ - group_formation.rs: Form groups based on config           │
│ - group_cohesion.rs: Maintain/dissolve groups               │
│ - group_coordination.rs: Apply species behavior bonuses     │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: Core Components (ECS Building Blocks)              │
│ - GroupLeader: Vec<Entity>, formed_tick, group_type         │
│ - GroupMember: Entity leader, joined_tick, group_type       │
│ - GroupFormationConfig: Species configuration                │
│ - GroupType enum: Pack/Herd/Flock/Warren/Colony/School      │
└─────────────────────────────────────────────────────────────┘
```

## Component Design

### 1. GroupType Enum (Domain Model)

```rust
/// Types of groups that can form in the simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum GroupType {
    Pack,      // Wolves: Coordinated hunting
    Herd,      // Deer/Cattle: Safety in numbers, grazing
    Flock,     // Birds: Coordinated flight, foraging
    Warren,    // Rabbits: Shared burrow, defense
    Colony,    // Ants/Bees: Shared nest, resource gathering
    School,    // Fish: Coordinated swimming, predator avoidance
}

impl GroupType {
    pub fn name(&self) -> &str {
        match self {
            GroupType::Pack => "pack",
            GroupType::Herd => "herd",
            GroupType::Flock => "flock",
            GroupType::Warren => "warren",
            GroupType::Colony => "colony",
            GroupType::School => "school",
        }
    }
}
```

### 2. GroupFormationConfig (Species Configuration)

```rust
/// Configuration for how a species forms and maintains groups
#[derive(Component, Clone, Debug)]
pub struct GroupFormationConfig {
    /// Is group formation enabled for this species?
    pub enabled: bool,

    /// Type of group this species forms
    pub group_type: GroupType,

    /// Minimum entities required to form a group (3 for packs, 5 for herds)
    pub min_group_size: usize,

    /// Maximum entities in a single group (8 for packs, 20 for herds)
    pub max_group_size: usize,

    /// Radius to search for potential group members (tiles)
    pub formation_radius: f32,

    /// Maximum distance members can drift before group dissolves (tiles)
    pub cohesion_radius: f32,

    /// How often to check for formation opportunities (ticks)
    pub check_interval_ticks: u64,

    /// Minimum time before a dissolved group can reform (ticks)
    pub reformation_cooldown_ticks: u64,
}

impl GroupFormationConfig {
    /// Wolf pack configuration
    pub fn wolf_pack() -> Self {
        Self {
            enabled: true,
            group_type: GroupType::Pack,
            min_group_size: 3,
            max_group_size: 8,
            formation_radius: 50.0,
            cohesion_radius: 150.0,
            check_interval_ticks: 300,
            reformation_cooldown_ticks: 600,
        }
    }

    /// Deer herd configuration
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

    /// Rabbit warren configuration
    pub fn rabbit_warren() -> Self {
        Self {
            enabled: true,
            group_type: GroupType::Warren,
            min_group_size: 4,
            max_group_size: 15,
            formation_radius: 30.0,  // Tighter formation
            cohesion_radius: 100.0,
            check_interval_ticks: 200,
            reformation_cooldown_ticks: 300,
        }
    }
}
```

### 3. Enhanced GroupLeader (Add GroupType)

```rust
#[derive(Component, Debug, Clone)]
pub struct GroupLeader {
    pub members: Vec<Entity>,
    pub formed_tick: u64,
    pub group_type: GroupType,  // NEW: What type of group
}
```

### 4. Enhanced GroupMember (Add GroupType)

```rust
#[derive(Component, Debug, Clone, Copy)]
pub struct GroupMember {
    pub leader: Entity,
    pub joined_tick: u64,
    pub group_type: GroupType,  // NEW: What type of group
}
```

## System Design

### 1. Generic Group Formation System

```rust
// src/ai/group_formation.rs

/// Generic group formation system - works for ANY species with GroupFormationConfig
pub fn generic_group_formation_system(
    mut commands: Commands,
    tick: Res<SimulationTick>,
    // Any entity with config, position, not already in a group
    candidates: Query<
        (Entity, &TilePosition, &GroupFormationConfig),
        (Without<GroupLeader>, Without<GroupMember>)
    >,
    world: &World,
) {
    // Group candidates by species (same config = same species)
    let mut species_groups: HashMap<GroupType, Vec<(Entity, IVec2, &GroupFormationConfig)>> = HashMap::new();

    for (entity, pos, config) in candidates.iter() {
        if !config.enabled {
            continue;
        }

        // Only check at configured intervals
        if tick.0 % config.check_interval_ticks != 0 {
            continue;
        }

        species_groups
            .entry(config.group_type)
            .or_default()
            .push((entity, pos.tile, config));
    }

    // Form groups for each species type
    for (_group_type, entities) in species_groups {
        if entities.is_empty() {
            continue;
        }

        let config = entities[0].2; // All same species = same config

        // Find clusters of nearby entities
        let clusters = find_proximity_clusters(&entities, config.formation_radius);

        // Form groups from clusters that meet min size
        for cluster in clusters {
            if cluster.len() >= config.min_group_size {
                form_group_from_cluster(
                    &mut commands,
                    cluster,
                    config,
                    tick.0,
                    world,
                );
            }
        }
    }
}

fn find_proximity_clusters(
    entities: &[(Entity, IVec2, &GroupFormationConfig)],
    radius: f32,
) -> Vec<Vec<Entity>> {
    // Spatial clustering algorithm
    // Returns groups of entities within radius of each other
    // Uses simple greedy clustering for now
    // TODO: Could optimize with spatial grid

    let mut clusters = Vec::new();
    let mut assigned = HashSet::new();

    for (i, (entity1, pos1, config)) in entities.iter().enumerate() {
        if assigned.contains(&i) {
            continue;
        }

        let mut cluster = vec![*entity1];
        assigned.insert(i);

        for (j, (entity2, pos2, _)) in entities.iter().enumerate() {
            if i == j || assigned.contains(&j) {
                continue;
            }

            if distance(*pos1, *pos2) <= radius {
                cluster.push(*entity2);
                assigned.insert(j);
            }
        }

        if cluster.len() >= config.min_group_size {
            clusters.push(cluster);
        }
    }

    clusters
}

fn form_group_from_cluster(
    commands: &mut Commands,
    cluster: Vec<Entity>,
    config: &GroupFormationConfig,
    tick: u64,
    world: &World,
) {
    if cluster.is_empty() {
        return;
    }

    // First entity becomes leader
    let leader = cluster[0];
    let members = &cluster[1..];

    // Establish leadership
    commands.entity(leader).insert(GroupLeader {
        members: members.to_vec(),
        formed_tick: tick,
        group_type: config.group_type,
    });

    // Add members
    for &member in members {
        commands.entity(member).insert(GroupMember {
            leader,
            joined_tick: tick,
            group_type: config.group_type,
        });
    }
}
```

### 2. Generic Group Cohesion System

```rust
// src/ai/group_cohesion.rs

/// Maintains group cohesion - dissolves groups when members drift too far
pub fn generic_group_cohesion_system(
    mut commands: Commands,
    tick: Res<SimulationTick>,
    groups: Query<(Entity, &GroupLeader, &TilePosition, &GroupFormationConfig)>,
    members: Query<(Entity, &GroupMember, &TilePosition)>,
) {
    for (leader_entity, leader, leader_pos, config) in groups.iter() {
        // Only check at intervals
        if tick.0 % config.check_interval_ticks != 0 {
            continue;
        }

        // Check member distances
        let mut members_to_remove = Vec::new();

        for &member_entity in &leader.members {
            if let Ok((_, _, member_pos)) = members.get(member_entity) {
                let dist = distance(leader_pos.tile, member_pos.tile);

                if dist > config.cohesion_radius {
                    members_to_remove.push(member_entity);
                }
            } else {
                // Member doesn't exist anymore
                members_to_remove.push(member_entity);
            }
        }

        // Remove distant/dead members
        for member in members_to_remove {
            commands.entity(member).remove::<GroupMember>();
            commands.entity(leader_entity).insert(RemoveMemberMarker(member));
        }

        // If group too small after removals, dissolve it
        let remaining_size = leader.members.len() - members_to_remove.len();
        if remaining_size < config.min_group_size - 1 {
            // Dissolve group
            for &member in &leader.members {
                commands.entity(member).remove::<GroupMember>();
            }
            commands.entity(leader_entity).remove::<GroupLeader>();
        }
    }
}

// Helper marker for deferred member removal
#[derive(Component)]
struct RemoveMemberMarker(Entity);

/// System to process deferred member removals
fn process_member_removals(
    mut commands: Commands,
    mut leaders: Query<&mut GroupLeader>,
    markers: Query<(Entity, &RemoveMemberMarker)>,
) {
    for (leader_entity, marker) in markers.iter() {
        if let Ok(mut leader) = leaders.get_mut(leader_entity) {
            leader.members.retain(|&e| e != marker.0);
        }
        commands.entity(leader_entity).remove::<RemoveMemberMarker>();
    }
}
```

### 3. Species Behavior Integration

```rust
// src/ai/group_coordination.rs

/// Apply group-specific behavior bonuses to actions
pub fn apply_group_behavior_bonuses(
    entity: Entity,
    actions: &mut Vec<UtilityScore>,
    world: &World,
) {
    // Check if entity is in a group
    let group_info = get_group_info(entity, world);

    if let Some((group_type, leader, members)) = group_info {
        // Apply species-specific bonuses based on group type
        match group_type {
            GroupType::Pack => apply_pack_hunting_bonus(entity, actions, leader, members, world),
            GroupType::Herd => apply_herd_safety_bonus(entity, actions, leader, members, world),
            GroupType::Warren => apply_warren_defense_bonus(entity, actions, leader, members, world),
            GroupType::Flock => apply_flock_coordination_bonus(entity, actions, leader, members, world),
            _ => {} // Other types can be added
        }
    }
}

fn get_group_info(entity: Entity, world: &World) -> Option<(GroupType, Entity, Vec<Entity>)> {
    // Check if leader
    if let Some(leader) = world.get::<GroupLeader>(entity) {
        return Some((leader.group_type, entity, leader.members.clone()));
    }

    // Check if member
    if let Some(member) = world.get::<GroupMember>(entity) {
        if let Some(leader_comp) = world.get::<GroupLeader>(member.leader) {
            return Some((member.group_type, member.leader, leader_comp.members.clone()));
        }
    }

    None
}

// Species-specific bonus functions in separate modules
```

### 4. Species-Specific Behavior Modules

```rust
// src/ai/behaviors/pack_hunting.rs
pub fn apply_pack_hunting_bonus(
    entity: Entity,
    actions: &mut Vec<UtilityScore>,
    leader: Entity,
    members: Vec<Entity>,
    world: &World,
) {
    const PACK_HUNT_BONUS: f32 = 0.15;

    // Boost hunt actions for wolves in packs
    for action in actions.iter_mut() {
        if matches!(action.action_type, ActionType::Hunt) {
            action.utility += PACK_HUNT_BONUS;
        }
    }
}

// src/ai/behaviors/herd_grazing.rs
pub fn apply_herd_safety_bonus(
    entity: Entity,
    actions: &mut Vec<UtilityScore>,
    leader: Entity,
    members: Vec<Entity>,
    world: &World,
) {
    const HERD_SAFETY_BONUS: f32 = 0.10;

    // Boost rest/graze for deer in herds (safety in numbers)
    for action in actions.iter_mut() {
        if matches!(action.action_type, ActionType::Graze | ActionType::Rest) {
            action.utility += HERD_SAFETY_BONUS;
        }
    }
}

// src/ai/behaviors/warren_defense.rs
pub fn apply_warren_defense_bonus(
    entity: Entity,
    actions: &mut Vec<UtilityScore>,
    leader: Entity,
    members: Vec<Entity>,
    world: &World,
) {
    const WARREN_FLEE_BONUS: f32 = 0.20;

    // Boost flee for rabbits in warrens (group alert)
    for action in actions.iter_mut() {
        if matches!(action.action_type, ActionType::Flee) {
            action.utility += WARREN_FLEE_BONUS;
        }
    }
}
```

## File Structure

```
src/
├── entities/
│   ├── group_config.rs          # GroupFormationConfig component
│   └── pack_relationships.rs    # GroupLeader, GroupMember (rename from pack)
│
├── ai/
│   ├── group_formation.rs       # Generic formation system
│   ├── group_cohesion.rs        # Generic cohesion system
│   ├── group_coordination.rs    # Generic behavior application
│   │
│   └── behaviors/
│       ├── pack_hunting.rs      # Wolf pack hunting bonuses
│       ├── herd_grazing.rs      # Deer herd safety bonuses
│       └── warren_defense.rs    # Rabbit warren defense bonuses
│
└── entities/types/
    ├── wolf.rs                  # Spawns with GroupFormationConfig::wolf_pack()
    ├── deer.rs                  # Spawns with GroupFormationConfig::deer_herd()
    └── rabbit.rs                # Spawns with GroupFormationConfig::rabbit_warren()
```

## Migration Strategy

### Phase 1: Add Generic Infrastructure (No Breaking Changes)
1. Create `GroupType` enum
2. Create `GroupFormationConfig` component
3. Add `group_type` field to `GroupLeader`/`GroupMember`
4. Create generic formation/cohesion systems

### Phase 2: Migrate Wolves to Generic System
1. Add `GroupFormationConfig::wolf_pack()` to wolf spawns
2. Register generic systems alongside wolf-specific ones
3. Validate wolves still form packs correctly
4. Remove wolf-specific systems

### Phase 3: Delete Wolf-Specific Code
1. Delete `wolf_pack_formation.rs`
2. Delete `wolf_pack_tactics.rs`
3. Move wolf bonuses to `behaviors/pack_hunting.rs`

### Phase 4: Extend to Other Species
1. Add `GroupFormationConfig` to deer (herds)
2. Add `GroupFormationConfig` to rabbits (warrens)
3. Implement species behaviors in `behaviors/`

## Testing Strategy

### Generic System Tests
```rust
#[test]
fn test_generic_formation_with_wolf_config() {
    let config = GroupFormationConfig::wolf_pack();
    // Test formation works with wolf parameters
}

#[test]
fn test_generic_formation_with_deer_config() {
    let config = GroupFormationConfig::deer_herd();
    // Same formation code, different config
}

#[test]
fn test_cohesion_dissolves_distant_members() {
    // Test generic cohesion system
}
```

### Species Behavior Tests
```rust
#[test]
fn test_pack_hunting_bonus() {
    // Test wolf-specific pack hunting bonus
}

#[test]
fn test_herd_safety_bonus() {
    // Test deer-specific herd safety bonus
}
```

## Success Criteria

✅ Wolf packs work identically to before
✅ Deer can form herds without new formation code
✅ Rabbits can form warrens without new formation code
✅ Adding bird flocks = just config + behavior module
✅ All tests pass (existing + new)
✅ No code duplication
✅ Clean modular architecture

---

**This is the PROPER architecture for a professional, maintainable, extensible group system.**
