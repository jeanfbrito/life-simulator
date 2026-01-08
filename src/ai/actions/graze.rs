use bevy::prelude::*;

use super::{Action, ActionResult};
use crate::entities::stats::Hunger;
use crate::entities::TilePosition;
use crate::pathfinding::PathRequestId;
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;

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
    WaitingForPath { request_id: PathRequestId },
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

    /// Transition from NeedPath to WaitingForPath state
    /// Called by pathfinding bridge system after queuing pathfinding request
    pub fn transition_to_waiting(&mut self, request_id: PathRequestId) {
        self.state = GrazeState::WaitingForPath { request_id };
    }

    /// Calculate grazing duration based on biomass availability
    /// Duration is a maximum - entity exits early when hunger satisfied (like Rest exits when energy full)
    /// Higher biomass = can graze longer before depleting, lower biomass = shorter max time
    fn calculate_duration(biomass: f32) -> u32 {
        // Base duration: 50 ticks (5 seconds at 10 TPS) - enough to satisfy hunger from ~70%
        // Entity exits early via hunger_satisfied check, so this is just the max
        let base_duration = 50;

        // Duration multiplier based on biomass quality
        // Biomass range: 0.0 - 100.0
        // Good biomass (50+) = graze up to full duration (patch can sustain it)
        // Poor biomass (<20) = shorter duration (patch depletes quickly)
        let duration_multiplier = if biomass >= 50.0 {
            1.5 // High quality: up to 75 ticks max
        } else if biomass >= 20.0 {
            1.0 // Medium quality: 50 ticks max
        } else {
            0.5 // Low quality: 25 ticks max, move on
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
            info!(
                "🎯 Entity {:?} arrived at grazing target {:?}",
                entity, self.target_tile
            );
        }

        // State machine for async pathfinding
        match &self.state {
            GrazeState::NeedPath => {
                // Signal system layer to queue pathfinding
                ActionResult::NeedsPathfinding {
                    target: self.target_tile,
                }
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
                            // Check if biomass is too low to graze
                            const MIN_BIOMASS_THRESHOLD: f32 = 1.0;
                            if cell.total_biomass < MIN_BIOMASS_THRESHOLD {
                                warn!(
                                    "🌾 Entity {:?} arrived at {:?} but biomass too low ({:.1} < {:.1}), grazing failed",
                                    entity,
                                    self.target_tile,
                                    cell.total_biomass,
                                    MIN_BIOMASS_THRESHOLD
                                );
                                return ActionResult::Failed;
                            }

                            self.initial_biomass = Some(cell.total_biomass);
                            self.duration_ticks = Self::calculate_duration(cell.total_biomass);
                            info!(
                                "🌾 Entity {:?} started grazing at {:?} (biomass: {:.1}, duration: {} ticks)",
                                entity,
                                self.target_tile,
                                cell.total_biomass,
                                self.duration_ticks
                            );
                        } else {
                            warn!(
                                "🌾 Entity {:?} arrived at grazing tile {:?} but found no biomass cell!",
                                entity,
                                self.target_tile
                            );
                        }
                    } else {
                        warn!(
                            "🌾 Entity {:?} arrived at grazing tile {:?} but ResourceGrid not available!",
                            entity,
                            self.target_tile
                        );
                    }
                }

                // Check if hunger is satisfied (like Rest checks energy.is_full())
                // grazing_hunger_system reduces hunger each tick while grazing
                let hunger_satisfied = if let Ok(entity_ref) = world.get_entity(entity) {
                    if let Some(hunger) = entity_ref.get::<Hunger>() {
                        hunger.0.is_empty() // is_empty = hunger at 0 = satisfied
                    } else {
                        false
                    }
                } else {
                    false
                };

                if hunger_satisfied {
                    info!(
                        "🌾 Entity {:?} finished grazing - hunger satisfied!",
                        entity
                    );
                    return ActionResult::Success;
                }

                // Check if we should continue grazing
                if self.ticks_elapsed < self.duration_ticks {
                    self.ticks_elapsed += 1;

                    // Log grazing progress periodically
                    if self.ticks_elapsed == 1
                        || self.ticks_elapsed % 5 == 0
                        || self.ticks_elapsed == self.duration_ticks
                    {
                        debug!(
                            "🐇 Entity {:?} grazing tick {}/{} at {:?}",
                            entity, self.ticks_elapsed, self.duration_ticks, self.target_tile
                        );
                    }

                    // Still have time to graze, check biomass every 2 ticks
                    if self.ticks_elapsed % 2 == 0 {
                        // Check giving-up conditions (read-only)
                        let should_give_up = if let Some(initial_biomass) = self.initial_biomass {
                            if let Some(resource_grid) = world
                                .get_resource::<crate::vegetation::resource_grid::ResourceGrid>()
                            {
                                if let Some(current_cell) = resource_grid.get_cell(self.target_tile)
                                {
                                    const GIVING_UP_THRESHOLD: f32 = 5.0;
                                    const GIVING_UP_THRESHOLD_RATIO: f32 = 0.2;
                                    let giving_up_absolute = GIVING_UP_THRESHOLD;
                                    let giving_up_ratio = initial_biomass * GIVING_UP_THRESHOLD_RATIO;
                                    let giving_up_threshold =
                                        giving_up_absolute.max(giving_up_ratio);

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

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn cancel(&mut self, world: &World, entity: Entity) {
        super::clear_navigation_state(world, entity);
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
