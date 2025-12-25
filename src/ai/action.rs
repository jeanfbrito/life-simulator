use crate::entities::stats::{Energy, Hunger, Thirst};
use crate::entities::{Carcass, Creature, MoveOrder, SpeciesNeeds, TilePosition};
use crate::pathfinding::{Path, PathRequest, PathfindingFailed};
use crate::resources::ResourceType;
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;
/// Action system for TQUAI
///
/// Actions are discrete behaviors that can be queued and executed on ticks.
/// They can be instant (complete in one tick) or multi-tick (span multiple ticks).
use bevy::prelude::*;
use rand::Rng;

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
}

/// Request to queue an action
#[derive(Debug, Clone)]
pub struct ActionRequest {
    pub entity: Entity,
    pub action_type: ActionType,
    pub utility: f32,
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
    fn can_execute(&self, world: &World, entity: Entity, tick: u64) -> bool;

    /// Execute the action for this tick
    /// Returns Success/Failed/InProgress
    fn execute(&mut self, world: &mut World, entity: Entity, tick: u64) -> ActionResult;

    /// Cancel the action (called when a higher priority action needs to interrupt)
    /// Default implementation does nothing - override for actions that need cleanup
    fn cancel(&mut self, world: &mut World, entity: Entity) {
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
fn clear_navigation_state(world: &mut World, entity: Entity) {
    if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
        entity_mut.remove::<MoveOrder>();
        entity_mut.remove::<PathRequest>();
        entity_mut.remove::<Path>();
        entity_mut.remove::<PathfindingFailed>();
    }
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
#[derive(Debug, Clone)]
pub struct DrinkWaterAction {
    pub target_tile: IVec2,
    pub started: bool,
    pub move_target: Option<IVec2>,
}

impl DrinkWaterAction {
    pub fn new(target_tile: IVec2) -> Self {
        Self {
            target_tile,
            started: false,
            move_target: None,
        }
    }
}

impl Action for DrinkWaterAction {
    fn can_execute(&self, world: &World, entity: Entity, _tick: u64) -> bool {
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

    fn execute(&mut self, world: &mut World, entity: Entity, tick: u64) -> ActionResult {
        // Get entity position
        let Some(position) = world.get::<TilePosition>(entity).copied() else {
            warn!("Entity {:?} has no position, cannot drink", entity);
            return ActionResult::Failed;
        };

        let current_pos = position.tile;

        // Check if pathfinding failed for this entity
        if world.get::<PathfindingFailed>(entity).is_some() {
            warn!(
                "Entity {:?} pathfinding failed to reach water at {:?}, aborting DrinkWater action",
                entity, self.target_tile
            );
            // Remove the PathfindingFailed component
            if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                entity_mut.remove::<PathfindingFailed>();
            }
            return ActionResult::Failed;
        }

        // Check if we're adjacent to the water tile (or standing in it)
        let distance = (current_pos - self.target_tile).abs();
        let is_adjacent = distance.x <= 1 && distance.y <= 1 && (distance.x + distance.y) > 0;
        let is_on_water = current_pos == self.target_tile;

        if is_adjacent || is_on_water {
            // We're close enough to drink!
            if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                // Compute drink amount before taking a mutable borrow on Thirst to avoid overlapping borrows
                let amount = entity_mut
                    .get::<crate::entities::types::SpeciesNeeds>()
                    .map(|needs| needs.drink_amount)
                    .unwrap_or(50.0);

                if let Some(mut thirst) = entity_mut.get_mut::<Thirst>() {
                    // Reduce thirst by species-specific amount instead of fully restoring
                    let old_thirst_units = thirst.0.current;
                    thirst.0.change(-amount);

                    info!(
                        "💧 Entity {:?} drank water from {:?} while at {:?} on tick {}! Thirst units: {:.1} -> {:.1} (Δ {:.1})",
                        entity,
                        self.target_tile,
                        current_pos,
                        tick,
                        old_thirst_units,
                        thirst.0.current,
                        amount.min(old_thirst_units)
                    );

                    return ActionResult::Success;
                }
            }

            return ActionResult::Failed;
        }

        // We need to move closer to the water
        let mut needs_new_target = self.move_target.is_none();

        if let Some(target) = self.move_target {
            // Ensure the cached move target is still valid
            if let Some(world_loader) = world.get_resource::<WorldLoader>() {
                let still_walkable = world_loader
                    .get_terrain_at(target.x, target.y)
                    .and_then(|terrain_str| TerrainType::from_str(&terrain_str))
                    .map(|terrain| terrain.is_walkable())
                    .unwrap_or(false);

                if !still_walkable {
                    needs_new_target = true;
                }
            } else {
                return ActionResult::Failed;
            }
        }

        if needs_new_target {
            if let Some(world_loader) = world.get_resource::<WorldLoader>() {
                self.move_target = find_adjacent_walkable_tile(self.target_tile, world_loader)
                    .or_else(|| {
                        // Fallback: allow standing in the shallow water tile itself
                        Some(self.target_tile)
                    });
            } else {
                return ActionResult::Failed;
            }

            if let Some(target) = self.move_target {
                info!(
                    "🐇 Entity {:?} heading to water at {:?} (move target {:?})",
                    entity, self.target_tile, target
                );

                if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                    entity_mut.insert(MoveOrder {
                        destination: target,
                        allow_diagonal: true,
                    });
                }

                self.started = true;
            } else {
                warn!(
                    "No adjacent or fallback tile found for water at {:?}",
                    self.target_tile
                );
                return ActionResult::Failed;
            }
        }

        // Still traveling
        ActionResult::InProgress
    }

    fn cancel(&mut self, world: &mut World, entity: Entity) {
        clear_navigation_state(world, entity);
        self.started = false;
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
#[derive(Debug, Clone)]
pub struct GrazeAction {
    pub target_tile: IVec2,
    pub started: bool,
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

impl GrazeAction {
    pub fn new(target_tile: IVec2) -> Self {
        Self {
            target_tile,
            started: false,
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
    fn can_execute(&self, world: &World, entity: Entity, _tick: u64) -> bool {
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

    fn execute(&mut self, world: &mut World, entity: Entity, tick: u64) -> ActionResult {
        // Get entity position
        let Some(position) = world.get::<TilePosition>(entity).copied() else {
            warn!("Entity {:?} has no position, cannot graze", entity);
            return ActionResult::Failed;
        };

        let current_pos = position.tile;

        // Check if pathfinding failed for this entity
        if world.get::<PathfindingFailed>(entity).is_some() {
            debug!(
                "Entity {:?} pathfinding failed to reach graze target {:?}, aborting Graze action",
                entity, self.target_tile
            );
            // Remove the PathfindingFailed component
            if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                entity_mut.remove::<PathfindingFailed>();
            }
            return ActionResult::Failed;
        }

        // Check if we've arrived at target
        if current_pos == self.target_tile {
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

                // Still have time to graze, consume some biomass every 2 ticks
                if self.ticks_elapsed % 2 == 0 {
                    // Consume every 2 ticks

                    // Get entity's hunger to determine demand
                    let demand =
                        if let Some(hunger) = world.get::<crate::entities::stats::Hunger>(entity) {
                            hunger.0.max - hunger.0.current
                        } else {
                            warn!("Entity {:?} has no hunger component, cannot graze", entity);
                            return ActionResult::Failed;
                        };

                    // Check giving-up conditions before consuming using ResourceGrid
                    let should_give_up = if let Some(initial_biomass) = self.initial_biomass {
                        if let Some(resource_grid) =
                            world.get_resource::<crate::vegetation::resource_grid::ResourceGrid>()
                        {
                            if let Some(current_cell) = resource_grid.get_cell(self.target_tile) {
                                // Giving up thresholds from old system
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

                    // Try to consume biomass using ResourceGrid's consume_at method
                    if let Some(mut resource_grid) =
                        world.get_resource_mut::<crate::vegetation::resource_grid::ResourceGrid>()
                    {
                        // MAX_MEAL_FRACTION from old system
                        const MAX_MEAL_FRACTION: f32 = 0.3;
                        let consumed =
                            resource_grid.consume_at(self.target_tile, demand, MAX_MEAL_FRACTION);

                        if consumed > 0.0 && !should_give_up {
                            // Successfully consumed, reduce hunger
                            if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                                if let Some(mut hunger) =
                                    entity_mut.get_mut::<crate::entities::stats::Hunger>()
                                {
                                    hunger.0.change(-consumed);
                                }
                            }

                            debug!(
                                "🐇 Entity {:?} grazing tick {}/{} - consumed {:.1} biomass",
                                entity, self.ticks_elapsed, self.duration_ticks, consumed
                            );
                        } else if consumed == 0.0 || should_give_up {
                            // No biomass or giving up, finish grazing
                            debug!(
                                "🌾 Entity {:?} {} grazing",
                                entity,
                                if should_give_up {
                                    "giving up early"
                                } else {
                                    "found no biomass"
                                }
                            );
                            return ActionResult::Success;
                        }
                    } else {
                        warn!("ResourceGrid resource not available for grazing");
                        return ActionResult::Failed;
                    }
                }

                // Continue grazing
                return ActionResult::InProgress;
            }

            // Grazing duration completed successfully
            debug!(
                "✅ Entity {:?} completed grazing at {:?} after {} ticks",
                entity, self.target_tile, self.ticks_elapsed
            );
            return ActionResult::Success;
        }

        // Start moving if not started yet
        if !self.started {
            debug!(
                "🐇 Entity {:?} moving to graze at {:?}",
                entity, self.target_tile
            );

            if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                entity_mut.insert(MoveOrder {
                    destination: self.target_tile,
                    allow_diagonal: true, // Enable diagonal pathfinding
                });
            }

            self.started = true;
        }

        // Still traveling
        ActionResult::InProgress
    }

    fn name(&self) -> &'static str {
        "Graze"
    }

    fn cancel(&mut self, world: &mut World, entity: Entity) {
        clear_navigation_state(world, entity);
        self.started = false;
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
    fn can_execute(&self, world: &World, entity: Entity, _tick: u64) -> bool {
        world.get::<Energy>(entity).is_some()
    }

    fn execute(&mut self, world: &mut World, entity: Entity, tick: u64) -> ActionResult {
        if !self.started {
            if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                if let Some(mut energy) = entity_mut.get_mut::<Energy>() {
                    energy.set_resting();
                    info!(
                        "😴 Entity {:?} started resting for {} ticks (energy: {:.1}%)",
                        entity,
                        self.duration_ticks,
                        energy.0.percentage()
                    );
                }
            }
            self.started = true;
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
            if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                if let Some(mut energy) = entity_mut.get_mut::<Energy>() {
                    energy.set_active();
                    info!(
                        "😊 Entity {:?} finished resting on tick {}! Energy: {:.1}%",
                        entity,
                        tick,
                        energy.0.percentage()
                    );
                }
            }
            return ActionResult::Success;
        }

        ActionResult::InProgress
    }

    fn cancel(&mut self, world: &mut World, entity: Entity) {
        // Reset energy state back to active when interrupted
        if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
            if let Some(mut energy) = entity_mut.get_mut::<Energy>() {
                energy.set_active();
                debug!(
                    "🚫 Entity {:?} resting interrupted, switching to active energy mode",
                    entity
                );
            }
        }
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
    fn can_execute(&self, world: &World, entity: Entity, _tick: u64) -> bool {
        world.get::<Hunger>(entity).is_some() && world.get::<Carcass>(self.carcass).is_some()
    }

    fn execute(&mut self, world: &mut World, entity: Entity, _tick: u64) -> ActionResult {
        let Some(position) = world.get::<TilePosition>(entity).copied() else {
            return ActionResult::Failed;
        };

        let Some(carcass_pos) = world.get::<TilePosition>(self.carcass).copied() else {
            debug!("🦴 Scavenge target vanished before arrival");
            return ActionResult::Failed;
        };

        if position.tile != carcass_pos.tile {
            if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                entity_mut.insert(MoveOrder {
                    destination: carcass_pos.tile,
                    allow_diagonal: true,
                });
            }
            self.started = true;
            return ActionResult::InProgress;
        }

        clear_navigation_state(world, entity);

        let bite_size = world
            .get::<SpeciesNeeds>(entity)
            .map(|n| n.eat_amount)
            .unwrap_or(50.0);

        let (consumed, spent) = if let Some(mut carcass) = world.get_mut::<Carcass>(self.carcass) {
            let consumed = carcass.consume(bite_size);
            let spent = carcass.is_spent();
            (consumed, spent)
        } else {
            return ActionResult::Failed;
        };

        if consumed <= 0.0 {
            return ActionResult::Failed;
        }

        if let Some(mut hunger) = world.get_mut::<Hunger>(entity) {
            hunger.0.change(-consumed);
        }

        if spent {
            if let Ok(carcass_entity) = world.get_entity_mut(self.carcass) {
                carcass_entity.despawn();
            }
        }

        info!(
            "🦴 Entity {:?} scavenged {:.1} nutrition from carcass {:?}",
            entity, consumed, self.carcass
        );

        self.started = false;
        ActionResult::Success
    }

    fn cancel(&mut self, world: &mut World, entity: Entity) {
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
#[derive(Debug, Clone)]
pub struct HuntAction {
    pub prey: Entity,
}

impl HuntAction {
    pub fn new(prey: Entity) -> Self {
        Self { prey }
    }
}

impl Action for HuntAction {
    fn can_execute(&self, world: &World, entity: Entity, _tick: u64) -> bool {
        world.get::<Hunger>(entity).is_some() && world.get::<TilePosition>(self.prey).is_some()
    }

    fn execute(&mut self, world: &mut World, entity: Entity, _tick: u64) -> ActionResult {
        let Some(predator_pos) = world.get::<TilePosition>(entity).copied() else {
            return ActionResult::Failed;
        };

        let Some(prey_pos) = world.get::<TilePosition>(self.prey).copied() else {
            debug!("🎯 Prey {:?} lost before hunt completed", self.prey);
            return ActionResult::Failed;
        };

        let diff = predator_pos.tile - prey_pos.tile;
        let distance = diff.x.abs().max(diff.y.abs()) as f32;

        if distance > 1.5 {
            if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                entity_mut.insert(MoveOrder {
                    destination: prey_pos.tile,
                    allow_diagonal: true,
                });
            }
            return ActionResult::InProgress;
        }

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
        if let Some(mut hunger) = world.get_mut::<Hunger>(entity) {
            hunger.0.change(-consumed);
        }

        let leftover = (available_meat - consumed).max(0.0);
        let species_label = world
            .get::<Creature>(self.prey)
            .map(|c| c.species.clone())
            .unwrap_or_else(|| "Prey".to_string());

        if let Ok(prey_entity) = world.get_entity_mut(self.prey) {
            prey_entity.despawn();
        }

        if leftover > MIN_CARCASS_NUTRITION {
            world.spawn((
                Carcass::new(species_label, leftover, DEFAULT_CARCASS_DECAY),
                TilePosition::from_tile(prey_pos.tile),
            ));
        }

        info!(
            "🐺 Entity {:?} hunted prey {:?}, consumed {:.1} nutrition",
            entity, self.prey, consumed
        );

        ActionResult::Success
    }

    fn cancel(&mut self, world: &mut World, entity: Entity) {
        clear_navigation_state(world, entity);
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
    fn can_execute(&self, world: &World, _entity: Entity, _tick: u64) -> bool {
        // Target must still exist and have a position
        world.get_entity(self.target).is_ok() && world.get::<TilePosition>(self.target).is_some()
    }

    fn execute(&mut self, world: &mut World, entity: Entity, _tick: u64) -> ActionResult {
        // Abort on pathfinding failure for this entity
        if world.get::<PathfindingFailed>(entity).is_some() {
            warn!(
                "Entity {:?} pathfinding failed while following, aborting Follow action",
                entity
            );
            if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                entity_mut.remove::<PathfindingFailed>();
            }
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

        // If not currently moving (no Path), issue/refresh a move order to the target's current tile
        let is_moving = world.get::<Path>(entity).is_some();
        if !is_moving {
            if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                entity_mut.insert(MoveOrder {
                    destination: target_pos.tile,
                    allow_diagonal: true,
                });
            }
            self.started = true;
        }

        ActionResult::InProgress
    }

    fn cancel(&mut self, world: &mut World, entity: Entity) {
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
    fn can_execute(&self, world: &World, _entity: Entity, _tick: u64) -> bool {
        world.get_entity(self.partner).is_ok() && world.get::<TilePosition>(self.partner).is_some()
    }

    fn execute(&mut self, world: &mut World, entity: Entity, tick: u64) -> ActionResult {
        use crate::entities::reproduction::{
            MatingIntent, Pregnancy, ReproductionConfig, ReproductionCooldown, Sex,
        };
        // Abort if either entity failed to find a path
        if world.get::<PathfindingFailed>(entity).is_some()
            || world.get::<PathfindingFailed>(self.partner).is_some()
        {
            warn!(
                "⚠️ MateAction: pathfinding failed for {:?} or {:?}, aborting",
                entity, self.partner
            );

            if let Some(mut me) = world.get_entity_mut(entity).ok() {
                me.remove::<MatingIntent>();
                me.remove::<PathfindingFailed>();
            }
            if let Some(mut partner) = world.get_entity_mut(self.partner).ok() {
                partner.remove::<MatingIntent>();
                partner.remove::<PathfindingFailed>();
            }

            return ActionResult::Failed;
        }

        // Abort if partner missing
        if world.get::<TilePosition>(self.partner).is_none() {
            if let Some(mut e) = world.get_entity_mut(entity).ok() {
                e.remove::<MatingIntent>();
            }
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
            if world.get::<Path>(entity).is_none() {
                if let Some(mut e) = world.get_entity_mut(entity).ok() {
                    e.insert(MoveOrder {
                        destination: self.meeting_tile,
                        allow_diagonal: true,
                    });
                }
            }
            return ActionResult::InProgress;
        }

        // We are on the meeting tile, ensure we're not still trying to path somewhere else
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
            // Partner is still approaching; keep encouraging movement and reset wait timer
            if world.get::<Path>(self.partner).is_none() {
                if let Some(mut partner_mut) = world.get_entity_mut(self.partner).ok() {
                    partner_mut.insert(MoveOrder {
                        destination: self.meeting_tile,
                        allow_diagonal: true,
                    });
                }
            }
            self.waited = self.waited.saturating_sub(1);

            // Debug logging for partner not adjacent
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

        // Duration complete: apply pregnancy and cooldowns (female only)
        let me_cfg = world.get::<ReproductionConfig>(entity).copied();
        let partner_cfg = world.get::<ReproductionConfig>(self.partner).copied();
        let me_female = world
            .get::<Sex>(entity)
            .is_some_and(|s| matches!(s, Sex::Female));
        let partner_female = world
            .get::<Sex>(self.partner)
            .is_some_and(|s| matches!(s, Sex::Female));

        if me_female {
            if let Some(cfg) = me_cfg.or(partner_cfg) {
                if let Some(mut e) = world.get_entity_mut(entity).ok() {
                    let litter = rand::thread_rng()
                        .gen_range(cfg.litter_size_range.0..=cfg.litter_size_range.1);
                    e.insert(Pregnancy {
                        remaining_ticks: cfg.gestation_ticks,
                        litter_size: litter,
                        father: Some(self.partner),
                    });
                    e.insert(ReproductionCooldown {
                        remaining_ticks: cfg.postpartum_cooldown_ticks,
                    });
                    info!(
                        "❤️ Pregnancy started for entity {:?} with father {:?} (litter size: {})",
                        entity, self.partner, litter
                    );
                }
                if let Some(mut p) = world.get_entity_mut(self.partner).ok() {
                    let male_cd = partner_cfg
                        .or(me_cfg)
                        .map(|cfg| cfg.mating_cooldown_ticks)
                        .unwrap_or(cfg.mating_cooldown_ticks);
                    p.insert(ReproductionCooldown {
                        remaining_ticks: male_cd,
                    });
                }
            } else {
                warn!(
                    "⚠️ MateAction: Missing reproduction config for female entity {:?}",
                    entity
                );
            }
        } else if partner_female {
            if let Some(cfg) = partner_cfg.or(me_cfg) {
                if let Some(mut me) = world.get_entity_mut(entity).ok() {
                    me.insert(ReproductionCooldown {
                        remaining_ticks: cfg.mating_cooldown_ticks,
                    });
                }
            } else {
                warn!(
                    "⚠️ MateAction: Missing reproduction config for female partner {:?}",
                    self.partner
                );
            }
        }

        // Clean up intents on both
        if let Some(mut e) = world.get_entity_mut(entity).ok() {
            e.remove::<MatingIntent>();
        }
        if let Some(mut p) = world.get_entity_mut(self.partner).ok() {
            p.remove::<MatingIntent>();
        }

        // Ensure neither retains stale movement orders
        clear_navigation_state(world, entity);
        clear_navigation_state(world, self.partner);

        ActionResult::Success
    }

    fn cancel(&mut self, world: &mut World, entity: Entity) {
        use crate::entities::reproduction::MatingIntent;

        // Clean up mating intents when interrupted
        if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
            entity_mut.remove::<MatingIntent>();
            debug!(
                "🚫 Entity {:?} mating interrupted, clearing mating intent",
                entity
            );
        }

        // Also clean up partner's intent
        if let Some(mut partner_mut) = world.get_entity_mut(self.partner).ok() {
            partner_mut.remove::<MatingIntent>();
            debug!(
                "🚫 Entity {:?} partner mating interrupted, clearing partner mating intent",
                entity
            );
        }

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
    fn can_execute(&self, world: &World, entity: Entity, _tick: u64) -> bool {
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

    fn execute(&mut self, world: &mut World, entity: Entity, tick: u64) -> ActionResult {
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

        // Perform the harvest using resource scope to get mutable access
        let harvest_result = world.resource_scope(
            |world, mut resource_grid: Mut<crate::vegetation::resource_grid::ResourceGrid>| {
                // Get world loader
                let world_loader = match world.get_resource::<crate::world_loader::WorldLoader>() {
                    Some(loader) => loader,
                    None => return ActionResult::Failed,
                };

                // Verify the resource is still present and harvestable
                let resource_at_tile = match world_loader.get_resource_at(self.target_tile.x, self.target_tile.y) {
                    Some(resource) => resource,
                    None => return ActionResult::Failed,
                };

                let actual_resource = match ResourceType::from_str(&resource_at_tile) {
                    Some(resource) => resource,
                    None => return ActionResult::Failed,
                };

                if actual_resource != self.resource_type || !actual_resource.is_gatherable() {
                    return ActionResult::Failed;
                }

                // Get harvest profile for this resource
                let harvest_profile = match actual_resource.get_harvest_profile() {
                    Some(profile) => profile,
                    None => return ActionResult::Failed,
                };

                // Perform the harvest
                if let Some(cell) = resource_grid.get_cell_mut(self.target_tile) {
                    // Check if resource is ready for harvest (past regrowth delay)
                    if tick < cell.regrowth_available_tick {
                        return ActionResult::Failed;
                    }

                    // Apply harvest yield
                    let harvested_amount = harvest_profile.harvest_yield.min(cell.total_biomass as u32);
                    cell.total_biomass = (cell.total_biomass - harvested_amount as f32).max(0.0);

                    // Apply regrowth delay for collectable resources
                    cell.regrowth_available_tick = tick + harvest_profile.regrowth_delay_ticks;
                    cell.last_update_tick = tick;

                    info!(
                        "🧺 Entity {:?} harvested {}x {} at tile {:?} (yield: {})",
                        entity, harvested_amount, actual_resource.as_str(), self.target_tile, harvested_amount
                    );

                    ActionResult::Success
                } else {
                    ActionResult::Failed
                }
            }
        );

        if harvest_result == ActionResult::Success {
            self.completed = true;
        }

        harvest_result
    }

    fn cancel(&mut self, _world: &mut World, _entity: Entity) {
        // No special cleanup needed for harvest actions
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
#[derive(Debug, Clone)]
pub struct WanderAction {
    pub target_tile: IVec2,
    pub started: bool,
}

impl WanderAction {
    pub fn new(target_tile: IVec2) -> Self {
        Self {
            target_tile,
            started: false,
        }
    }
}

impl Action for WanderAction {
    fn can_execute(&self, world: &World, entity: Entity, _tick: u64) -> bool {
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

    fn execute(&mut self, world: &mut World, entity: Entity, _tick: u64) -> ActionResult {
        // Get current position
        let Some(pos) = world.get::<TilePosition>(entity) else {
            return ActionResult::Failed;
        };
        let current_pos = pos.tile;

        // Check if arrived
        if current_pos == self.target_tile {
            return ActionResult::Success;
        }

        // Start pathfinding
        if !self.started {
            if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
                entity_mut.insert(MoveOrder {
                    destination: self.target_tile,
                    allow_diagonal: true,
                });
            }
            self.started = true;
        }

        // Check for pathfinding failure
        if world.get::<PathfindingFailed>(entity).is_some() {
            return ActionResult::Failed;
        }

        ActionResult::InProgress
    }

    fn cancel(&mut self, world: &mut World, entity: Entity) {
        if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
            entity_mut.remove::<MoveOrder>();
            entity_mut.remove::<Path>();
        }
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
