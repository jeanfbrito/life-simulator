/// Species Registry - Centralized species management system
///
/// This module provides a centralized registry for all species in the game.
/// Instead of hard-coding spawn logic, behavior configs, and stats in multiple places,
/// species register themselves with descriptors that the core engine can iterate over.
use bevy::prelude::*;
use rand::Rng;

use super::{Bear, Deer, Fox, Herbivore, Human, Rabbit, Raccoon, Wolf};
use crate::entities::reproduction::{Age, ReproductionCooldown, Sex, WellFedStreak};
use crate::entities::{Creature, CurrentAction, EntityStatsBundle, MovementSpeed, TilePosition};
use crate::pathfinding::PathfindingGrid;

// ============================================================================
// REGISTRY TYPES
// ============================================================================

/// A function that spawns an entity of a specific species
pub type SpawnFunction = fn(&mut Commands, name: String, position: IVec2) -> Entity;

/// Descriptor containing all information needed to spawn and manage a species
#[derive(Debug, Clone)]
pub struct SpeciesDescriptor {
    /// Species identifier
    pub species: &'static str,

    /// Name prefix for spawned entities
    pub name_prefix: &'static str,

    /// Emoji for visualization
    pub emoji: &'static str,

    /// Spawn function
    pub spawn_fn: SpawnFunction,

    /// Movement speed in ticks per tile
    pub movement_speed: u32,

    /// Default wander radius
    pub wander_radius: i32,

    /// Whether this species is considered a juvenile (affects naming/scaling)
    pub is_juvenile: bool,

    /// Juvenile naming scheme (if applicable)
    pub juvenile_name_prefix: Option<&'static str>,

    /// Viewer ordering priority (lower = rendered first/behind)
    pub viewer_order: i32,

    /// Scale factor for viewer rendering
    pub viewer_scale: f32,

    /// Color for viewer legend/tooltips
    pub viewer_color: &'static str,
}

// ============================================================================
// CENTRAL REGISTRY
// ============================================================================

/// Global registry containing all registered species
pub static SPECIES_REGISTRY: SpeciesRegistry = SpeciesRegistry::new();

/// Registry that holds all species descriptors
pub struct SpeciesRegistry {
    descriptors: &'static [SpeciesDescriptor],
}

impl SpeciesRegistry {
    /// Create the species registry with all registered species
    pub const fn new() -> Self {
        // Define all species in one place
        const DESCRIPTORS: &[SpeciesDescriptor] = &[
            SpeciesDescriptor {
                species: "Human",
                name_prefix: "Person",
                emoji: "🧍‍♂️",
                spawn_fn: spawn_human_registry,
                movement_speed: 30,
                wander_radius: 30,
                is_juvenile: false,
                juvenile_name_prefix: None,
                viewer_order: 100,
                viewer_scale: 1.2,
                viewer_color: "#4a90e2",
            },
            SpeciesDescriptor {
                species: "Rabbit",
                name_prefix: "Rabbit",
                emoji: "🐇",
                spawn_fn: spawn_rabbit_registry,
                movement_speed: 20,
                wander_radius: 15,
                is_juvenile: false,
                juvenile_name_prefix: Some("Bunny"),
                viewer_order: 50,
                viewer_scale: 0.5,
                viewer_color: "#8b4513",
            },
            SpeciesDescriptor {
                species: "Deer",
                name_prefix: "Deer",
                emoji: "🦌",
                spawn_fn: spawn_deer_registry,
                movement_speed: 10,
                wander_radius: 40,
                is_juvenile: false,
                juvenile_name_prefix: Some("Fawn"),
                viewer_order: 60,
                viewer_scale: 0.9,
                viewer_color: "#a0522d",
            },
            SpeciesDescriptor {
                species: "Raccoon",
                name_prefix: "Raccoon",
                emoji: "🦝",
                spawn_fn: spawn_raccoon_registry,
                movement_speed: 16,
                wander_radius: 25,
                is_juvenile: false,
                juvenile_name_prefix: Some("Kit"),
                viewer_order: 55,
                viewer_scale: 0.65,
                viewer_color: "#696969",
            },
            SpeciesDescriptor {
                species: "Bear",
                name_prefix: "Bear",
                emoji: "🐻",
                spawn_fn: spawn_bear_registry,
                movement_speed: 12,
                wander_radius: 80,
                is_juvenile: false,
                juvenile_name_prefix: Some("Cub"),
                viewer_order: 70,
                viewer_scale: 1.2,
                viewer_color: "#3b2f2f",
            },
            SpeciesDescriptor {
                species: "Fox",
                name_prefix: "Fox",
                emoji: "🦊",
                spawn_fn: spawn_fox_registry,
                movement_speed: 16,
                wander_radius: 40,
                is_juvenile: false,
                juvenile_name_prefix: Some("Kit"),
                viewer_order: 65,
                viewer_scale: 0.6,
                viewer_color: "#c1440e",
            },
            SpeciesDescriptor {
                species: "Wolf",
                name_prefix: "Wolf",
                emoji: "🐺",
                spawn_fn: spawn_wolf_registry,
                movement_speed: 12,
                wander_radius: 200,
                is_juvenile: false,
                juvenile_name_prefix: Some("Pup"),
                viewer_order: 80,
                viewer_scale: 0.9,
                viewer_color: "#666666",
            },
        ];

        Self {
            descriptors: DESCRIPTORS,
        }
    }

    /// Get all species descriptors
    pub fn get_descriptors(&self) -> &[SpeciesDescriptor] {
        self.descriptors
    }

    /// Find a descriptor by species name
    pub fn find_by_species(&self, species: &str) -> Option<&SpeciesDescriptor> {
        self.descriptors.iter().find(|d| d.species == species)
    }

    /// Get descriptor for rabbit
    pub fn rabbit(&self) -> &SpeciesDescriptor {
        self.find_by_species("Rabbit").unwrap()
    }

    /// Get descriptor for deer
    pub fn deer(&self) -> &SpeciesDescriptor {
        self.find_by_species("Deer").unwrap()
    }

    /// Get descriptor for raccoon
    pub fn raccoon(&self) -> &SpeciesDescriptor {
        self.find_by_species("Raccoon").unwrap()
    }

    /// Get descriptor for human
    pub fn human(&self) -> &SpeciesDescriptor {
        self.find_by_species("Human").unwrap()
    }

    /// Get descriptor for wolf
    pub fn wolf(&self) -> &SpeciesDescriptor {
        self.find_by_species("Wolf").unwrap()
    }
}

// ============================================================================
// REGISTRY SPAWN FUNCTIONS
// ============================================================================

/// Registry-based spawn function for humans
fn spawn_human_registry(commands: &mut Commands, name: String, position: IVec2) -> Entity {
    let descriptor = SPECIES_REGISTRY.human();

    commands
        .spawn((
            Creature {
                name,
                species: descriptor.species.to_string(),
            },
            Human,
            TilePosition::from_tile(position),
            MovementSpeed::custom(descriptor.movement_speed),
            EntityStatsBundle::default(),
            CurrentAction::none(),
        ))
        .id()
}

/// Registry-based spawn function for rabbits
fn spawn_rabbit_registry(commands: &mut Commands, name: String, position: IVec2) -> Entity {
    let descriptor = SPECIES_REGISTRY.rabbit();

    use super::types::rabbit::RabbitBehavior;

    let cfg = RabbitBehavior::reproduction_config();
    let mut rng = rand::thread_rng();
    let sex = if rng.gen_bool(0.5) {
        Sex::Male
    } else {
        Sex::Female
    };

    commands
        .spawn((
            Creature {
                name,
                species: descriptor.species.to_string(),
            },
            Herbivore,
            Rabbit,
            TilePosition::from_tile(position),
            MovementSpeed::custom(descriptor.movement_speed),
            RabbitBehavior::stats_bundle(),
            RabbitBehavior::config(),
            RabbitBehavior::needs(),
            sex,
            Age {
                ticks_alive: cfg.maturity_ticks as u64,
                mature_at_ticks: cfg.maturity_ticks,
            }, // spawn as adult
            ReproductionCooldown::default(),
            WellFedStreak::default(),
            CurrentAction::none(),
            cfg,
        ))
        .id()
}

/// Registry-based spawn function for deer
fn spawn_deer_registry(commands: &mut Commands, name: String, position: IVec2) -> Entity {
    let descriptor = SPECIES_REGISTRY.deer();

    use super::types::deer::DeerBehavior;

    let cfg = DeerBehavior::reproduction_config();
    let mut rng = rand::thread_rng();
    let sex = if rng.gen_bool(0.5) {
        Sex::Male
    } else {
        Sex::Female
    };

    commands
        .spawn((
            Creature {
                name,
                species: descriptor.species.to_string(),
            },
            Herbivore,
            Deer,
            TilePosition::from_tile(position),
            MovementSpeed::custom(descriptor.movement_speed),
            DeerBehavior::stats_bundle(),
            DeerBehavior::config(),
            DeerBehavior::needs(),
            sex,
            Age {
                ticks_alive: cfg.maturity_ticks as u64,
                mature_at_ticks: cfg.maturity_ticks,
            }, // spawn as adult
            ReproductionCooldown::default(),
            WellFedStreak::default(),
            CurrentAction::none(),
            cfg,
        ))
        .id()
}

/// Registry-based spawn function for raccoons
fn spawn_raccoon_registry(commands: &mut Commands, name: String, position: IVec2) -> Entity {
    let descriptor = SPECIES_REGISTRY.raccoon();

    use super::types::raccoon::RaccoonBehavior;

    let cfg = RaccoonBehavior::reproduction_config();
    let mut rng = rand::thread_rng();
    let sex = if rng.gen_bool(0.5) {
        Sex::Male
    } else {
        Sex::Female
    };

    commands
        .spawn((
            Creature {
                name,
                species: descriptor.species.to_string(),
            },
            Herbivore,
            Raccoon,
            TilePosition::from_tile(position),
            MovementSpeed::custom(descriptor.movement_speed),
            RaccoonBehavior::stats_bundle(),
            RaccoonBehavior::config(),
            RaccoonBehavior::needs(),
            sex,
            Age {
                ticks_alive: cfg.maturity_ticks as u64,
                mature_at_ticks: cfg.maturity_ticks,
            },
            ReproductionCooldown::default(),
            WellFedStreak::default(),
            CurrentAction::none(),
            cfg,
        ))
        .id()
}

/// Registry-based spawn function for bears
fn spawn_bear_registry(commands: &mut Commands, name: String, position: IVec2) -> Entity {
    let descriptor = SPECIES_REGISTRY
        .find_by_species("Bear")
        .expect("Bear descriptor registered");

    use super::types::bear::BearBehavior;

    let cfg = BearBehavior::reproduction_config();
    let mut rng = rand::thread_rng();
    let sex = if rng.gen_bool(0.5) {
        Sex::Male
    } else {
        Sex::Female
    };

    commands
        .spawn((
            Creature {
                name,
                species: descriptor.species.to_string(),
            },
            Bear,
            TilePosition::from_tile(position),
            MovementSpeed::custom(descriptor.movement_speed),
            BearBehavior::stats_bundle(),
            BearBehavior::config(),
            BearBehavior::needs(),
            sex,
            Age {
                ticks_alive: cfg.maturity_ticks as u64,
                mature_at_ticks: cfg.maturity_ticks,
            },
            ReproductionCooldown::default(),
            WellFedStreak::default(),
            CurrentAction::none(),
            cfg,
        ))
        .id()
}

/// Registry-based spawn function for foxes
fn spawn_fox_registry(commands: &mut Commands, name: String, position: IVec2) -> Entity {
    let descriptor = SPECIES_REGISTRY
        .find_by_species("Fox")
        .expect("Fox descriptor registered");

    use super::types::fox::FoxBehavior;

    let cfg = FoxBehavior::reproduction_config();
    let mut rng = rand::thread_rng();
    let sex = if rng.gen_bool(0.5) {
        Sex::Male
    } else {
        Sex::Female
    };

    commands
        .spawn((
            Creature {
                name,
                species: descriptor.species.to_string(),
            },
            Fox,
            TilePosition::from_tile(position),
            MovementSpeed::custom(descriptor.movement_speed),
            FoxBehavior::stats_bundle(),
            FoxBehavior::config(),
            FoxBehavior::needs(),
            sex,
            Age {
                ticks_alive: cfg.maturity_ticks as u64,
                mature_at_ticks: cfg.maturity_ticks,
            },
            ReproductionCooldown::default(),
            WellFedStreak::default(),
            CurrentAction::none(),
            cfg,
        ))
        .id()
}

/// Registry-based spawn function for wolves
fn spawn_wolf_registry(commands: &mut Commands, name: String, position: IVec2) -> Entity {
    let descriptor = SPECIES_REGISTRY.wolf();

    use super::types::wolf::WolfBehavior;

    let cfg = WolfBehavior::reproduction_config();
    let mut rng = rand::thread_rng();
    let sex = if rng.gen_bool(0.5) {
        Sex::Male
    } else {
        Sex::Female
    };

    commands
        .spawn((
            Creature {
                name,
                species: descriptor.species.to_string(),
            },
            Wolf,
            TilePosition::from_tile(position),
            MovementSpeed::custom(descriptor.movement_speed),
            WolfBehavior::stats_bundle(),
            WolfBehavior::config(),
            WolfBehavior::needs(),
            sex,
            Age {
                ticks_alive: cfg.maturity_ticks as u64,
                mature_at_ticks: cfg.maturity_ticks,
            },
            ReproductionCooldown::default(),
            WellFedStreak::default(),
            CurrentAction::none(),
            cfg,
        ))
        .id()
}

// ============================================================================
// SPAWN HELPERS
// ============================================================================

/// Spawn multiple entities of a specific species
pub fn spawn_species_batch(
    commands: &mut Commands,
    species: &str,
    count: usize,
    center: IVec2,
    spawn_radius: i32,
    grid: &PathfindingGrid,
) -> Vec<Entity> {
    let descriptor = SPECIES_REGISTRY
        .find_by_species(species)
        .unwrap_or_else(|| panic!("Unknown species: {}", species));

    let mut entities = Vec::new();
    let mut rng = rand::thread_rng();

    for i in 0..count {
        if let Some(spawn_pos) = pick_random_walkable_tile(center, spawn_radius, grid, &mut rng) {
            let name = format!("{}_{}", descriptor.name_prefix, i);
            let entity = (descriptor.spawn_fn)(commands, name, spawn_pos);
            entities.push(entity);
            info!("Spawned {} at {:?}", species, spawn_pos);
        } else {
            warn!(
                "Couldn't find walkable spawn position for {} {}",
                species, i
            );
        }
    }

    entities
}

/// Spawn entities using the registry
pub fn spawn_using_registry(
    commands: &mut Commands,
    species: &str,
    name: String,
    position: IVec2,
) -> Entity {
    let descriptor = SPECIES_REGISTRY
        .find_by_species(species)
        .unwrap_or_else(|| panic!("Unknown species: {}", species));

    (descriptor.spawn_fn)(commands, name, position)
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

// ============================================================================
// LEGACY WRAPPER FUNCTIONS
// ============================================================================

/// Legacy wrapper for spawn_human - maintains compatibility
pub fn spawn_human_legacy(
    commands: &mut Commands,
    name: impl Into<String>,
    position: IVec2,
) -> Entity {
    spawn_human_registry(commands, name.into(), position)
}

/// Legacy wrapper for spawn_rabbit - maintains compatibility
pub fn spawn_rabbit_legacy(
    commands: &mut Commands,
    name: impl Into<String>,
    position: IVec2,
) -> Entity {
    spawn_rabbit_registry(commands, name.into(), position)
}

/// Legacy wrapper for spawn_deer - maintains compatibility
pub fn spawn_deer_legacy(
    commands: &mut Commands,
    name: impl Into<String>,
    position: IVec2,
) -> Entity {
    spawn_deer_registry(commands, name.into(), position)
}

/// Legacy wrapper for spawn_raccoon - maintains compatibility
pub fn spawn_raccoon_legacy(
    commands: &mut Commands,
    name: impl Into<String>,
    position: IVec2,
) -> Entity {
    spawn_raccoon_registry(commands, name.into(), position)
}

/// Legacy wrapper for spawn_humans - maintains compatibility
pub fn spawn_humans_legacy(
    commands: &mut Commands,
    count: usize,
    center: IVec2,
    spawn_radius: i32,
    grid: &PathfindingGrid,
) -> Vec<Entity> {
    spawn_species_batch(commands, "Human", count, center, spawn_radius, grid)
}

/// Legacy wrapper for spawn_rabbits - maintains compatibility
pub fn spawn_rabbits_legacy(
    commands: &mut Commands,
    count: usize,
    center: IVec2,
    spawn_radius: i32,
    grid: &PathfindingGrid,
) -> Vec<Entity> {
    spawn_species_batch(commands, "Rabbit", count, center, spawn_radius, grid)
}
