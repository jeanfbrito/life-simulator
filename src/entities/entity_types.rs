/// Modular entity types system
/// 
/// This module defines different entity types (humans, animals, etc.) with their
/// unique properties while sharing common systems (movement, stats, AI).
use bevy::prelude::*;
use super::{TilePosition, MovementSpeed, EntityStatsBundle, Creature, CurrentAction};
use super::types::rabbit::RabbitBehavior;
use super::types::deer::DeerBehavior;
use crate::pathfinding::PathfindingGrid;
use rand::Rng;

// ============================================================================
// ENTITY TYPE MARKERS
// ============================================================================

/// Marker component for human entities
#[derive(Component, Debug, Clone, Copy)]
pub struct Human;

/// Marker component for rabbit entities
#[derive(Component, Debug, Clone, Copy)]
pub struct Rabbit;

/// Marker component for deer entities (future)
#[derive(Component, Debug, Clone, Copy)]
pub struct Deer;

/// Marker component for wolf entities (future)
#[derive(Component, Debug, Clone, Copy)]
pub struct Wolf;

// ============================================================================
// ENTITY TEMPLATES
// ============================================================================

/// Template defining entity properties
#[derive(Debug, Clone)]
pub struct EntityTemplate {
    pub name_prefix: &'static str,
    pub species: &'static str,
    pub movement_speed: u32,  // Ticks per tile
    pub wander_radius: i32,
    pub emoji: &'static str,  // For future rendering customization
}

impl EntityTemplate {
    /// Human template - standard walking speed (Dwarf Fortress-like)
    pub const HUMAN: EntityTemplate = EntityTemplate {
        name_prefix: "Person",
        species: "Human",
        movement_speed: 30,  // 3 seconds per tile at 10 TPS (comfortable walking)
        wander_radius: 30,
        emoji: "🧍‍♂️",
    };

    /// Rabbit template - faster, smaller wander radius
    pub const RABBIT: EntityTemplate = EntityTemplate {
        name_prefix: "Rabbit",
        species: "Rabbit",
        movement_speed: 20,   // 2 seconds per tile at 10 TPS (faster than humans)
        wander_radius: 15,   // Smaller territory
        emoji: "🐇",
    };

    /// Deer template (future) - moderate speed, large range
    pub const DEER: EntityTemplate = EntityTemplate {
        name_prefix: "Deer",
        species: "Deer",
        movement_speed: 10,  // 1.0 tiles/sec at 10 TPS
        wander_radius: 40,   // Large territory
        emoji: "🦌",
    };

    /// Wolf template (future) - fast predator
    pub const WOLF: EntityTemplate = EntityTemplate {
        name_prefix: "Wolf",
        species: "Wolf",
        movement_speed: 6,   // 1.67 tiles/sec at 10 TPS (very fast)
        wander_radius: 50,   // Roams widely
        emoji: "🐺",
    };
}

// ============================================================================
// SPAWN FUNCTIONS
// ============================================================================

/// Spawn a human entity
pub fn spawn_human(
    commands: &mut Commands,
    name: impl Into<String>,
    position: IVec2,
) -> Entity {
    let template = EntityTemplate::HUMAN;
    
    commands.spawn((
        Creature {
            name: name.into(),
            species: template.species.to_string(),
        },
        Human,
        TilePosition::from_tile(position),
        MovementSpeed::custom(template.movement_speed),
        EntityStatsBundle::default(),
        // NO Wanderer component - movement driven by utility AI!
    )).id()
}

/// Spawn a rabbit entity
pub fn spawn_rabbit(
    commands: &mut Commands,
    name: impl Into<String>,
    position: IVec2,
) -> Entity {
    let template = EntityTemplate::RABBIT;
    
    commands.spawn((
        Creature {
            name: name.into(),
            species: template.species.to_string(),
        },
        Rabbit,
        TilePosition::from_tile(position),
        MovementSpeed::custom(template.movement_speed),
        EntityStatsBundle::default(),
        RabbitBehavior::config(), // Attach behavior configuration
        CurrentAction::none(), // Track current action for viewer
        // NO Wanderer component - movement driven by utility AI!
    )).id()
}

/// Spawn a deer entity
pub fn spawn_deer(
    commands: &mut Commands,
    name: impl Into<String>,
    position: IVec2,
) -> Entity {
    let template = EntityTemplate::DEER;
    
    commands.spawn((
        Creature {
            name: name.into(),
            species: template.species.to_string(),
        },
        Deer,
        TilePosition::from_tile(position),
        MovementSpeed::custom(template.movement_speed),
        EntityStatsBundle::default(),
        DeerBehavior::config(), // Attach behavior configuration
        CurrentAction::none(), // Track current action for viewer
    )).id()
}

/// Spawn multiple humans at random positions
pub fn spawn_humans(
    commands: &mut Commands,
    count: usize,
    center: IVec2,
    spawn_radius: i32,
    grid: &PathfindingGrid,
) -> Vec<Entity> {
    let mut entities = Vec::new();
    let mut rng = rand::thread_rng();
    
    for i in 0..count {
        if let Some(spawn_pos) = pick_random_walkable_tile(center, spawn_radius, grid, &mut rng) {
            let name = format!("Human_{}", i);
            let entity = spawn_human(commands, name, spawn_pos);
            entities.push(entity);
            info!("Spawned human at {:?}", spawn_pos);
        } else {
            warn!("Couldn't find walkable spawn position for human {}", i);
        }
    }
    
    entities
}

/// Spawn multiple rabbits at random positions
pub fn spawn_rabbits(
    commands: &mut Commands,
    count: usize,
    center: IVec2,
    spawn_radius: i32,
    grid: &PathfindingGrid,
) -> Vec<Entity> {
    let mut entities = Vec::new();
    let mut rng = rand::thread_rng();
    
    for i in 0..count {
        if let Some(spawn_pos) = pick_random_walkable_tile(center, spawn_radius, grid, &mut rng) {
            let name = format!("Rabbit_{}", i);
            let entity = spawn_rabbit(commands, name, spawn_pos);
            entities.push(entity);
            info!("Spawned rabbit at {:?}", spawn_pos);
        } else {
            warn!("Couldn't find walkable spawn position for rabbit {}", i);
        }
    }
    
    entities
}

// ============================================================================
// FUTURE: Entity-Specific Behaviors
// ============================================================================

/// Example: Rabbit-specific behavior system (future)
/// Rabbits could have unique behaviors like:
/// - Flee from predators
/// - Eat grass/plants
/// - Breed when healthy
pub fn rabbit_behavior_system(
    // query: Query<&Rabbit, With<Wanderer>>,
) {
    // Future: Add rabbit-specific behaviors here
    // For now, rabbits just use the standard wandering system
}

/// Example: Wolf hunting system (future)
pub fn wolf_hunting_system() {
    // Future: Wolves hunt rabbits/deer
}

// ============================================================================
// QUERY HELPERS
// ============================================================================

/// Count entities by type
pub fn count_entities_by_type(
    humans: Query<(), With<Human>>,
    rabbits: Query<(), With<Rabbit>>,
) -> (usize, usize) {
    (humans.iter().count(), rabbits.iter().count())
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Pick a random walkable tile within radius
fn pick_random_walkable_tile(
    center: IVec2,
    radius: i32,
    grid: &PathfindingGrid,
    rng: &mut impl Rng,
) -> Option<IVec2> {
    // Try up to 20 random positions
    for _ in 0..20 {
        let offset_x = rng.gen_range(-radius..=radius);
        let offset_y = rng.gen_range(-radius..=radius);
        let candidate = center + IVec2::new(offset_x, offset_y);
        
        if grid.is_walkable(candidate) {
            return Some(candidate);
        }
    }
    
    // If we couldn't find anything, try a simple grid search
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            let candidate = center + IVec2::new(dx, dy);
            if grid.is_walkable(candidate) {
                return Some(candidate);
            }
        }
    }
    
    None
}
