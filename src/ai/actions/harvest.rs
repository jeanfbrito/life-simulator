use super::*;
use crate::entities::TilePosition;
use crate::resources::ResourceType;
use crate::simulation::tick::SimulationTick;
use bevy::prelude::*;

// =============================================================================
// HARVEST ACTION
// =============================================================================

/// Harvest Action - Collect harvestable resources like mushrooms, roots, etc.
#[derive(Debug, Clone)]
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

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
