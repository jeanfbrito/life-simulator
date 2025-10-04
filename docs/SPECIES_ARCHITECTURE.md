# Species Architecture Guide

This document describes the modular species architecture implemented in the life simulator, which allows new species to be added with minimal engine changes.

## Overview

The species architecture replaces hard-coded entity definitions with a modular, registry-based system that centralizes:

- Species metadata and spawn configuration
- Behavior configuration and AI parameters
- System registration for species-specific logic
- Viewer rendering metadata
- Demo bootstrap configuration

## Core Components

### 1. Species Registry (`src/entities/registry.rs`)

The `SpeciesRegistry` provides a centralized repository of species descriptors containing all metadata needed for spawning and rendering.

```rust
pub static SPECIES_REGISTRY: SpeciesRegistry = SpeciesRegistry::new();

pub struct SpeciesDescriptor {
    pub species: &'static str,
    pub name_prefix: &'static str,
    pub emoji: &'static str,
    pub spawn_fn: SpawnFunction,
    pub movement_speed: u32,
    pub wander_radius: i32,
    pub viewer_scale: f32,
    pub viewer_color: &'static str,
    // ... additional metadata
}
```

**Key Benefits:**
- Single source of truth for species data
- Automatic viewer integration via metadata API
- Simplified spawning with consistent behavior
- Easy addition of new species without code changes

**Usage Example:**
```rust
// Spawn a rabbit using the registry
let rabbit = spawn_using_registry(&mut commands, "Rabbit", "Bugs", IVec2::new(5, 10));

// Access species metadata
let deer_descriptor = SPECIES_REGISTRY.deer();
println!("Deer emoji: {}", deer_descriptor.emoji);
```

### 2. Species Systems Registry (`src/entities/systems_registry.rs`)

The `SpeciesSystemsRegistry` provides information about which systems each species has, without attempting complex dynamic system registration.

```rust
pub static SPECIES_SYSTEMS_REGISTRY: SpeciesSystemsRegistry = SpeciesSystemsRegistry::new();

pub struct SpeciesSystemsDescriptor {
    pub species: &'static str,
    pub has_mate_matching: bool,
    pub has_birth_system: bool,
    pub has_planner_system: bool,
}
```

**Key Benefits:**
- Centralized system information
- Queryable system capabilities
- No complex dynamic registration (Bevy limitation)
- Future-proof for system automation

**Usage Example:**
```rust
// Check if species has reproduction systems
if SPECIES_SYSTEMS_REGISTRY.species_has_birth_system("Rabbit") {
    // Rabbit can reproduce
}
```

### 3. Dynamic Viewer Metadata (`/api/species` endpoint)

The web server now serves species metadata dynamically, eliminating hard-coded viewer configuration.

**API Response:**
```json
{
  "species": {
    "Rabbit": {
      "emoji": "🐇",
      "viewer_scale": 0.5,
      "viewer_color": "#8b4513",
      "movement_speed": 20
    }
  },
  "juvenile_scales": {
    "Rabbit": 0.7
  },
  "default_entity": {
    "emoji": "❓",
    "sizeMultiplier": 1.0
  }
}
```

**JavaScript Integration:**
```javascript
// Load species configuration dynamically
const config = await fetch('/api/species').then(r => r.json());
ENTITY_CONFIG = config.species;
JUVENILE_SCALES = config.juvenile_scales;
```

### 4. Spawn Configuration System (`src/entities/spawn_config.rs`)

The `SpawnConfig` system replaces hard-coded demo spawns with data-driven configuration loaded from RON files.

**Configuration Structure:**
```rust
pub struct SpawnConfig {
    pub spawn_groups: Vec<SpawnGroup>,
    pub settings: SpawnSettings,
}

pub struct SpawnGroup {
    pub species: String,
    pub count: usize,
    pub names: Vec<String>,
    pub spawn_area: SpawnArea,
    pub sex_sequence: Option<Vec<SpawnSex>>,
    pub messages: Option<SpawnMessages>,
}

pub enum SpawnSex {
    Male,
    Female,
}

pub struct SpawnSettings {
    pub verbose_logging: bool,
    pub enable_spawning: bool,
    pub completion_message: String,
    pub post_spawn_messages: Vec<String>,
}
```

**Default Configuration (`config/spawn_config.ron`):**
```ron
(
    spawn_groups: [
        (
            species: "Rabbit",
            count: 5,
            names: ["Bugs", "Roger", "Thumper", "Peter", "Clover"],
            spawn_area: (
                center: (0, 0),
                search_radius: 15,
                max_attempts: 30,
            ),
            sex_sequence: None,
            messages: Some((
                start_message: "🎯 LIFE_SIMULATOR: Spawning 5 rabbits for testing...",
                success_template: "   ✅ Spawned rabbit #{index}: {name} 🐇 at {pos}",
                completion_message: "✅ LIFE_SIMULATOR: Spawned {count} rabbits successfully!",
            )),
        ),
        (
            species: "Deer",
            count: 2,
            names: ["Stag", "Doe"],
            spawn_area: (
                center: (0, 0),
                search_radius: 5,
                max_attempts: 50,
            ),
            sex_sequence: Some([Male, Female]),
            messages: Some((
                start_message: "🦌 Spawning deer pair near origin...",
                success_template: "   🦌 Spawned deer {name} ({sex}) at {pos}",
                completion_message: "🦌 Deer pair ready for testing ({count} spawned)",
            )),
        ),
    ],
    settings: (
        verbose_logging: true,
        enable_spawning: true,
        completion_message: "🌐 LIFE_SIMULATOR: View at http://127.0.0.1:54321/viewer.html\n🌐 LIFE_SIMULATOR: Entity API at http://127.0.0.1:54321/api/entities",
        post_spawn_messages: [
            "📊 Rabbits will only move when thirsty/hungry (no wandering)",
            "🧠 Behavior: Drinks at 15% thirst, grazes at 3-8 tile range",
            "🦌 Example: Deer follow their mothers while idle",
        ],
    ),
)
```

## Adding New Species

### Step 1: Create Species Behavior Module

Create `src/entities/types/newspecies.rs`:

```rust
pub struct NewSpeciesBehavior;

impl NewSpeciesBehavior {
    pub fn config() -> BehaviorConfig {
        BehaviorConfig::new(
            0.2,        // thirst_threshold
            0.3,        // hunger_threshold
            (5, 15),    // graze_range
            150,        // water_search_radius
            150,        // food_search_radius
            40,         // wander_radius
        )
    }

    pub fn stats_bundle() -> EntityStatsBundle {
        // Species-specific stats
    }

    pub fn reproduction_config() -> ReproductionConfig {
        // Species-specific reproduction parameters
    }
}
```

### Step 2: Update Species Registry

Add your species to `SPECIES_REGISTRY` in `src/entities/registry.rs`:

```rust
SpeciesDescriptor {
    species: "NewSpecies",
    name_prefix: "New",
    emoji: "🦊",
    spawn_fn: spawn_newspecies_registry,
    movement_speed: 12,
    wander_radius: 35,
    viewer_scale: 0.8,
    viewer_color: "#ff6b35",
    is_juvenile: false,
    juvenile_name_prefix: Some("Kit"),
    viewer_order: 70,
},
```

### Step 3: Update Systems Registry

Add to `SPECIES_SYSTEMS_REGISTRY`:

```rust
SpeciesSystemsDescriptor::new("NewSpecies")
    .with_mate_matching()
    .with_birth_system()
    .with_planner_system(),
```

### Step 4: Add Spawn Functions

Create registry spawn function and individual spawn helper:

```rust
fn spawn_newspecies_registry(commands: &mut Commands, name: String, position: IVec2) -> Entity {
    // Use NewSpeciesBehavior presets
}

pub fn spawn_newspecies(commands: &mut Commands, name: impl Into<String>, position: IVec2) -> Entity {
    spawn_using_registry(commands, "NewSpecies", name.into(), position)
}
```

### Step 5: Add AI Systems

Create planner and reproduction systems following existing patterns:

```rust
pub fn plan_newspecies_actions(...) { /* AI logic */ }
pub fn newspecies_mate_matching_system(...) { /* reproduction logic */ }
pub fn newspecies_birth_system(...) { /* birth logic */ }
```

### Step 6: Update Spawn Configuration

Add to default spawn configuration or create custom `config/spawn_config.ron`:

```ron
(
    species: "NewSpecies",
    count: 3,
    names: ["Fox", "Coyote", "Wolf"],
    spawn_area: (
        center: (10, 10),
        search_radius: 20,
        max_attempts: 40,
    ),
)
```

## Behavior System Integration

### Follow Behavior Generalization

The follow behavior now works with any candidate entities:

```rust
// Generic follow evaluation
pub fn evaluate_follow_behavior(
    entity: Entity,
    position: &TilePosition,
    candidates: &[(Entity, IVec2)],  // Generic candidates
    stop_distance: i32,
    max_follow_distance: i32,
) -> Option<UtilityScore>
```

**Usage in Species Planner:**
```rust
// Find mothers to follow
let mothers: Vec<(Entity, IVec2)> = mother_query.iter()
    .filter(|(_, mother_pos)| distance < max_distance)
    .map(|(entity, pos)| (entity, pos.tile))
    .collect();

// Add follow behavior if mothers are nearby
if let Some(follow_score) = evaluate_follow_behavior(
    entity, position, &mothers, stop_distance, max_distance
) {
    actions.push(follow_score);
}
```

## Testing and Validation

### Integration Testing

Add tests to verify species functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newspecies_spawning() {
        // Test that new species spawn correctly
    }

    #[test]
    fn test_newspecies_ai() {
        // Test AI behavior
    }

    #[test]
    fn test_viewer_integration() {
        // Test viewer metadata loading
    }
}
```

### Regression Testing

Run existing test suite to ensure no breaking changes:

```bash
cargo test
cargo run --bin life-simulator  # Manual verification
```

## Migration Guide

### From Hard-coded to Registry-based

1. **Replace Direct Spawns:**
   ```rust
   // Old
   let rabbit = commands.spawn((
       Rabbit,
       // ... manual component assembly
   )).id();

   // New
   let rabbit = spawn_using_registry(&mut commands, "Rabbit", "Bugs", pos);
   ```

2. **Replace Hard-coded Viewer Config:**
   ```javascript
   // Old
   const ENTITY_CONFIG = {
       'Rabbit': { emoji: '🐇', sizeMultiplier: 0.5 }
   };

   // New
   await loadSpeciesConfig(); // Loads from /api/species
   ```

3. **Replace Demo Spawns:**
   ```rust
   // Old: Hard-coded spawn_wanderers function
   // New: Configuration-driven spawn_entities_from_config system
   ```

## Performance Considerations

- **Registry Lookup**: O(1) array access, negligible impact
- **Dynamic Config**: Loaded once at startup, cached in memory
- **API Overhead**: Species metadata cached in JavaScript, no repeated requests
- **RON Parsing**: Done once at startup, minimal overhead

## Future Extensions

The architecture supports future enhancements:

- **Dynamic Species Loading**: Load species from external files
- **Species Inheritance**: Share behavior between related species
- **Ecosystem Interactions**: Predator-prey relationships
- **Species Evolution**: Mutate species traits over time
- **Mod Support**: Load species from external modules

## Troubleshooting

### Common Issues

1. **Missing BehaviorConfig**: Ensure spawn functions attach all required components
2. **AI Not Responding**: Check that species has planner system registered
3. **Viewer Issues**: Verify `/api/species` endpoint returns correct data
4. **Spawn Failures**: Check pathfinding grid and spawn area configuration

### Debug Commands

```bash
# Check species registry
RUST_LOG=debug cargo run --bin life-simulator

# Test API endpoints
curl http://127.0.0.1:54321/api/species

# Validate configuration
cargo run --bin life-simulator -- --validate-config
```

This architecture provides a solid foundation for scalable species management while maintaining the simulation's performance and modularity.
