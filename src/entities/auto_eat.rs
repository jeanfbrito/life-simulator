use crate::entities::types::SpeciesNeeds;
use crate::entities::{stats::Hunger, TilePosition};
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;
/// Auto-eat system for herbivores
///
/// When an entity is standing on grass and is hungry, automatically eat
use bevy::prelude::*;

/// System that auto-eats grass when standing on it and hungry
pub fn auto_eat_system(
    mut query: Query<(Entity, &TilePosition, &mut Hunger, Option<&SpeciesNeeds>)>,
    world_loader: Res<WorldLoader>,
) {
    for (entity, position, mut hunger, needs) in query.iter_mut() {
        // Eat while at or above a lower hysteresis threshold (15%)
        // This lets entities keep eating a bit more once they start, for completeness.
        if hunger.0.normalized() < 0.15 {
            continue;
        }

        // Check if standing on grass
        if let Some(terrain_str) = world_loader.get_terrain_at(position.tile.x, position.tile.y) {
            if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                if matches!(terrain, TerrainType::Grass) {
                    // Eat the grass! Use species-specific amount if available
                    let amount = needs.map(|n| n.eat_amount).unwrap_or(25.0);
                    hunger.0.change(-amount);

                    info!(
                        "🐇 Entity {:?} ate grass at {:?}! Hunger reduced by {:.1} (now: {:.1}%)",
                        entity,
                        position.tile,
                        amount,
                        hunger.0.percentage()
                    );
                }
            }
        }
    }
}
