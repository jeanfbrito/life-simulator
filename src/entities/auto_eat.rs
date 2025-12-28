use crate::entities::types::SpeciesNeeds;
use crate::entities::{stats::Hunger, Herbivore, TilePosition};
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;
/// Auto-eat system for herbivores
///
/// When an entity is standing on grass and is hungry, automatically eat
use bevy::prelude::*;

/// System that auto-eats grass when standing on it and hungry
pub fn auto_eat_system(
    mut query: Query<(Entity, &TilePosition, &mut Hunger, Option<&SpeciesNeeds>), With<Herbivore>>,
    world_loader: Res<WorldLoader>,
) {
    for (entity, position, mut hunger, needs) in query.iter_mut() {
        // Eat when hungry (normalized >= 0.15, i.e., hunger >= 15 out of 100)
        // normalized() = 0.0 (not hungry) to 1.0 (starving)
        // Skip if not hungry enough (normalized < 0.15, i.e., hunger < 15)
        if hunger.0.normalized() < 0.15 {
            // Entity is not hungry enough, skip
            continue;
        }

        // Check if standing on herbivore-friendly terrain (grass, forest, swamp where vegetation grows)
        if let Some(terrain_str) = world_loader.get_terrain_at(position.tile.x, position.tile.y) {
            if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                // Can eat on grass, forest, and other fertile terrains
                let can_eat = matches!(terrain,
                    TerrainType::Grass | TerrainType::Forest | TerrainType::Swamp | TerrainType::Dirt
                );

                if can_eat {
                    // Eat the vegetation! Use species-specific amount if available
                    let amount = needs.map(|n| n.eat_amount).unwrap_or(25.0);
                    hunger.0.change(-amount);

                    info!(
                        "🐇 Entity {:?} ate vegetation on {:?} at {:?}! Hunger reduced by {:.1} (now: {:.1}%)",
                        entity,
                        format!("{:?}", terrain),
                        position.tile,
                        amount,
                        hunger.0.percentage()
                    );
                }
            }
        }
    }
}
