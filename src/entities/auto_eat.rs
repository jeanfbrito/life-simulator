/// Auto-eat system for herbivores
/// 
/// When an entity is standing on grass and is hungry, automatically eat

use bevy::prelude::*;
use crate::entities::{TilePosition, stats::Hunger};
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;

/// System that auto-eats grass when standing on it and hungry
pub fn auto_eat_system(
    mut query: Query<(Entity, &TilePosition, &mut Hunger)>,
    world_loader: Res<WorldLoader>,
) {
    for (entity, position, mut hunger) in query.iter_mut() {
        // Only eat if hungry (above 30%)
        if hunger.0.normalized() < 0.3 {
            continue;
        }
        
        // Check if standing on grass
        if let Some(terrain_str) = world_loader.get_terrain_at(position.tile.x, position.tile.y) {
            if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                if matches!(terrain, TerrainType::Grass) {
                    // Eat the grass!
                    hunger.0.change(-25.0);
                    
                    info!(
                        "🐇 Entity {:?} ate grass at {:?}! Hunger: {:.1}%",
                        entity,
                        position.tile,
                        hunger.0.percentage()
                    );
                }
            }
        }
    }
}
