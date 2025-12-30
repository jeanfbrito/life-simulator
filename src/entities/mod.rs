pub mod active_action;
pub mod ai_bundle;
pub mod birth_relationships;
pub mod cached_state;
pub mod carcass;
pub mod current_action;
pub mod entity_tracker;
pub mod entity_types;
/// Entities module - manages creatures and their behaviors
pub mod fear;
pub mod group_config;
pub mod hunting_relationships;
pub mod mating_relationships;
pub mod movement;
pub mod movement_component;
pub mod pack_relationships;
pub mod parent_child_relationships;
pub mod registry;
pub mod reproduction;
pub mod spatial_cell;
pub mod spatial_index;
pub mod spatial_maintenance;
pub mod spawn_config;
pub mod stats;
pub mod systems_registry;
pub mod types;

use bevy::prelude::*;

pub use movement::{
    execute_movement_component, get_position, is_moving, issue_move_order, stop_movement, MoveOrder, MovementSpeed,
    MovementState, TilePosition,
};

pub use movement_component::MovementComponent;

pub use spatial_index::{EntityType as SpatialEntityType, SpatialEntityIndex};

pub use spatial_cell::{
    entities_in_radius_via_children, spawn_spatial_grid,
    SpatialCell, SpatialCellGrid, SpatiallyParented,
    CHUNK_SIZE,
};

// Wandering component REMOVED - use utility AI Wander action instead!

pub use entity_tracker::{get_entities_json, init_entity_tracker, sync_entities_to_tracker};

pub use stats::{
    death_system, get_most_urgent_need, movement_energy_system, need_damage_system, tick_stats_system,
    utility_drink, utility_eat, utility_heal, utility_rest, Energy, EntityStatsBundle, Health,
    Hunger, Stat, Thirst,
};

pub use cached_state::{CachedEntityState, update_cached_entity_state_system};

pub use carcass::{tick_carcasses, Carcass};

pub use fear::{fear_speed_system, predator_proximity_system, FearPlugin, FearState};

pub use hunting_relationships::{ActiveHunter, HuntingTarget};

pub use mating_relationships::{ActiveMate, MatingTarget};

pub use pack_relationships::{PackLeader, PackMember, GroupType};

pub use group_config::GroupFormationConfig;

pub use parent_child_relationships::{BirthInfo, LegacyChildOf, LegacyParentOf};

// Backward compatibility type aliases (deprecated)
#[deprecated(since = "0.1.0", note = "Use LegacyChildOf instead")]
pub type ChildOf = LegacyChildOf;

#[deprecated(since = "0.1.0", note = "Use LegacyParentOf instead")]
pub type ParentOf = LegacyParentOf;

pub use entity_types::{
    count_entities_by_type, spawn_bear, spawn_deer, spawn_fox, spawn_human, spawn_humans,
    spawn_rabbit, spawn_rabbits, spawn_raccoon, spawn_wolf, Bear, Deer, EntityTemplate, Fox,
    Herbivore, Human, Rabbit, Raccoon, Wolf,
};

pub use registry::{
    spawn_deer_legacy, spawn_human_legacy, spawn_humans_legacy, spawn_rabbit_legacy,
    spawn_rabbits_legacy, spawn_raccoon_legacy, spawn_species_batch, spawn_using_registry,
    SpeciesDescriptor, SpeciesRegistry, SPECIES_REGISTRY,
};

/// Generate JSON metadata for all species for viewer configuration
pub fn get_species_metadata_json() -> String {
    use crate::entities::registry::SPECIES_REGISTRY;
    use std::collections::HashMap;

    let mut species_data: HashMap<String, serde_json::Value> = HashMap::new();

    for descriptor in SPECIES_REGISTRY.get_descriptors() {
        let species_info = serde_json::json!({
            "name": descriptor.species,
            "emoji": descriptor.emoji,
            "viewer_scale": descriptor.viewer_scale,
            "viewer_order": descriptor.viewer_order,
            "viewer_color": descriptor.viewer_color,
            "name_prefix": descriptor.name_prefix,
            "juvenile_name_prefix": descriptor.juvenile_name_prefix,
            "is_juvenile": descriptor.is_juvenile,
            "movement_speed": descriptor.movement_speed,
            "wander_radius": descriptor.wander_radius
        });

        species_data.insert(descriptor.species.to_string(), species_info);
    }

    // Add juvenile scaling information that was previously hard-coded in renderer
    let juvenile_scales = serde_json::json!({
        "Rabbit": 0.7,
        "Deer": 0.8,
        "Raccoon": 0.75,
        "Bear": 0.65,
        "Fox": 0.6,
        "Wolf": 0.75
    });

    let result = serde_json::json!({
        "species": species_data,
        "juvenile_scales": juvenile_scales,
        "default_entity": {
            "emoji": "❓",
            "sizeMultiplier": 1.0,
            "offsetX": 0.0,
            "offsetY": -0.2
        }
    });

    result.to_string()
}

pub use systems_registry::{
    get_birth_system_names, get_mate_matching_system_names, get_planner_system_names,
    SpeciesSystemsDescriptor, SpeciesSystemsRegistry, SPECIES_SYSTEMS_REGISTRY,
};

pub use reproduction::{
    mate_matching_system, tick_reproduction_timers_system, update_age_and_wellfed_system, Age,
    MatingIntent, Mother, Pregnancy, ReproductionConfig, ReproductionCooldown, Sex, WellFedStreak,
};

pub use types::bear::{bear_birth_system, bear_mate_matching_system, plan_bear_actions};
pub use types::deer::{deer_birth_system, deer_mate_matching_system, plan_deer_actions};
pub use types::fox::{fox_birth_system, fox_mate_matching_system, plan_fox_actions};
pub use types::rabbit::{plan_rabbit_actions, rabbit_birth_system, rabbit_mate_matching_system};
pub use types::raccoon::{
    plan_raccoon_actions, raccoon_birth_system, raccoon_mate_matching_system,
};
pub use types::wolf::{plan_wolf_actions, wolf_birth_system, wolf_mate_matching_system};
pub use types::{BehaviorConfig, SpeciesNeeds};

pub use active_action::ActiveAction;
pub use ai_bundle::AIEntityBundle;
pub use current_action::CurrentAction;

pub use spawn_config::{
    spawn_entities_from_config, SpawnArea, SpawnConfig, SpawnGroup, SpawnMessages, SpawnSettings,
    SpawnSex,
};

// ============================================================================
// ENTITY TYPES
// ============================================================================

/// Basic creature entity
#[derive(Component, Debug, Clone)]
pub struct Creature {
    pub name: String,
    pub species: String,
}

impl Default for Creature {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            species: "Unknown".to_string(),
        }
    }
}

/// Marker for different entity types
#[derive(Component, Debug)]
pub enum EntityType {
    Human,
    Animal,
    Monster,
}

// ============================================================================
// PLUGIN
// ============================================================================

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut App) {
        use crate::simulation::SimulationSet;

        app
            // Add fear system plugin
            .add_plugins(FearPlugin)
            // Startup
            .add_systems(Startup, (
                entity_tracker::init_entity_tracker,
                spawn_spatial_grid,  // Phase 4.1: Spawn spatial grid hierarchy
            ))
            // Non-tick systems (run every frame)
            .add_systems(
                Update,
                (
                    movement::initiate_pathfinding,
                    movement::initialize_movement_state,
                    entity_tracker::sync_entities_to_tracker, // Sync for web API
                    // Phase 7: Spatial parent updates now handled by TilePosition component hooks
                ),
            )
            // === PLANNING PHASE ===
            // NOTE: Species planners (plan_rabbit_actions, plan_deer_actions, etc.) are
            // registered in EventDrivenPlannerPlugin where they run AFTER ultrathink_system
            // in a proper chain. Do NOT register them here to avoid duplicate execution.
            //
            // The group formation/cohesion systems still run here in SimulationSet::Planning
            .add_systems(
                Update,
                (
                    // Generic group formation systems (work for all species with GroupFormationConfig)
                    crate::ai::generic_group_formation_system,
                    crate::ai::generic_group_cohesion_system,
                    crate::ai::process_member_removals,
                )
                    .in_set(SimulationSet::Planning)
                    .run_if(should_run_tick_systems),
            )
            // === MOVEMENT PHASE ===
            // Movement systems run in parallel
            .add_systems(
                Update,
                (
                    movement::tick_movement_system,       // Legacy movement
                    movement::execute_movement_component, // Phase 3 movement
                )
                    .in_set(SimulationSet::Movement)
                    .after(SimulationSet::ActionExecution)
                    .run_if(should_run_tick_systems),
            )
            // === STATS PHASE ===
            // Movement energy and grazing hunger must run first to set rates before tick applies them
            .add_systems(
                Update,
                (
                    stats::movement_energy_system, // Set energy rate based on movement
                    stats::grazing_hunger_system,  // Reduce hunger while grazing (like rest restores energy)
                    stats::tick_stats_system,      // Apply hunger, thirst, energy decay
                )
                    .chain() // MUST run in order: energy rate → grazing reduction → tick update
                    .in_set(SimulationSet::Stats)
                    .after(SimulationSet::Movement)
                    .run_if(should_run_tick_systems),
            )
            // === REPRODUCTION PHASE ===
            // Reproduction systems run in parallel (can also run parallel with Stats)
            .add_systems(
                Update,
                (
                    // Age and timers
                    update_age_and_wellfed_system,
                    tick_reproduction_timers_system,
                    // Mate matching (all species)
                    rabbit_mate_matching_system,
                    deer_mate_matching_system,
                    raccoon_mate_matching_system,
                    bear_mate_matching_system,
                    fox_mate_matching_system,
                    wolf_mate_matching_system,
                    // Birth systems (all species)
                    rabbit_birth_system,
                    deer_birth_system,
                    raccoon_birth_system,
                    bear_birth_system,
                    fox_birth_system,
                    wolf_birth_system,
                    // Establish parent-child relationships using Bevy hierarchy
                    birth_relationships::establish_birth_relationships,
                )
                    .in_set(SimulationSet::Reproduction)
                    .after(SimulationSet::Movement)
                    .run_if(should_run_tick_systems),
            )
            // === CLEANUP PHASE ===
            // Cleanup must run last
            .add_systems(
                Update,
                (
                    stats::death_system,
                    tick_carcasses,
                    // Phase 7: Spatial reparenting now handled by TilePosition component hooks
                )
                    .in_set(SimulationSet::Cleanup)
                    .after(SimulationSet::Stats)
                    .after(SimulationSet::Reproduction)
                    .run_if(should_run_tick_systems),
            );
    }
}

/// Run condition for tick-based systems
fn should_run_tick_systems(state: Res<crate::simulation::SimulationState>) -> bool {
    state.should_tick
}

// ============================================================================
// SPAWN HELPERS
// ============================================================================

/// Spawn a basic creature at a tile position
pub fn spawn_creature(
    commands: &mut Commands,
    name: impl Into<String>,
    species: impl Into<String>,
    tile_pos: IVec2,
    speed: MovementSpeed,
) -> Entity {
    commands
        .spawn((
            Creature {
                name: name.into(),
                species: species.into(),
            },
            TilePosition::from_tile(tile_pos),
            speed,
        ))
        .id()
}
