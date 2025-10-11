/// Collectable resource utilities and API
///
/// Provides functions for finding, filtering, and managing collectable resources
/// for future gameplay systems like gathering and crafting.

use crate::resources::{ResourceType, ResourceCategory, ConsumptionKind};
use crate::world_loader::WorldLoader;
use crate::vegetation::resource_grid::ResourceGrid;
use bevy::prelude::*;
use std::collections::HashMap;

/// Configuration for collectable searches
#[derive(Debug, Clone)]
pub struct CollectableSearchConfig {
    /// Search radius in tiles
    pub radius: i32,
    /// Resource types to include (None = all collectables)
    pub resource_types: Option<Vec<ResourceType>>,
    /// Minimum biomass threshold
    pub min_biomass: f32,
    /// Check regrowth delay
    pub check_regrowth: bool,
}

impl Default for CollectableSearchConfig {
    fn default() -> Self {
        Self {
            radius: 20,
            resource_types: None,
            min_biomass: 10.0,
            check_regrowth: true,
        }
    }
}

/// Information about a collectable resource
#[derive(Debug, Clone)]
pub struct CollectableInfo {
    pub resource_type: ResourceType,
    pub position: IVec2,
    pub biomass: f32,
    pub harvest_yield: u32,
    pub regrowth_available_tick: u64,
    pub nutritional_value: f32,
}

/// Find collectable resources within search radius of a position
pub fn get_collectable_targets(
    center: IVec2,
    config: &CollectableSearchConfig,
    world_loader: &WorldLoader,
    resource_grid: &ResourceGrid,
    current_tick: u64,
) -> Vec<CollectableInfo> {
    let mut targets = Vec::new();

    // Define search bounds
    let min_x = center.x - config.radius;
    let max_x = center.x + config.radius;
    let min_y = center.y - config.radius;
    let max_y = center.y + config.radius;

    // Search area for collectable resources
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            let pos = IVec2::new(x, y);

            // Check distance (circular radius)
            if (pos - center).length_squared() as i32 > config.radius * config.radius {
                continue;
            }

            // Check if resource exists and is collectable
            if let Some(resource_str) = world_loader.get_resource_at(x, y) {
                if let Some(resource_type) = ResourceType::from_str(&resource_str) {
                    if !is_collectable(&resource_type) {
                        continue;
                    }

                    // Filter by specific resource types if specified
                    if let Some(ref allowed_types) = config.resource_types {
                        if !allowed_types.contains(&resource_type) {
                            continue;
                        }
                    }

                    // Check biomass in resource grid
                    if let Some(cell) = resource_grid.get_cell(pos) {
                        if cell.total_biomass < config.min_biomass {
                            continue;
                        }

                        // Check regrowth delay if requested
                        if config.check_regrowth && current_tick < cell.regrowth_available_tick {
                            continue;
                        }

                        // Get harvest profile
                        let harvest_profile = match resource_type.get_harvest_profile() {
                            Some(profile) => profile,
                            None => continue,
                        };

                        targets.push(CollectableInfo {
                            resource_type,
                            position: pos,
                            biomass: cell.total_biomass,
                            harvest_yield: harvest_profile.harvest_yield,
                            regrowth_available_tick: cell.regrowth_available_tick,
                            nutritional_value: harvest_profile.nutritional_value,
                        });
                    }
                }
            }
        }
    }

    // Sort by distance (closest first)
    targets.sort_by_key(|t| (t.position - center).length_squared());
    targets
}

/// Check if a resource type is collectable
pub fn is_collectable(resource_type: &ResourceType) -> bool {
    resource_type.get_category() == Some(ResourceCategory::Collectable)
        && resource_type.get_consumption_kind() == Some(ConsumptionKind::HumanGather)
}

/// Get all collectable resource types
pub fn get_all_collectable_types() -> Vec<ResourceType> {
    use std::collections::HashSet;

    let mut collectables = HashSet::new();

    // Iterate through all resource types and filter collectables
    // This is a placeholder - in a real implementation, we might want to
    // cache this or have a more efficient method
    for resource_str in [
        "MushroomPatch", "WildRoot", "BerryBush", "HazelShrub" // Add more as needed
    ] {
        if let Some(resource_type) = ResourceType::from_str(resource_str) {
            if is_collectable(&resource_type) {
                collectables.insert(resource_type);
            }
        }
    }

    collectables.into_iter().collect()
}

/// Get collectable statistics for an area
pub fn get_collectable_stats(
    center: IVec2,
    radius: i32,
    world_loader: &WorldLoader,
    resource_grid: &ResourceGrid,
) -> HashMap<ResourceType, CollectableStats> {
    let mut stats = HashMap::new();

    let config = CollectableSearchConfig {
        radius,
        resource_types: None,
        min_biomass: 0.0,
        check_regrowth: false,
    };

    let targets = get_collectable_targets(center, &config, world_loader, resource_grid, 0);

    for target in targets {
        let entry = stats.entry(target.resource_type.clone()).or_insert_with(|| CollectableStats {
            resource_type: target.resource_type.clone(),
            count: 0,
            total_biomass: 0.0,
            average_biomass: 0.0,
            total_yield: 0,
            ready_to_harvest: 0,
        });

        entry.count += 1;
        entry.total_biomass += target.biomass;
        entry.total_yield += target.harvest_yield;

        if target.regrowth_available_tick == 0 {
            entry.ready_to_harvest += 1;
        }
    }

    // Calculate averages
    for stats in stats.values_mut() {
        if stats.count > 0 {
            stats.average_biomass = stats.total_biomass / stats.count as f32;
        }
    }

    stats
}

/// Statistics for collectable resources in an area
#[derive(Debug, Clone)]
pub struct CollectableStats {
    pub resource_type: ResourceType,
    pub count: u32,
    pub total_biomass: f32,
    pub average_biomass: f32,
    pub total_yield: u32,
    pub ready_to_harvest: u32,
}

/// Debug function to list all collectables in range
pub fn debug_list_collectables(
    center: IVec2,
    radius: i32,
    world_loader: &WorldLoader,
    resource_grid: &ResourceGrid,
    current_tick: u64,
) {
    let config = CollectableSearchConfig {
        radius,
        resource_types: None,
        min_biomass: 1.0,
        check_regrowth: false,
    };

    let targets = get_collectable_targets(center, &config, world_loader, resource_grid, current_tick);

    println!("🧺 Collectables within {} tiles of {:?}:", radius, center);
    println!("   Total found: {}", targets.len());

    for target in targets {
        let distance = ((target.position - center).length_squared() as f32).sqrt();
        let regrowth_status = if current_tick >= target.regrowth_available_tick {
            "✅ Ready"
        } else {
            "⏳ Waiting"
        };

        println!("   {} at {:?} (dist: {:.1}, biomass: {:.1}, yield: {}, {})",
            target.resource_type.as_str(),
            target.position,
            distance,
            target.biomass,
            target.harvest_yield,
            regrowth_status
        );
    }
}

/// JSON serialization functions for web API
pub mod web_api {
    use super::*;
    use crate::simulation::SimulationTick;
    use crate::vegetation::resource_grid::ResourceGrid;
    use crate::world_loader::WorldLoader;
    use serde_json;

    /// Get collectable statistics as JSON for web API
    pub fn get_collectable_stats_json() -> String {
        // Try to get access to world data
        let (world_loader, resource_grid, tick) = match get_world_resources() {
            Some(resources) => resources,
            None => {
                return serde_json::json!({
                    "error": "World resources not available",
                    "timestamp": "2025-01-07T00:00:00Z"
                }).to_string();
            }
        };

        let stats = get_collectable_stats(IVec2::new(0, 0), 20, &world_loader, &resource_grid);

        let mut stats_map = serde_json::Map::new();
        for (resource_type, stat) in stats {
            stats_map.insert(
                resource_type.as_str().to_string(),
                serde_json::json!({
                    "resource_type": resource_type.as_str(),
                    "count": stat.count,
                    "total_biomass": stat.total_biomass,
                    "average_biomass": stat.average_biomass,
                    "total_yield": stat.total_yield,
                    "ready_to_harvest": stat.ready_to_harvest,
                    "category": "collectable"
                })
            );
        }

        serde_json::json!({
            "center": {"x": 0, "y": 0},
            "radius": 20,
            "timestamp": "2025-01-07T00:00:00Z",
            "current_tick": tick,
            "statistics": stats_map,
            "total_collectable_types": stats_map.len()
        }).to_string()
    }

    /// Get debug information about collectables as JSON
    pub fn debug_collectables_json() -> String {
        let (world_loader, resource_grid, tick) = match get_world_resources() {
            Some(resources) => resources,
            None => {
                return serde_json::json!({
                    "error": "World resources not available",
                    "timestamp": "2025-01-07T00:00:00Z"
                }).to_string();
            }
        };

        let config = CollectableSearchConfig {
            radius: 15,
            resource_types: None,
            min_biomass: 1.0,
            check_regrowth: false,
        };

        let targets = get_collectable_targets(IVec2::new(0, 0), &config, &world_loader, &resource_grid, tick);

        let targets_json: Vec<serde_json::Value> = targets.into_iter().map(|target| {
            serde_json::json!({
                "resource_type": target.resource_type.as_str(),
                "position": {
                    "x": target.position.x,
                    "y": target.position.y
                },
                "biomass": target.biomass,
                "harvest_yield": target.harvest_yield,
                "regrowth_available_tick": target.regrowth_available_tick,
                "nutritional_value": target.nutritional_value,
                "distance_from_origin": ((target.position.x * target.position.x + target.position.y * target.position.y) as f32).sqrt()
            })
        }).collect();

        serde_json::json!({
            "center": {"x": 0, "y": 0},
            "radius": 15,
            "timestamp": "2025-01-07T00:00:00Z",
            "current_tick": tick,
            "total_found": targets_json.len(),
            "collectables": targets_json
        }).to_string()
    }

    /// Get all collectable types as JSON
    pub fn get_collectable_types_json() -> String {
        let collectable_types = get_all_collectable_types();

        let types_json: Vec<serde_json::Value> = collectable_types.into_iter().map(|resource_type| {
            serde_json::json!({
                "name": resource_type.as_str(),
                "category": "collectable",
                "gatherable": is_collectable(&resource_type),
                "herbivore_edible": resource_type.is_herbivore_edible(),
                "nutritional_value": resource_type.get_harvest_profile()
                    .map(|p| p.nutritional_value)
                    .unwrap_or(0.0),
                "harvest_yield": resource_type.get_harvest_profile()
                    .map(|p| p.harvest_yield)
                    .unwrap_or(0),
                "regrowth_delay_ticks": resource_type.get_harvest_profile()
                    .map(|p| p.regrowth_delay_ticks)
                    .unwrap_or(0)
            })
        }).collect();

        serde_json::json!({
            "collectable_types": types_json,
            "total_count": types_json.len(),
            "timestamp": "2025-01-07T00:00:00Z"
        }).to_string()
    }

    /// Helper function to get world resources safely
    fn get_world_resources() -> Option<(WorldLoader, ResourceGrid, u64)> {
        // For now, return dummy data to make the API work
        // In a real implementation, this would access the actual world data
        let world_loader = WorldLoader::load_default().ok()?;

        // Create a basic resource grid for this purpose
        let resource_grid = ResourceGrid::new();

        // Use a default tick
        let tick = 0;

        Some((world_loader, resource_grid, tick))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::{ResourceType, ResourceCategory, ConsumptionKind};

    #[test]
    fn test_collectable_identification() {
        // Test known collectables
        assert!(is_collectable(&ResourceType::MushroomPatch));
        assert!(is_collectable(&ResourceType::WildRoot));

        // Test non-collectables
        assert!(!is_collectable(&ResourceType::OakTree));
        assert!(!is_collectable(&ResourceType::BirchShrub));
    }

    #[test]
    fn test_collectable_search_config() {
        let config = CollectableSearchConfig::default();
        assert_eq!(config.radius, 20);
        assert_eq!(config.min_biomass, 10.0);
        assert!(config.check_regrowth);
        assert!(config.resource_types.is_none());

        let custom_config = CollectableSearchConfig {
            radius: 10,
            resource_types: Some(vec![ResourceType::MushroomPatch]),
            min_biomass: 5.0,
            check_regrowth: false,
        };
        assert_eq!(custom_config.radius, 10);
        assert_eq!(custom_config.min_biomass, 5.0);
        assert!(!custom_config.check_regrowth);
        assert!(custom_config.resource_types.is_some());
    }
}