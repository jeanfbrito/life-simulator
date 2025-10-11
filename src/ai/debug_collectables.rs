/// Debug systems for collectable harvesting functionality
///
/// Provides debugging tools and commands for testing collectable resources
/// and harvest mechanics during development.

use crate::ai::collectables::*;
use crate::simulation::SimulationTick;
use crate::vegetation::resource_grid::ResourceGrid;
use crate::world_loader::WorldLoader;
use bevy::prelude::*;

/// Plugin that adds collectable debug systems
pub struct CollectableDebugPlugin;

impl Plugin for CollectableDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, debug_collectable_commands);
    }
}

/// System to process debug commands for collectables
fn debug_collectable_commands(
    tick: Res<SimulationTick>,
    world_loader: Option<Res<WorldLoader>>,
    resource_grid: Option<Res<ResourceGrid>>,
) {
    // Skip if systems aren't available
    if world_loader.is_none() || resource_grid.is_none() {
        return;
    }

    let world_loader = world_loader.unwrap();
    let resource_grid = resource_grid.unwrap();

    // Simple debug trigger - every 500 ticks (50 seconds at 10 TPS)
    if tick.0 % 500 == 0 && tick.0 > 0 {
        // List collectables around origin
        debug_list_collectables(IVec2::new(0, 0), 15, &world_loader, &resource_grid, tick.0);

        // Show statistics for the area
        let stats = get_collectable_stats(IVec2::new(0, 0), 15, &world_loader, &resource_grid);
        println!("📊 Collectable Statistics (15 tile radius):");

        if stats.is_empty() {
            println!("   No collectables found in range.");
        } else {
            for (resource_type, stat) in stats {
                println!("   {}: {} patches, {:.1} total biomass, {} ready to harvest",
                    resource_type.as_str(),
                    stat.count,
                    stat.total_biomass,
                    stat.ready_to_harvest
                );
            }
        }

        // Test harvest action creation (if there are collectables)
        let config = CollectableSearchConfig {
            radius: 15,
            resource_types: None,
            min_biomass: 5.0,
            check_regrowth: false,
        };

        let targets = get_collectable_targets(IVec2::new(0, 0), &config, &world_loader, &resource_grid, tick.0);

        if !targets.is_empty() {
            println!("🎯 Test Harvest Targets:");
            for (i, target) in targets.iter().take(3).enumerate() {
                println!("   {}. {} at {:?} (biomass: {:.1}, yield: {})",
                    i + 1,
                    target.resource_type.as_str(),
                    target.position,
                    target.biomass,
                    target.harvest_yield
                );
            }
        }

        println!("---");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collectable_debug_plugin() {
        // Basic test to ensure plugin can be created
        let _plugin = CollectableDebugPlugin;
        // In a real test, we would set up a full Bevy app and test the systems
    }
}