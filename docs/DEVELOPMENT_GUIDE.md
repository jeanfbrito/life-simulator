# Development Guide

Complete guide for developing and extending the Life Simulator.

## Development Environment Setup

### Prerequisites
- Rust 1.70+ with Cargo
- Git for version control
- Web browser for viewer testing

### Project Structure
```
life-simulator/
├── src/                    # Core simulation code
│   ├── entities/          # Entity types and behaviors
│   ├── ai/               # AI planning and decision systems
│   ├── vegetation/       # Plant system and ResourceGrid
│   ├── world/            # World generation and terrain
│   └── web_server_simple.rs # HTTP API server
├── config/               # Configuration files
├── maps/                 # Generated world files
├── web-viewer/           # HTML/JS viewer application
├── godot-viewer/         # Godot engine viewer (alternative)
├── tools/                # Extraction and utility tools
├── tests/                # Integration and benchmark tests
└── docs/                 # Documentation
```

## Build & Run Commands

### Development
```bash
# Check compilation
cargo check

# Run with debug logging
RUST_LOG=info cargo run --bin life-simulator

# Run specific binary
cargo run --bin map_generator
cargo run --bin life_simulator
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test module
cargo test --test predator_fear_test
cargo test --test resource_grid_benchmark

# Run with output
cargo test -- --nocapture

# Benchmark specific systems
cargo test --test vegetation_performance -- --bench
```

### Release Builds
```bash
# Optimized release build
cargo build --release

# Run release version
cargo run --release --bin life-simulator

# Performance profiling
cargo flamegraph --bin life-simulator
```

## Core Architecture

### Entity Component System (ECS)
Built on Bevy 0.16 ECS:
- **Entities**: Individual animals and objects
- **Components**: Data (Position, Health, Species, etc.)
- **Systems**: Logic that processes components

Key components:
```rust
// Position and movement
#[derive(Component)]
struct Position { x: f32, y: f32 }

#[derive(Component)] 
struct Velocity { x: f32, y: f32 }

// Entity state
#[derive(Component)]
struct Health {
    current: u32,
    max: u32,
}

#[derive(Component)]
struct Species(String);

// AI and behavior
#[derive(Component)]
struct AIState {
    current_action: Action,
    utility_scores: HashMap<String, f32>,
}
```

### ResourceGrid Vegetation System
High-performance sparse storage for vegetation data:

```rust
// Core vegetation storage
pub struct ResourceGrid {
    chunks: HashMap<Vector2i, VegetationChunk>,
    chunk_size: u32,
    lod_manager: ChunkLODManager,
}

impl ResourceGrid {
    pub fn get_biomass_at(&self, x: i32, y: i32) -> f32 {
        let chunk_coord = self.world_to_chunk(x, y);
        let chunk = self.chunks.get(&chunk_coord)?;
        chunk.get_biomass(x % self.chunk_size, y % self.chunk_size)
    }
    
    pub fn consume_grass(&mut self, x: i32, y: i32, amount: f32) {
        // Update vegetation with grazing
        self.update_biomass(x, y, |biomass| (biomass - amount).max(0.0));
    }
}
```

### AI Planning System
Event-driven AI with utility-based decision making:

```rust
// AI planner with fear responses
pub struct AIPlanner {
    available_actions: Vec<Box<dyn Action>>,
    utility_threshold: f32,
    fear_modifier: f32,
}

impl AIPlanner {
    pub fn plan_action(&self, entity: &Entity, world: &World) -> Option<Action> {
        let mut best_action = None;
        let mut best_utility = self.utility_threshold;
        
        // Evaluate each action
        for action in &self.available_actions {
            let utility = action.calculate_utility(entity, world);
            
            // Apply fear modifier if predators nearby
            let fear_modifier = self.calculate_fear_modifier(entity, world);
            let adjusted_utility = utility * fear_modifier;
            
            if adjusted_utility > best_utility {
                best_utility = adjusted_utility;
                best_action = Some(action.clone());
            }
        }
        
        best_action
    }
}
```

## Adding New Species

### 1. Create Species Module
Create `src/entities/types/my_species.rs`:

```rust
use bevy::prelude::*;
use crate::entities::{EntityBehavior, SpawnHandler};

pub struct MySpeciesPlugin;

impl Plugin for MySpeciesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_my_species)
           .add_systems(Update, (
               my_species_ai,
               my_species_reproduction,
               my_species_movement,
           ));
    }
}

#[derive(Component)]
struct MySpeciesBehavior {
    // Species-specific behavior data
    foraging_radius: f32,
    social_tendency: f32,
    reproduction_cooldown: Timer,
}

fn setup_my_species(mut commands: Commands) {
    // Initialize species systems
}

fn my_species_ai(
    mut query: Query<(Entity, &mut AIState, &MySpeciesBehavior)>,
    world: Res<World>,
) {
    // Species-specific AI logic
}

fn my_species_reproduction(
    mut commands: Commands,
    query: Query<(Entity, &MySpeciesBehavior)>,
    time: Res<Time>,
) {
    // Reproduction logic
}
```

### 2. Define Species Config
Add to `config/species_config.ron`:

```ron
MySpecies: (
    name: "My Species",
    emoji: "🦎",
    stats: (
        health: 100,
        speed: 2.5,
        vision_radius: 8.0,
        reproduction_rate: 0.8,
    ),
    behaviors: (
        foraging_radius: 5.0,
        social_tendency: 0.6,
        fear_threshold: 0.3,
    ),
    spawn: (
        min_count: 5,
        max_count: 15,
        preferred_terrain: ["grass", "forest"],
    ),
)
```

### 3. Register in Species Registry
Update `src/entities/registry.rs`:

```rust
use crate::entities::types::my_species::MySpeciesPlugin;

impl Plugin for SpeciesRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RabbitPlugin,
            DeerPlugin,
            WolfPlugin,
            MySpeciesPlugin, // Add new species
        ));
        
        // Register spawn handler
        register_spawn_handler!(MySpecies, spawn_my_species);
    }
}

fn spawn_my_species(
    commands: &mut Commands,
    position: Vec2,
    species_config: &SpeciesConfig,
) -> Entity {
    commands.spawn((
        Position::new(position.x, position.y),
        Species("my_species".to_string()),
        MySpeciesBehavior {
            foraging_radius: species_config.behaviors.foraging_radius,
            social_tendency: species_config.behaviors.social_tendency,
            reproduction_cooldown: Timer::from_seconds(30.0, TimerMode::Once),
        },
        // ... other components
    )).id()
}
```

## Extending the Plant System

### Adding New Resource Types
Extend `src/vegetation/resources.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResourceType {
    Grass,
    TreeOak,
    TreePine,
    BerryBush,    // New resource type
    Mushroom,     // New resource type
}

#[derive(Component)]
struct Vegetation {
    resource_type: ResourceType,
    biomass: f32,
    growth_rate: f32,
    max_biomass: f32,
}

impl Vegetation {
    pub fn new(resource_type: ResourceType) -> Self {
        let (growth_rate, max_biomass) = match resource_type {
            ResourceType::BerryBush => (0.8, 25.0),
            ResourceType::Mushroom => (0.3, 5.0),
            _ => (0.5, 20.0),
        };
        
        Self {
            resource_type,
            biomass: max_biomass * 0.8,
            growth_rate,
            max_biomass,
        }
    }
}
```

### New Plant Behaviors
Create `src/vegetation/behaviors.rs`:

```rust
pub fn berry_growth_system(
    mut query: Query<&mut Vegetation, With<BerryBush>>,
    time: Res<Time>,
) {
    for mut vegetation in &mut query {
        // Berries grow seasonally
        if is_berry_season(time.elapsed_seconds()) {
            vegetation.biomass = (vegetation.biomass + vegetation.growth_rate * time.delta_seconds())
                .min(vegetation.max_biomass);
        }
    }
}

pub fn mushroom_spread_system(
    mut commands: Commands,
    query: Query<(Entity, &Position, &Vegetation), With<Mushroom>>,
    grid: Res<ResourceGrid>,
) {
    for (entity, position, vegetation) in &query {
        if vegetation.biomass > vegetation.max_biomass * 0.9 {
            // Spread to nearby tiles
            let spread_positions = get_spread_positions(position, 2.0);
            for spread_pos in spread_positions {
                if grid.can_spawn_resource(spread_pos) {
                    spawn_mushroom(&mut commands, spread_pos);
                }
            }
        }
    }
}
```

## Testing Guidelines

### Unit Tests
Test individual components and systems:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_biomass_consumption() {
        let mut grid = ResourceGrid::new(16);
        grid.set_biomass(0, 0, 10.0);
        
        let consumed = grid.consume_grass(0, 0, 3.0);
        assert_eq!(consumed, 3.0);
        assert_eq!(grid.get_biomass(0, 0), 7.0);
    }
    
    #[test]
    fn test_ai_utility_calculation() {
        let entity = create_test_entity();
        let world = create_test_world();
        let planner = AIPlanner::new();
        
        let action = planner.plan_action(&entity, &world);
        assert!(action.is_some());
    }
}
```

### Integration Tests
Test system interactions:

```rust
// tests/predator_prey_integration.rs
use life_simulator::*;

#[test]
fn test_rabbit_fear_response() {
    let mut app = App::new();
    app.add_plugins((
        LifeSimulatorPlugin,
        TestWorldPlugin,
    ));
    
    // Spawn rabbit and wolf
    let rabbit = app.world_mut().spawn(test_rabbit()).id();
    let wolf = app.world_mut().spawn(test_wolf()).id();
    
    // Place wolf near rabbit
    place_entity_near(app.world_mut(), wolf, rabbit, 5.0);
    
    // Update systems
    app.update();
    
    // Check fear response
    let fear_level = app.world().entity(rabbit).get::<FearLevel>().unwrap();
    assert!(fear_level.current > 0.5);
}
```

### Performance Benchmarks
Measure system performance:

```rust
// benches/vegetation_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_resource_grid_operations(c: &mut Criterion) {
    let mut grid = ResourceGrid::new(16);
    
    c.bench_function("biomass_access", |b| {
        b.iter(|| {
            for x in 0..100 {
                for y in 0..100 {
                    black_box(grid.get_biomass_at(x, y));
                }
            }
        })
    });
}

criterion_group!(benches, benchmark_resource_grid_operations);
criterion_main!(benches);
```

## Debugging Tools

### Logging Configuration
```bash
# Debug specific systems
RUST_LOG=life_simulator::vegetation=debug cargo run

# Performance logging
RUST_LOG=life_simulator::performance=info cargo run

# AI decision logging
RUST_LOG=life_simulator::ai=trace cargo run
```

### Debug Commands
Add debug systems to `src/debug.rs`:

```rust
pub fn debug_entities_system(
    query: Query<(Entity, &Position, &Species)>,
) {
    println!("=== Entity Debug ===");
    for (entity, position, species) in &query {
        println!("{}: {} at ({:.1}, {:.1})", entity, species.0, position.x, position.y);
    }
}

pub fn debug_vegetation_system(
    grid: Res<ResourceGrid>,
) {
    println!("=== Vegetation Debug ===");
    println!("Total biomass: {:.0}", grid.get_total_biomass());
    println!("Chunk count: {}", grid.get_chunk_count());
}
```

## Performance Optimization

### Profiling
```bash
# CPU profiling
cargo install cargo-flamegraph
cargo flamegraph --bin life-simulator

# Memory profiling
valgrind --tool=massif cargo run --bin life-simulator
```

### Optimization Techniques
- **Spatial indexing**: Use quad-trees for entity queries
- **LOD systems**: Reduce update frequency for distant objects
- **Batching**: Group similar operations together
- **Caching**: Cache expensive calculations
- **Component splitting**: Separate frequently vs rarely accessed data

## Code Style Guidelines

### Rust Conventions
- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`
- Prefer `&str` over `String` for function parameters
- Use `Result<T, E>` for error handling
- Document public APIs with `///`

### Architecture Patterns
- **Plugin-based**: Use Bevy plugins for modular features
- **Event-driven**: Use events for system communication
- **Component composition**: Prefer composition over inheritance
- **System separation**: Keep systems focused on single responsibilities

## Contributing

1. **Fork** the repository
2. **Create feature branch**: `git checkout -b feature/new-feature`
3. **Make changes** following the code style guidelines
4. **Add tests** for new functionality
5. **Run tests**: `cargo test`
6. **Update documentation** as needed
7. **Submit pull request** with clear description

### Pull Request Template
```markdown
## Description
Brief description of changes

## Changes
- List of specific changes made

## Testing
- How changes were tested
- Test cases added

## Breaking Changes
- Any breaking changes and migration guide
```
