use crate::entities::stats::{Energy, Hunger, Thirst};
use crate::entities::{Carcass, Creature, MoveOrder, SpeciesNeeds, TilePosition};
use crate::pathfinding::{GridPathRequest, Path, PathfindingFailed};
use crate::resources::ResourceType;
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;
use crate::types::newtypes::Utility;
/// Action system for TQUAI
///
/// Actions are discrete behaviors that can be queued and executed on ticks.
/// They can be instant (complete in one tick) or multi-tick (span multiple ticks).
use bevy::prelude::*;
use rand::Rng;

use crate::simulation::tick::SimulationTick;

const DEFAULT_CARCASS_DECAY: u32 = 6_000;
const MIN_CARCASS_NUTRITION: f32 = 5.0;

/// Result of executing an action
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionResult {
    /// Action completed successfully
    Success,
    /// Action failed (preconditions not met, resource unavailable, etc.)
    Failed,
    /// Action is still in progress (will continue next tick)
    InProgress,
    /// Action completed but should trigger a follow-up action
    /// Used for giving-up behavior when patch quality is too low
    TriggerFollowUp,
    /// Action needs pathfinding to target position
    /// System layer will queue pathfinding request and transition action to WaitingForPath
    NeedsPathfinding { target: IVec2 },
}

/// Request to queue an action
#[derive(Debug, Clone)]
pub struct ActionRequest {
    pub entity: Entity,
    pub action_type: ActionType,
    /// How desirable this action is (0.0-1.0) - now typed for clarity
    pub utility: Utility,
    pub priority: i32,
}

/// Types of actions available
#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    DrinkWater {
        target_tile: IVec2,
    },
    Graze {
        target_tile: IVec2,
    }, // Move to grass tile (eating happens via auto-eat system)
    Rest {
        duration_ticks: u32,
    },
    Scavenge {
        carcass: Entity,
    },
    Hunt {
        prey: Entity,
    },
    Follow {
        target: Entity,
        stop_distance: i32,
    },
    Mate {
        partner: Entity,
        meeting_tile: IVec2,
        duration_ticks: u32,
    },
    Harvest {
        target_tile: IVec2,
        resource_type: crate::resources::ResourceType,
    },
    Wander {
        target_tile: IVec2,
    },
    // Future actions:
    // Hunt { target: Entity },
    // Flee { from: Entity },
}

/// Core Action trait
/// All actions must implement this to be executable in the TQUAI system
pub trait Action: Send + Sync {
    /// Check if action can be executed (preconditions)
    fn can_execute(&self, world: &World, entity: Entity) -> bool;

    /// Execute the action for this tick
    /// Returns Success/Failed/InProgress
    /// Uses read-only World access - mutations handled by systems via Commands
    fn execute(&mut self, world: &World, entity: Entity) -> ActionResult;

    /// Cancel the action (called when a higher priority action needs to interrupt)
    /// Default implementation does nothing - override for actions that need cleanup
    fn cancel(&mut self, world: &World, entity: Entity) {
        // Default: no cleanup needed
    }

    /// Get action name for debugging
    fn name(&self) -> &'static str;
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Find a walkable tile adjacent to a water tile
fn find_adjacent_walkable_tile(water_pos: IVec2, world_loader: &WorldLoader) -> Option<IVec2> {
    // Check all 8 adjacent tiles (including diagonals)
    let adjacent_offsets = [
        IVec2::new(0, 1),
        IVec2::new(1, 0),
        IVec2::new(0, -1),
        IVec2::new(-1, 0),
        IVec2::new(1, 1),
        IVec2::new(1, -1),
        IVec2::new(-1, 1),
        IVec2::new(-1, -1),
    ];

    for offset in adjacent_offsets {
        let check_pos = water_pos + offset;

        if let Some(terrain_str) = world_loader.get_terrain_at(check_pos.x, check_pos.y) {
            if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                // Must be walkable but NOT water
                if terrain.is_walkable()
                    && !matches!(
                        terrain,
                        TerrainType::ShallowWater | TerrainType::DeepWater | TerrainType::Water
                    )
                {
                    // CRITICAL: Also check that tile doesn't have resources blocking it
                    let has_blocking_resource = world_loader
                        .get_resource_at(check_pos.x, check_pos.y)
                        .map(|r| !r.is_empty())
                        .unwrap_or(false);

                    if !has_blocking_resource {
                        return Some(check_pos);
                    }
                }
            }
        }
    }

    None
}

/// Remove movement-related components so a cancelled action stops any in-flight navigation
/// NOTE: This function is deprecated in favor of using Commands in the system layer.
/// Actions should not mutate directly - mutations handled by execute_active_actions system.
#[deprecated(note = "Use Commands in system layer instead")]
fn clear_navigation_state(world: &World, entity: Entity) {
    // This function is now a no-op since actions can't mutate World.
    // Navigation state clearing will be handled by the system layer via Commands.
    // Keeping function signature for compatibility during refactor.
    let _ = (world, entity); // Suppress unused warnings
}

// =============================================================================
// DRINK WATER ACTION
// =============================================================================

/// Action: Drink water from a shallow water tile
///
/// Behavior:
/// 1. If not at water tile, path to it (multi-tick)
/// 2. Once at water, drink (instant)
/// 3. Reduces thirst significantly
///
/// Phase 3: Uses PathfindingQueue for async pathfinding
#[derive(Debug, Clone)]
pub struct DrinkWaterAction {
    pub target_tile: IVec2,
    state: DrinkWaterState,
    retry_count: u32,
    max_retries: u32,
    pub move_target: Option<IVec2>,
}

/// State machine for async drinking with PathfindingQueue
#[derive(Debug, Clone)]
enum DrinkWaterState {
    /// Need to request path to target
    NeedPath,
    /// Waiting for pathfinding result
    WaitingForPath {
        request_id: crate::pathfinding::PathRequestId,
    },
    /// Moving to target (MovementComponent handles actual movement)
    Moving,
    /// At water, drinking
    Drinking,
}

impl DrinkWaterAction {
    pub fn new(target_tile: IVec2) -> Self {
        Self {
            target_tile,
            state: DrinkWaterState::NeedPath,
            retry_count: 0,
            max_retries: 3,
            move_target: None,
        }
    }
}

impl Action for DrinkWaterAction {
    fn can_execute(&self, world: &World, entity: Entity) -> bool {
        // Check entity has thirst component
        if world.get::<Thirst>(entity).is_none() {
            return false;
        }

        // Check target tile is actually water
        if let Some(world_loader) = world.get_resource::<WorldLoader>() {
            if let Some(terrain_str) =
                world_loader.get_terrain_at(self.target_tile.x, self.target_tile.y)
            {
                if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                    matches!(terrain, TerrainType::ShallowWater)
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    fn execute(&mut self, world: &World, entity: Entity) -> ActionResult {
        // Get current tick from SimulationTick resource
        let tick = world.get_resource::<SimulationTick>()
            .map(|t| t.0)
            .unwrap_or(0);

        // Get entity position
        let Some(position) = world.get::<TilePosition>(entity).copied() else {
            warn!("Entity {:?} has no position, cannot drink", entity);
            return ActionResult::Failed;
        };

        let current_pos = position.tile;

        // Compute move target once if not set
        if self.move_target.is_none() {
            if let Some(world_loader) = world.get_resource::<WorldLoader>() {
                self.move_target = find_adjacent_walkable_tile(self.target_tile, world_loader)
                    .or_else(|| Some(self.target_tile));
            } else {
                return ActionResult::Failed;
            }
        }

        let move_target = self.move_target.unwrap_or(self.target_tile);

        // Check if we're adjacent to the water tile (or standing in it)
        let distance = (current_pos - self.target_tile).abs();
        let is_adjacent = distance.x <= 1 && distance.y <= 1 && (distance.x + distance.y) > 0;
        let is_on_water = current_pos == self.target_tile;

        if is_adjacent || is_on_water {
            // Transition to drinking state
            self.state = DrinkWaterState::Drinking;
        }

        // State machine for async pathfinding
        match &self.state {
            DrinkWaterState::NeedPath => {
                // NOTE: Cannot queue pathfinding request with read-only World
                // This mutation will be handled by the system layer
                // For now, mark as in progress - system will detect NeedPath state
                warn!("DrinkWater: NeedPath state requires system layer to queue pathfinding");
                ActionResult::InProgress
            }

            DrinkWaterState::WaitingForPath { request_id: _ } => {
                // Check for PathReady component (Phase 2: Component-based pathfinding)
                let entity_ref = world.get_entity(entity).ok();

                // Check if path is ready
                if let Some(entity_ref) = entity_ref {
                    if entity_ref.contains::<crate::pathfinding::PathReady>() {
                        // Path ready! System layer will insert MovementComponent
                        self.state = DrinkWaterState::Moving;
                        return ActionResult::InProgress;
                    }

                    // Check if path failed
                    if entity_ref.contains::<crate::pathfinding::PathFailed>() {
                        // Pathfinding failed, retry if under max retries
                        if self.retry_count < self.max_retries {
                            self.retry_count += 1;
                            self.state = DrinkWaterState::NeedPath;
                            debug!(
                                "DrinkWater path failed for entity {:?}, retry {}/{}",
                                entity, self.retry_count, self.max_retries
                            );
                            return ActionResult::InProgress;
                        } else {
                            debug!(
                                "DrinkWater gave up for entity {:?} after {} retries",
                                entity, self.max_retries
                            );
                            return ActionResult::Failed;
                        }
                    }
                }

                // Still waiting for path (no PathReady or PathFailed component yet)
                ActionResult::InProgress
            }

            DrinkWaterState::Moving => {
                // Check if movement is complete via MovementComponent
                if let Ok(entity_ref) = world.get_entity(entity) {
                    if let Some(movement) = entity_ref.get::<crate::entities::MovementComponent>() {
                        if movement.is_idle() {
                            // Movement complete, transition to drinking
                            self.state = DrinkWaterState::Drinking;
                        }
                    }
                }

                // Continue moving (execute_movement_component system handles actual movement)
                ActionResult::InProgress
            }

            DrinkWaterState::Drinking => {
                // We're close enough to drink!
                // NOTE: Cannot mutate Thirst with read-only World
                // System layer will handle the actual drinking mutation
                // Return Success to signal action complete
                let entity_ref = world.get_entity(entity).ok();
                if let Some(entity_ref) = entity_ref {
                    let amount = entity_ref
                        .get::<crate::entities::types::SpeciesNeeds>()
                        .map(|needs| needs.drink_amount)
                        .unwrap_or(50.0);

                    if entity_ref.contains::<Thirst>() {
                        info!(
                            "💧 Entity {:?} drinking water from {:?} at {:?} on tick {} (amount: {:.1})",
                            entity,
                            self.target_tile,
                            current_pos,
                            tick,
                            amount
                        );

                        // Return Success - system layer will apply thirst reduction
                        return ActionResult::Success;
                    }
                }

                ActionResult::Failed
            }
        }
    }

    fn cancel(&mut self, world: &World, entity: Entity) {
        clear_navigation_state(world, entity);
        self.state = DrinkWaterState::NeedPath;
        self.retry_count = 0;
        self.move_target = None;
        debug!(
            "🚫 DrinkWater action cancelled for entity {:?}, clearing navigation state",
            entity
        );
    }

    fn name(&self) -> &'static str {
        "DrinkWater"
    }
}

// =============================================================================
// GRAZE ACTION
// =============================================================================

/// Action: Move to a grass tile (for grazing/eating)
///
/// Behavior:
/// - Moves to target grass tile
/// - Once there, auto-eat system will trigger eating
/// - Used when hungry
///
/// Phase 3: Uses PathfindingQueue for async pathfinding
#[derive(Debug, Clone)]
pub struct GrazeAction {
    pub target_tile: IVec2,
    state: GrazeState,
    retry_count: u32,
    max_retries: u32,
    /// Initial biomass at the grazing location
    /// Used to determine when to give up on a patch
    initial_biomass: Option<f32>,
    /// Number of feeding attempts made at this location
    feeding_attempts: u32,
    /// Duration of grazing action in ticks
    /// Adjusted based on biomass availability
    duration_ticks: u32,
    /// Ticks elapsed during grazing
    ticks_elapsed: u32,
}

/// State machine for async grazing with PathfindingQueue
#[derive(Debug, Clone)]
enum GrazeState {
    /// Need to request path to target
    NeedPath,
    /// Waiting for pathfinding result
    WaitingForPath {
        request_id: crate::pathfinding::PathRequestId,
    },
    /// Moving to target (MovementComponent handles actual movement)
    Moving,
    /// At grass, grazing
    Grazing,
}

impl GrazeAction {
    pub fn new(target_tile: IVec2) -> Self {
        Self {
            target_tile,
            state: GrazeState::NeedPath,
            retry_count: 0,
            max_retries: 3,
            initial_biomass: None,
            feeding_attempts: 0,
            duration_ticks: 0, // Will be calculated when we arrive at the tile
            ticks_elapsed: 0,
        }
    }

    /// Calculate grazing duration based on biomass availability
    /// Higher biomass = longer feeding time, lower biomass = shorter feeding time
    fn calculate_duration(biomass: f32) -> u32 {
        // Base duration: 10 ticks (1 second at 10 TPS)
        let base_duration = 10;

        // Duration multiplier based on biomass quality (0.1 to 2.0)
        // Biomass range: 0.0 - 100.0
        // Good biomass (50+) = longer feeding
        // Poor biomass (<20) = quick feeding
        let duration_multiplier = if biomass >= 50.0 {
            2.0 // High quality: graze longer
        } else if biomass >= 20.0 {
            1.0 // Medium quality: normal duration
        } else {
            0.5 // Low quality: quick feeding, move to next patch
        };

        (base_duration as f32 * duration_multiplier) as u32
    }
}

impl Action for GrazeAction {
    fn can_execute(&self, world: &World, entity: Entity) -> bool {
        // Check entity has position
        if world.get::<TilePosition>(entity).is_none() {
            return false;
        }

        // Check target tile is walkable
        if let Some(world_loader) = world.get_resource::<WorldLoader>() {
            if let Some(terrain_str) =
                world_loader.get_terrain_at(self.target_tile.x, self.target_tile.y)
            {
                if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                    terrain.is_walkable()
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    fn execute(&mut self, world: &World, entity: Entity) -> ActionResult {
        // Get entity position
        let Some(position) = world.get::<TilePosition>(entity).copied() else {
            warn!("Entity {:?} has no position, cannot graze", entity);
            return ActionResult::Failed;
        };

        let current_pos = position.tile;

        // Check if we've arrived at target - transition to Grazing state
        if current_pos == self.target_tile && !matches!(self.state, GrazeState::Grazing) {
            self.state = GrazeState::Grazing;
        }

        // State machine for async pathfinding
        match &self.state {
            GrazeState::NeedPath => {
                // Signal system layer to queue pathfinding
                ActionResult::NeedsPathfinding { target: self.target_tile }
            }

            GrazeState::WaitingForPath { request_id: _ } => {
                // Check for PathReady component (Phase 2: Component-based pathfinding)
                let entity_ref = world.get_entity(entity).ok();

                // Check if path is ready
                if let Some(entity_ref) = entity_ref {
                    if entity_ref.contains::<crate::pathfinding::PathReady>() {
                        // Path ready! System layer will insert MovementComponent
                        self.state = GrazeState::Moving;
                        return ActionResult::InProgress;
                    }

                    // Check if path failed
                    if entity_ref.contains::<crate::pathfinding::PathFailed>() {
                        // Pathfinding failed, retry if under max retries
                        if self.retry_count < self.max_retries {
                            self.retry_count += 1;
                            self.state = GrazeState::NeedPath;
                            debug!(
                                "Graze path failed for entity {:?}, retry {}/{}",
                                entity, self.retry_count, self.max_retries
                            );
                            return ActionResult::InProgress;
                        } else {
                            debug!(
                                "Graze gave up for entity {:?} after {} retries",
                                entity, self.max_retries
                            );
                            return ActionResult::Failed;
                        }
                    }
                }

                // Still waiting for path (no PathReady or PathFailed component yet)
                ActionResult::InProgress
            }

            GrazeState::Moving => {
                // Check if movement is complete via MovementComponent
                if let Ok(entity_ref) = world.get_entity(entity) {
                    if let Some(movement) = entity_ref.get::<crate::entities::MovementComponent>() {
                        if movement.is_idle() {
                            // Movement complete, transition to grazing
                            self.state = GrazeState::Grazing;
                        }
                    }
                }

                // Continue moving (execute_movement_component system handles actual movement)
                ActionResult::InProgress
            }

            GrazeState::Grazing => {
                // We've arrived at target - now graze
                // Record initial biomass on first visit and calculate duration
                if self.initial_biomass.is_none() {
                    if let Some(resource_grid) =
                        world.get_resource::<crate::vegetation::resource_grid::ResourceGrid>()
                    {
                        if let Some(cell) = resource_grid.get_cell(self.target_tile) {
                            self.initial_biomass = Some(cell.total_biomass);
                            self.duration_ticks = Self::calculate_duration(cell.total_biomass);
                        }
                    }
                }

                // Check if we should continue grazing
                if self.ticks_elapsed < self.duration_ticks {
                    self.ticks_elapsed += 1;

                    // Still have time to graze, check biomass every 2 ticks
                    if self.ticks_elapsed % 2 == 0 {
                        // Check giving-up conditions (read-only)
                        let should_give_up = if let Some(initial_biomass) = self.initial_biomass {
                            if let Some(resource_grid) =
                                world.get_resource::<crate::vegetation::resource_grid::ResourceGrid>()
                            {
                                if let Some(current_cell) = resource_grid.get_cell(self.target_tile) {
                                    const GIVING_UP_THRESHOLD: f32 = 5.0;
                                    const GIVING_UP_THRESHOLD_RATIO: f32 = 0.2;
                                    let giving_up_absolute = GIVING_UP_THRESHOLD;
                                    let giving_up_ratio = initial_biomass * GIVING_UP_THRESHOLD_RATIO;
                                    let giving_up_threshold = giving_up_absolute.max(giving_up_ratio);

                                    if current_cell.total_biomass < giving_up_threshold {
                                        info!(
                                            "🌾 Entity {:?} giving up early - biomass {:.1} < threshold {:.1}",
                                            entity, current_cell.total_biomass, giving_up_threshold
                                        );
                                        true
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if should_give_up {
                            debug!("🌾 Entity {:?} giving up grazing early", entity);
                            return ActionResult::Success;
                        }

                        // NOTE: Actual biomass consumption and hunger reduction
                        // will be handled by the system layer via Commands
                        debug!(
                            "🐇 Entity {:?} grazing tick {}/{}",
                            entity, self.ticks_elapsed, self.duration_ticks
                        );
                    }

                    // Continue grazing
                    return ActionResult::InProgress;
                }

                // Grazing duration completed successfully
                debug!(
                    "✅ Entity {:?} completed grazing at {:?} after {} ticks",
                    entity, self.target_tile, self.ticks_elapsed
                );
                ActionResult::Success
            }
        }
    }

    fn name(&self) -> &'static str {
        "Graze"
    }

    fn cancel(&mut self, world: &World, entity: Entity) {
        clear_navigation_state(world, entity);
        self.state = GrazeState::NeedPath;
        self.retry_count = 0;
        self.initial_biomass = None;
        self.feeding_attempts = 0;
        self.duration_ticks = 0;
        self.ticks_elapsed = 0;
        debug!(
            "🚫 Graze action cancelled for entity {:?}, clearing grazing state",
            entity
        );
    }
}

// =============================================================================
// REST ACTION
// =============================================================================

/// Action: Rest in place to regenerate energy
#[derive(Debug, Clone)]
pub struct RestAction {
    pub duration_ticks: u32,
    pub ticks_remaining: u32,
    pub started: bool,
}

impl RestAction {
    pub fn new(duration_ticks: u32) -> Self {
        Self {
            duration_ticks,
            ticks_remaining: duration_ticks,
            started: false,
        }
    }
}

impl Action for RestAction {
    fn can_execute(&self, world: &World, entity: Entity) -> bool {
        world.get::<Energy>(entity).is_some()
    }

    fn execute(&mut self, world: &World, entity: Entity) -> ActionResult {
        let tick = world.get_resource::<SimulationTick>()
            .map(|t| t.0)
            .unwrap_or(0);

        if !self.started {
            if let Some(entity_ref) = world.get_entity(entity).ok() {
                if let Some(energy) = entity_ref.get::<Energy>() {
                    info!(
                        "😴 Entity {:?} started resting for {} ticks (energy: {:.1}%)",
                        entity,
                        self.duration_ticks,
                        energy.0.percentage()
                    );
                }
            }
            self.started = true;
            // NOTE: Energy state changes (set_resting/set_active) will be handled by system layer
        }

        self.ticks_remaining = self.ticks_remaining.saturating_sub(1);

        let energy_full = if let Some(entity_ref) = world.get_entity(entity).ok() {
            if let Some(energy) = entity_ref.get::<Energy>() {
                energy.0.is_full()
            } else {
                false
            }
        } else {
            false
        };

        if self.ticks_remaining == 0 || energy_full {
            if let Some(entity_ref) = world.get_entity(entity).ok() {
                if let Some(energy) = entity_ref.get::<Energy>() {
                    info!(
                        "😊 Entity {:?} finished resting on tick {}! Energy: {:.1}%",
                        entity,
                        tick,
                        energy.0.percentage()
                    );
                }
            }
            // NOTE: Energy state changes will be handled by system layer
            return ActionResult::Success;
        }

        ActionResult::InProgress
    }

    fn cancel(&mut self, world: &World, entity: Entity) {
        // NOTE: Energy state changes will be handled by system layer
        debug!(
            "🚫 Entity {:?} resting interrupted, system will reset energy to active",
            entity
        );
    }

    fn name(&self) -> &'static str {
        "Rest"
    }
}

// =============================================================================
// SCAVENGE ACTION
// =============================================================================

/// Action: Move to a carcass and consume available nutrition.
#[derive(Debug, Clone)]
pub struct ScavengeAction {
    pub carcass: Entity,
    pub started: bool,
}

impl ScavengeAction {
    pub fn new(carcass: Entity) -> Self {
        Self {
            carcass,
            started: false,
        }
    }
}

impl Action for ScavengeAction {
    fn can_execute(&self, world: &World, entity: Entity) -> bool {
        world.get::<Hunger>(entity).is_some() && world.get::<Carcass>(self.carcass).is_some()
    }

    fn execute(&mut self, world: &World, entity: Entity) -> ActionResult {
        let Some(position) = world.get::<TilePosition>(entity).copied() else {
            return ActionResult::Failed;
        };

        let Some(carcass_pos) = world.get::<TilePosition>(self.carcass).copied() else {
            debug!("🦴 Scavenge target vanished before arrival");
            return ActionResult::Failed;
        };

        if position.tile != carcass_pos.tile {
            // NOTE: MoveOrder insertion will be handled by system layer
            self.started = true;
            return ActionResult::InProgress;
        }

        clear_navigation_state(world, entity);

        let bite_size = world
            .get::<SpeciesNeeds>(entity)
            .map(|n| n.eat_amount)
            .unwrap_or(50.0);

        // NOTE: Carcass consumption, hunger changes, and despawning
        // will be handled by system layer via Commands
        info!(
            "🦴 Entity {:?} scavenging from carcass {:?} (bite size: {:.1})",
            entity, self.carcass, bite_size
        );

        self.started = false;
        ActionResult::Success
    }

    fn cancel(&mut self, world: &World, entity: Entity) {
        clear_navigation_state(world, entity);
        self.started = false;
    }

    fn name(&self) -> &'static str {
        "Scavenge"
    }
}

// =============================================================================
// HUNT ACTION
// =============================================================================

/// Action: Pursue prey and attempt a kill.
///
/// Phase 3: Uses PathfindingQueue for async pathfinding
#[derive(Debug, Clone)]
pub struct HuntAction {
    pub prey: Entity,
    state: HuntState,
    retry_count: u32,
    max_retries: u32,
    last_prey_pos: Option<IVec2>,
}

/// State machine for async hunting with PathfindingQueue
#[derive(Debug, Clone)]
enum HuntState {
    /// Need to request path to prey
    NeedPath,
    /// Waiting for pathfinding result
    WaitingForPath {
        request_id: crate::pathfinding::PathRequestId,
        target_pos: IVec2,
    },
    /// Moving to target (MovementComponent handles actual movement)
    Moving {
        target_pos: IVec2,
    },
    /// Close enough to attack
    Attacking,
}

impl HuntAction {
    pub fn new(prey: Entity) -> Self {
        Self {
            prey,
            state: HuntState::NeedPath,
            retry_count: 0,
            max_retries: 3,
            last_prey_pos: None,
        }
    }
}

impl Action for HuntAction {
    fn can_execute(&self, world: &World, entity: Entity) -> bool {
        world.get::<Hunger>(entity).is_some() && world.get::<TilePosition>(self.prey).is_some()
    }

    fn execute(&mut self, world: &World, entity: Entity) -> ActionResult {
        let Some(predator_pos) = world.get::<TilePosition>(entity).copied() else {
            return ActionResult::Failed;
        };

        let Some(prey_pos) = world.get::<TilePosition>(self.prey).copied() else {
            debug!("🎯 Prey {:?} lost before hunt completed", self.prey);
            return ActionResult::Failed;
        };

        let diff = predator_pos.tile - prey_pos.tile;
        let distance = diff.x.abs().max(diff.y.abs()) as f32;

        // Check if close enough to attack
        if distance <= 1.5 {
            self.state = HuntState::Attacking;
        }

        // If prey moved significantly, request new path
        if let Some(last_pos) = self.last_prey_pos {
            if (last_pos - prey_pos.tile).abs().max_element() > 3 {
                // Prey moved significantly, need new path
                self.state = HuntState::NeedPath;
            }
        }
        self.last_prey_pos = Some(prey_pos.tile);

        // State machine for async pathfinding
        match &self.state {
            HuntState::NeedPath => {
                // NOTE: Pathfinding queue mutation handled by system layer
                warn!("Hunt: NeedPath state requires system layer to queue pathfinding");
                ActionResult::InProgress
            }

            HuntState::WaitingForPath { request_id: _, target_pos } => {
                // Check for PathReady component (Phase 2: Component-based pathfinding)
                let entity_ref = world.get_entity(entity).ok();

                // Check if path is ready
                if let Some(entity_ref) = entity_ref {
                    if entity_ref.contains::<crate::pathfinding::PathReady>() {
                        // Path ready! System layer will insert MovementComponent
                        self.state = HuntState::Moving {
                            target_pos: *target_pos,
                        };
                        return ActionResult::InProgress;
                    }

                    // Check if path failed
                    if entity_ref.contains::<crate::pathfinding::PathFailed>() {
                        // Pathfinding failed, retry if under max retries
                        if self.retry_count < self.max_retries {
                            self.retry_count += 1;
                            self.state = HuntState::NeedPath;
                            debug!(
                                "Hunt path failed for entity {:?}, retry {}/{}",
                                entity, self.retry_count, self.max_retries
                            );
                            return ActionResult::InProgress;
                        } else {
                            debug!(
                                "Hunt gave up for entity {:?} after {} retries",
                                entity, self.max_retries
                            );
                            return ActionResult::Failed;
                        }
                    }
                }

                // Still waiting for path (no PathReady or PathFailed component yet)
                ActionResult::InProgress
            }

            HuntState::Moving { target_pos: _ } => {
                // Movement is handled by execute_movement_component system
                // Just continue progress - attack transition is handled by distance check above
                ActionResult::InProgress
            }

            HuntState::Attacking => {
                // We're close enough to attack!
                clear_navigation_state(world, entity);

                let bite_size = world
                    .get::<SpeciesNeeds>(entity)
                    .map(|n| n.eat_amount)
                    .unwrap_or(60.0);
                let available_meat = world
                    .get::<SpeciesNeeds>(self.prey)
                    .map(|n| n.eat_amount * 3.0)
                    .unwrap_or(80.0);

                // Allow predators to fully consume small prey (e.g., rabbits) while
                // still leaving carcasses for large kills.
                let consumed = if available_meat <= bite_size * 2.0 {
                    available_meat
                } else {
                    bite_size
                };

                // NOTE: Hunger changes, prey despawning, and carcass spawning
                // will be handled by system layer via Commands
                info!(
                    "🐺 Entity {:?} hunted prey {:?}, will consume {:.1} nutrition",
                    entity, self.prey, consumed
                );

                ActionResult::Success
            }
        }
    }

    fn cancel(&mut self, world: &World, entity: Entity) {
        clear_navigation_state(world, entity);
        self.state = HuntState::NeedPath;
        self.retry_count = 0;
        self.last_prey_pos = None;
    }

    fn name(&self) -> &'static str {
        "Hunt"
    }
}

// =============================================================================
// FOLLOW ACTION
// =============================================================================

/// Action: Follow a target entity until within a certain distance
#[derive(Debug, Clone)]
pub struct FollowAction {
    pub target: Entity,
    pub stop_distance: i32,
    pub started: bool,
}

impl FollowAction {
    pub fn new(target: Entity, stop_distance: i32) -> Self {
        Self {
            target,
            stop_distance,
            started: false,
        }
    }
}

impl Action for FollowAction {
    fn can_execute(&self, world: &World, _entity: Entity) -> bool {
        // Target must still exist and have a position
        world.get_entity(self.target).is_ok() && world.get::<TilePosition>(self.target).is_some()
    }

    fn execute(&mut self, world: &World, entity: Entity) -> ActionResult {
        // Abort on pathfinding failure for this entity
        if world.get::<PathfindingFailed>(entity).is_some() {
            warn!(
                "Entity {:?} pathfinding failed while following, aborting Follow action",
                entity
            );
            // NOTE: PathfindingFailed removal handled by system layer
            return ActionResult::Failed;
        }

        let Some(follower_pos) = world.get::<TilePosition>(entity).copied() else {
            return ActionResult::Failed;
        };
        let Some(target_pos) = world.get::<TilePosition>(self.target).copied() else {
            return ActionResult::Failed;
        };

        // Check distance
        let d = {
            let diff = (follower_pos.tile - target_pos.tile).abs();
            diff.x.max(diff.y)
        };

        if d <= self.stop_distance {
            return ActionResult::Success;
        }

        // If not currently moving (no Path), system layer will issue move order
        let is_moving = world.get::<Path>(entity).is_some();
        if !is_moving {
            self.started = true;
            // NOTE: MoveOrder insertion handled by system layer
        }

        ActionResult::InProgress
    }

    fn cancel(&mut self, world: &World, entity: Entity) {
        clear_navigation_state(world, entity);
        self.started = false;
        debug!(
            "🚫 Follow action cancelled for entity {:?}, stopping movement",
            entity
        );
    }

    fn name(&self) -> &'static str {
        "Follow"
    }
}

// =============================================================================
// =============================================================================
// MATE ACTION
// =============================================================================

/// Action: Rendezvous with partner and mate; pregnancy applied on female only
#[derive(Debug, Clone)]
pub struct MateAction {
    pub partner: Entity,
    pub meeting_tile: IVec2,
    pub duration_ticks: u32,
    pub started: bool,
    pub waited: u32,
    pub total_wait: u32,
    pub max_wait_ticks: u32,
}

impl MateAction {
    pub fn new(partner: Entity, meeting_tile: IVec2, duration_ticks: u32) -> Self {
        Self {
            partner,
            meeting_tile,
            duration_ticks,
            started: false,
            waited: 0,
            total_wait: 0,
            max_wait_ticks: duration_ticks.saturating_mul(5).max(duration_ticks + 25),
        }
    }
}

impl Action for MateAction {
    fn can_execute(&self, world: &World, _entity: Entity) -> bool {
        world.get_entity(self.partner).is_ok() && world.get::<TilePosition>(self.partner).is_some()
    }

    fn execute(&mut self, world: &World, entity: Entity) -> ActionResult {
        use crate::entities::reproduction::{Pregnancy, ReproductionConfig, ReproductionCooldown, Sex};

        // NOTE: This action has been simplified to read-only World access.
        // All component mutations (removing ActiveMate/MatingTarget, PathfindingFailed, inserting Pregnancy, etc.)
        // will be handled by the system layer based on the returned ActionResult.

        // Abort if either entity failed to find a path
        if world.get::<PathfindingFailed>(entity).is_some()
            || world.get::<PathfindingFailed>(self.partner).is_some()
        {
            warn!(
                "⚠️ MateAction: pathfinding failed for {:?} or {:?}, aborting",
                entity, self.partner
            );
            // NOTE: Component removal handled by system layer
            return ActionResult::Failed;
        }

        // Abort if partner missing
        if world.get::<TilePosition>(self.partner).is_none() {
            warn!("⚠️ MateAction: partner {:?} missing", self.partner);
            // NOTE: Component removal handled by system layer
            return ActionResult::Failed;
        }

        let Some(me_pos) = world.get::<TilePosition>(entity).copied() else {
            return ActionResult::Failed;
        };
        let Some(partner_pos) = world.get::<TilePosition>(self.partner).copied() else {
            return ActionResult::Failed;
        };

        // Move towards meeting tile until arrived
        if me_pos.tile != self.meeting_tile {
            // NOTE: MoveOrder insertion handled by system layer
            return ActionResult::InProgress;
        }

        // We are on the meeting tile
        clear_navigation_state(world, entity);
        self.started = true;

        // Track total waiting time once we've reached the rendezvous point
        self.total_wait = self.total_wait.saturating_add(1);
        if self.total_wait > self.max_wait_ticks {
            debug!(
                "⚠️ MateAction: entity {:?} waited {} ticks for partner {:?} without success",
                entity, self.total_wait, self.partner
            );
            return ActionResult::TriggerFollowUp;
        }

        // At meeting tile: ensure partner arrives on same or adjacent tile
        let diff = (self.meeting_tile - partner_pos.tile).abs();
        let partner_adjacent = diff.x.max(diff.y) <= 1;

        if !partner_adjacent {
            // Partner is still approaching
            // NOTE: MoveOrder insertion for partner handled by system layer
            self.waited = self.waited.saturating_sub(1);

            debug!(
                "💕 MateAction: Entity {:?} waiting for partner {:?} - not adjacent. Me: {:?}, Partner: {:?}, Meeting: {:?}",
                entity, self.partner, me_pos.tile, partner_pos.tile, self.meeting_tile
            );

            return ActionResult::InProgress;
        }

        // Partner is adjacent—stop them from wandering off while waiting
        clear_navigation_state(world, self.partner);

        // Both are within touching distance: perform mating over duration
        self.waited = self.waited.saturating_add(1);

        // Debug logging for mating progress
        if self.waited <= 1 || self.waited % 10 == 0 || self.waited >= self.duration_ticks {
            info!(
                "💕 MateAction: Entity {:?} mating progress: {}/{} ticks, partner {:?} at meeting tile {:?}",
                entity, self.waited, self.duration_ticks, self.partner, self.meeting_tile
            );
        }

        if self.waited < self.duration_ticks {
            return ActionResult::InProgress;
        }

        // Duration complete: mating successful!
        // NOTE: System layer will handle:
        // - Pregnancy/ReproductionCooldown insertion based on Sex
        // - ActiveMate/MatingTarget relationship cleanup
        // - Navigation state clearing
        let me_female = world
            .get::<Sex>(entity)
            .is_some_and(|s| matches!(s, Sex::Female));
        let partner_female = world
            .get::<Sex>(self.partner)
            .is_some_and(|s| matches!(s, Sex::Female));

        info!(
            "❤️ Mating complete for entity {:?} with partner {:?} (me_female: {}, partner_female: {})",
            entity, self.partner, me_female, partner_female
        );

        // Clear navigation state (read-only operation)
        clear_navigation_state(world, entity);
        clear_navigation_state(world, self.partner);

        ActionResult::Success
    }

    fn cancel(&mut self, world: &World, entity: Entity) {
        // NOTE: System layer will handle ActiveMate/MatingTarget cleanup
        debug!(
            "🚫 Entity {:?} mating interrupted, system will clean up mating relationship",
            entity
        );

        clear_navigation_state(world, entity);
        clear_navigation_state(world, self.partner);
    }

    fn name(&self) -> &'static str {
        "Mate"
    }
}

/// Harvest Action - Collect harvestable resources like mushrooms, roots, etc.
pub struct HarvestAction {
    target_tile: IVec2,
    resource_type: ResourceType,
    completed: bool,
}

impl HarvestAction {
    pub fn new(target_tile: IVec2, resource_type: ResourceType) -> Self {
        Self {
            target_tile,
            resource_type,
            completed: false,
        }
    }
}

impl Action for HarvestAction {
    fn can_execute(&self, world: &World, entity: Entity) -> bool {
        // Check if entity is at the target tile
        if let Some(position) = world.get::<TilePosition>(entity) {
            if position.tile != self.target_tile {
                return false;
            }
        } else {
            return false;
        }

        // Check if the resource is still available and can be harvested
        if let Some(world_loader) = world.get_resource::<crate::world_loader::WorldLoader>() {
            // Check if resource type matches what we expect to harvest
            if let Some(resource_at_tile) = world_loader.get_resource_at(self.target_tile.x, self.target_tile.y) {
                if let Some(actual_resource) = ResourceType::from_str(&resource_at_tile) {
                    return actual_resource == self.resource_type && actual_resource.is_gatherable();
                }
            }
        }

        false
    }

    fn execute(&mut self, world: &World, entity: Entity) -> ActionResult {
        let tick = world.get_resource::<SimulationTick>()
            .map(|t| t.0)
            .unwrap_or(0);

        if self.completed {
            return ActionResult::Success;
        }

        // Check if entity is at the correct position
        let position = match world.get::<TilePosition>(entity) {
            Some(pos) => pos,
            None => return ActionResult::Failed,
        };

        if position.tile != self.target_tile {
            return ActionResult::Failed;
        }

        // NOTE: Harvest operations (resource_grid mutations) will be handled by system layer
        // For now, just verify the harvest is valid and return Success
        let harvest_valid = if let Some(world_loader) = world.get_resource::<crate::world_loader::WorldLoader>() {
            if let Some(resource_at_tile) = world_loader.get_resource_at(self.target_tile.x, self.target_tile.y) {
                if let Some(actual_resource) = ResourceType::from_str(&resource_at_tile) {
                    if actual_resource == self.resource_type && actual_resource.is_gatherable() {
                        if let Some(harvest_profile) = actual_resource.get_harvest_profile() {
                            if let Some(resource_grid) = world.get_resource::<crate::vegetation::resource_grid::ResourceGrid>() {
                                if let Some(cell) = resource_grid.get_cell(self.target_tile) {
                                    // Check if ready for harvest
                                    tick >= cell.regrowth_available_tick
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if harvest_valid {
            info!(
                "🧺 Entity {:?} harvesting {} at tile {:?}",
                entity, self.resource_type.as_str(), self.target_tile
            );
            self.completed = true;
            ActionResult::Success
        } else {
            ActionResult::Failed
        }
    }

    fn cancel(&mut self, _world: &World, _entity: Entity) {
        // No special cleanup needed for harvest actions (removed mutations)
        debug!("🚫 Harvest action cancelled for resource {} at tile {:?}", self.resource_type.as_str(), self.target_tile);
    }

    fn name(&self) -> &'static str {
        "Harvest"
    }
}

// =============================================================================
// WANDER ACTION
// =============================================================================

/// Wander action - idle exploration within territory
///
/// Behavior:
/// - Moves to a random walkable tile within wander_radius
/// - Lowest priority action (always available as fallback)
/// - Used when no needs are pressing
///
/// Phase 2: Uses PathfindingQueue for async pathfinding
#[derive(Debug, Clone)]
pub struct WanderAction {
    pub target_tile: IVec2,
    state: WanderState,
    retry_count: u32,
    max_retries: u32,
}

/// State machine for async wandering with PathfindingQueue
#[derive(Debug, Clone)]
enum WanderState {
    /// Need to request path to target
    NeedPath,
    /// Waiting for pathfinding result
    WaitingForPath {
        request_id: crate::pathfinding::PathRequestId,
    },
    /// Moving to target (MovementComponent handles actual movement)
    Moving,
}

impl WanderAction {
    pub fn new(target_tile: IVec2) -> Self {
        Self {
            target_tile,
            state: WanderState::NeedPath,
            retry_count: 0,
            max_retries: 3,
        }
    }
}

impl Action for WanderAction {
    fn can_execute(&self, world: &World, entity: Entity) -> bool {
        // Check entity has position
        if world.get::<TilePosition>(entity).is_none() {
            return false;
        }

        // Check target tile is walkable
        if let Some(world_loader) = world.get_resource::<WorldLoader>() {
            if let Some(terrain_str) =
                world_loader.get_terrain_at(self.target_tile.x, self.target_tile.y)
            {
                if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                    terrain.is_walkable()
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    fn execute(&mut self, world: &World, entity: Entity) -> ActionResult {
        // Get current position
        let Some(pos) = world.get::<TilePosition>(entity) else {
            return ActionResult::Failed;
        };
        let current_pos = pos.tile;

        // Check if arrived at target
        if current_pos == self.target_tile {
            return ActionResult::Success;
        }

        // State machine for async pathfinding
        match &self.state {
            WanderState::NeedPath => {
                // NOTE: Pathfinding queue mutation handled by system layer
                warn!("Wander: NeedPath state requires system layer to queue pathfinding");
                ActionResult::InProgress
            }

            WanderState::WaitingForPath { request_id: _ } => {
                // Check for PathReady component (Phase 2: Component-based pathfinding)
                let entity_ref = world.get_entity(entity).ok();

                // Check if path is ready
                if let Some(entity_ref) = entity_ref {
                    if entity_ref.contains::<crate::pathfinding::PathReady>() {
                        // Path ready! System layer will insert MovementComponent
                        self.state = WanderState::Moving;
                        return ActionResult::InProgress;
                    }

                    // Check if path failed
                    if entity_ref.contains::<crate::pathfinding::PathFailed>() {
                        // Pathfinding failed, retry with new target if under max retries
                        if self.retry_count < self.max_retries {
                            self.retry_count += 1;
                            self.state = WanderState::NeedPath;
                            debug!(
                                "Wander path failed for entity {:?}, retry {}/{}",
                                entity, self.retry_count, self.max_retries
                            );
                            return ActionResult::InProgress;
                        } else {
                            debug!(
                                "Wander gave up for entity {:?} after {} retries",
                                entity, self.max_retries
                            );
                            return ActionResult::Failed;
                        }
                    }
                }

                // Still waiting for path (no PathReady or PathFailed component yet)
                ActionResult::InProgress
            }

            WanderState::Moving => {
                // Check if movement is complete via MovementComponent
                if let Ok(entity_ref) = world.get_entity(entity) {
                    if let Some(movement) = entity_ref.get::<crate::entities::MovementComponent>() {
                        if movement.is_idle() {
                            // Movement complete!
                            return ActionResult::Success;
                        }
                    }
                }

                // Continue moving (execute_movement_component system handles actual movement)
                ActionResult::InProgress
            }
        }
    }

    fn cancel(&mut self, world: &World, entity: Entity) {
        // NOTE: MovementComponent insertion handled by system layer
        let _ = (world, entity); // Suppress unused warnings
        // Reset state machine
        self.state = WanderState::NeedPath;
        self.retry_count = 0;
    }

    fn name(&self) -> &'static str {
        "Wander"
    }
}

// =============================================================================
// ACTION FACTORY
// =============================================================================

/// Create an action from an ActionType
pub fn create_action(action_type: ActionType) -> Box<dyn Action> {
    match action_type {
        ActionType::DrinkWater { target_tile } => Box::new(DrinkWaterAction::new(target_tile)),
        ActionType::Graze { target_tile } => Box::new(GrazeAction::new(target_tile)),
        ActionType::Rest { duration_ticks } => Box::new(RestAction::new(duration_ticks)),
        ActionType::Scavenge { carcass } => Box::new(ScavengeAction::new(carcass)),
        ActionType::Hunt { prey } => Box::new(HuntAction::new(prey)),
        ActionType::Follow {
            target,
            stop_distance,
        } => Box::new(FollowAction::new(target, stop_distance)),
        ActionType::Mate {
            partner,
            meeting_tile,
            duration_ticks,
        } => Box::new(MateAction::new(partner, meeting_tile, duration_ticks)),
        ActionType::Harvest {
            target_tile,
            resource_type,
        } => Box::new(HarvestAction::new(target_tile, resource_type)),
        ActionType::Wander { target_tile } => Box::new(WanderAction::new(target_tile)),
    }
}
