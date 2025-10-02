/// Wandering AI - entities that move randomly around the map
use bevy::prelude::*;
use rand::Rng;

use crate::{
    entities::movement::{TilePosition, MoveOrder, MovementSpeed},
    pathfinding::PathfindingGrid,
    simulation::SimulationTick,
};

// ============================================================================
// COMPONENTS
// ============================================================================

/// Component that makes an entity wander randomly around the map
#[derive(Component, Debug)]
pub struct Wanderer {
    /// Radius in tiles to wander from spawn point
    pub wander_radius: i32,
    /// Center point for wandering (usually spawn position)
    pub home_position: IVec2,
    /// Ticks to wait before picking new destination (when idle)
    pub idle_ticks: u32,
    /// Current ticks spent idle
    pub ticks_idle: u32,
}

impl Wanderer {
    pub fn new(home_position: IVec2, wander_radius: i32) -> Self {
        Self {
            wander_radius,
            home_position,
            idle_ticks: 10, // Wait 1 second before moving again (at 10 TPS)
            ticks_idle: 0,
        }
    }
    
    pub fn with_idle_time(mut self, ticks: u32) -> Self {
        self.idle_ticks = ticks;
        self
    }
}

// ============================================================================
// SYSTEMS
// ============================================================================

/// System that makes wanderers pick random destinations when idle
/// Runs every 5 ticks to avoid constant recalculation
pub fn wanderer_ai_system(
    mut commands: Commands,
    mut query: Query<(Entity, &TilePosition, &mut Wanderer), Without<MoveOrder>>,
    pathfinding_grid: Res<PathfindingGrid>,
    tick: Res<SimulationTick>,
) {
    // Only run every 5 ticks
    if tick.0 % 5 != 0 {
        return;
    }
    
    let mut rng = rand::thread_rng();
    
    for (entity, position, mut wanderer) in query.iter_mut() {
        wanderer.ticks_idle += 5; // Add 5 since we skip 4 ticks
        
        // Wait for idle period before picking new destination
        if wanderer.ticks_idle < wanderer.idle_ticks {
            continue;
        }
        
        // Reset idle counter
        wanderer.ticks_idle = 0;
        
        // Pick random destination within wander radius
        let destination = pick_random_walkable_tile(
            wanderer.home_position,
            wanderer.wander_radius,
            &pathfinding_grid,
            &mut rng,
        );
        
        if let Some(dest) = destination {
            // Don't move to current position
            if dest != position.tile {
                debug!("Entity {:?} wandering from {:?} to {:?}", 
                       entity, position.tile, dest);
                
                commands.entity(entity).insert(MoveOrder {
                    destination: dest,
                    allow_diagonal: false,
                });
            }
        } else {
            warn!("Entity {:?} couldn't find walkable tile near {:?}", 
                  entity, wanderer.home_position);
        }
    }
}

/// Pick a random walkable tile within radius
fn pick_random_walkable_tile(
    center: IVec2,
    radius: i32,
    grid: &PathfindingGrid,
    rng: &mut impl Rng,
) -> Option<IVec2> {
    // Try up to 20 random positions
    for _ in 0..20 {
        let offset_x = rng.gen_range(-radius..=radius);
        let offset_y = rng.gen_range(-radius..=radius);
        let candidate = center + IVec2::new(offset_x, offset_y);
        
        if grid.is_walkable(candidate) {
            return Some(candidate);
        }
    }
    
    // If we couldn't find anything, try a simple grid search
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            let candidate = center + IVec2::new(dx, dy);
            if grid.is_walkable(candidate) {
                return Some(candidate);
            }
        }
    }
    
    None
}

// ============================================================================
// SPAWN HELPERS
// ============================================================================

/// Spawn a wandering person at a specific position
pub fn spawn_wandering_person(
    commands: &mut Commands,
    name: impl Into<String>,
    position: IVec2,
    wander_radius: i32,
) -> Entity {
    commands.spawn((
        crate::entities::Creature {
            name: name.into(),
            species: "Human".to_string(),
        },
        TilePosition::from_tile(position),
        MovementSpeed::custom(15), // 1 tile per 15 ticks = 0.67 tiles/sec (walking speed)
        Wanderer::new(position, wander_radius),
    )).id()
}

/// Spawn multiple wandering people at random positions
pub fn spawn_wandering_people(
    commands: &mut Commands,
    count: usize,
    center: IVec2,
    spawn_radius: i32,
    wander_radius: i32,
    grid: &PathfindingGrid,
) -> Vec<Entity> {
    let mut entities = Vec::new();
    let mut rng = rand::thread_rng();
    
    for i in 0..count {
        // Find a random walkable spawn position
        if let Some(spawn_pos) = pick_random_walkable_tile(center, spawn_radius, grid, &mut rng) {
            let name = format!("Wanderer_{}", i);
            let entity = spawn_wandering_person(commands, name, spawn_pos, wander_radius);
            entities.push(entity);
            info!("Spawned wanderer at {:?}", spawn_pos);
        } else {
            warn!("Couldn't find walkable spawn position for wanderer {}", i);
        }
    }
    
    entities
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wanderer_creation() {
        let wanderer = Wanderer::new(IVec2::new(5, 5), 10);
        assert_eq!(wanderer.home_position, IVec2::new(5, 5));
        assert_eq!(wanderer.wander_radius, 10);
        assert_eq!(wanderer.ticks_idle, 0);
    }
    
    #[test]
    fn test_wanderer_with_idle_time() {
        let wanderer = Wanderer::new(IVec2::new(0, 0), 5)
            .with_idle_time(20);
        assert_eq!(wanderer.idle_ticks, 20);
    }
}
