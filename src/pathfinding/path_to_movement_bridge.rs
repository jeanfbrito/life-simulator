/// PathReady → MovementComponent Bridge System
///
/// CRITICAL MISSING PIECE: This system bridges pathfinding results to actual movement.
/// When pathfinding completes and adds PathReady component, this system:
/// 1. Extracts the computed path waypoints
/// 2. Creates MovementComponent::FollowingPath with the waypoints
/// 3. Removes PathReady component
///
/// Without this system, entities get paths but never start moving!

use bevy::prelude::*;
use crate::entities::{MovementComponent, TilePosition};
use crate::pathfinding::PathReady;

/// Bridge system: PathReady → MovementComponent
///
/// This system completes the pathfinding → movement pipeline by converting
/// computed paths into actual entity movement.
pub fn bridge_path_ready_to_movement(
    mut commands: Commands,
    query: Query<(Entity, &PathReady, &TilePosition)>,
) {
    for (entity, path_ready, tile_pos) in query.iter() {
        let waypoints = path_ready.path.as_ref().clone();

        debug!(
            "🗺️ PathReady Bridge: Entity {:?} starting path with {} waypoints from {:?}",
            entity,
            waypoints.len(),
            tile_pos.tile
        );

        // Create MovementComponent with the computed path
        let movement = MovementComponent::following_path(waypoints);

        // Insert movement component and remove PathReady
        commands.entity(entity)
            .insert(movement)
            .remove::<PathReady>();
    }
}
