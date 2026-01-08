// ============================================================================
// Spatial Cell Component Infrastructure
// ============================================================================
//
// Phase 4.1: Create SpatialCell marker component and grid resource
//
// This provides the foundation for migrating from HashMap-based
// SpatialEntityIndex to a Parent/Child hierarchy system.
//
// Design:
// - SpatialCell component marks entities representing spatial grid cells
// - SpatialCellGrid resource provides O(1) lookups from chunk coords to entities
// - spawn_spatial_grid creates 4096 cell entities (64x64 grid, -32 to +32)
//
// ============================================================================

use bevy::prelude::*;
use std::collections::HashMap;

/// Grid chunk size - matches existing SpatialEntityIndex constant
pub const CHUNK_SIZE: i32 = 16;

/// Grid dimensions: -32 to +32 in both axes (64x64 = 4096 cells)
const GRID_MIN: i32 = -32;
const GRID_MAX: i32 = 32;

// ============================================================================
// COMPONENTS
// ============================================================================

/// Marker component for spatial grid cell entities
///
/// Each SpatialCell represents a 16x16 tile chunk in the world.
/// In Phase 4.2+, entities will be parented to these cells for spatial queries.
#[derive(Component, Debug, Clone, Copy)]
pub struct SpatialCell {
    /// Which chunk coordinate this cell represents
    pub chunk_coord: IVec2,
}

/// Marker component for entities that have been reparented to spatial cells
///
/// Used to prevent duplicate reparenting operations.
///
/// Phase 4: Required Components
/// SpatiallyParented automatically requires TilePosition - compile-time guarantee
/// that any entity in the spatial grid has a position.
#[derive(Component, Debug, Clone, Copy)]
#[require(crate::entities::TilePosition)]
pub struct SpatiallyParented;

// ============================================================================
// RESOURCES
// ============================================================================

/// Resource for O(1) lookups from chunk coordinates to cell entities
///
/// Provides fast spatial cell lookups without iterating through all entities.
/// Initialized at startup with 4096 cell entities covering a 1024x1024 world.
#[derive(Resource, Debug)]
pub struct SpatialCellGrid {
    /// HashMap from chunk coordinates to cell entity IDs
    cells: HashMap<IVec2, Entity>,
    /// Chunk size (tiles per chunk)
    chunk_size: i32,
}

impl SpatialCellGrid {
    /// Create a new empty grid with the specified chunk size
    pub fn new(chunk_size: i32) -> Self {
        Self {
            cells: HashMap::with_capacity(4096),
            chunk_size,
        }
    }

    /// Insert a cell entity at a specific chunk coordinate
    pub fn insert_cell(&mut self, chunk_coord: IVec2, entity: Entity) {
        self.cells.insert(chunk_coord, entity);
    }

    /// Get the cell entity for a specific chunk coordinate
    ///
    /// Returns None if no cell exists at that coordinate.
    pub fn get_cell(&self, chunk_coord: IVec2) -> Option<Entity> {
        self.cells.get(&chunk_coord).copied()
    }

    /// Convert a world position to its chunk coordinate
    ///
    /// Uses div_euclid for correct handling of negative coordinates.
    pub fn chunk_coord_for_position(&self, world_pos: IVec2) -> IVec2 {
        IVec2::new(
            world_pos.x.div_euclid(self.chunk_size),
            world_pos.y.div_euclid(self.chunk_size),
        )
    }

    /// Get the chunk size
    pub fn chunk_size(&self) -> i32 {
        self.chunk_size
    }

    /// Get the total number of cells in the grid
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Check if a chunk coordinate is within grid bounds
    pub fn is_in_bounds(&self, chunk_coord: IVec2) -> bool {
        chunk_coord.x >= GRID_MIN
            && chunk_coord.x < GRID_MAX
            && chunk_coord.y >= GRID_MIN
            && chunk_coord.y < GRID_MAX
    }
}

// ============================================================================
// SYSTEMS
// ============================================================================

/// Spawn spatial grid at startup
///
/// Creates 4096 SpatialCell entities (64x64 grid) and initializes
/// the SpatialCellGrid resource for O(1) lookups.
///
/// Grid coverage: -32 to +32 chunks = -512 to +512 tiles (1024x1024)
pub fn spawn_spatial_grid(mut commands: Commands) {
    let chunk_size = CHUNK_SIZE;
    let mut cells = HashMap::with_capacity(4096);

    // Spawn 64x64 grid = 4096 cell entities
    for x in GRID_MIN..GRID_MAX {
        for y in GRID_MIN..GRID_MAX {
            let chunk_coord = IVec2::new(x, y);

            // Spawn cell entity with marker component and name
            let cell_entity = commands
                .spawn((
                    SpatialCell { chunk_coord },
                    Name::new(format!("SpatialCell({},{})", x, y)),
                ))
                .id();

            cells.insert(chunk_coord, cell_entity);
        }
    }

    let cell_count = cells.len();

    // Insert grid resource
    commands.insert_resource(SpatialCellGrid { cells, chunk_size });

    info!(
        "Spawned spatial grid: {} cells, chunk_size: {}",
        cell_count,
        chunk_size
    );
}

/// Budget-controlled reparenting system with change detection
///
/// Incrementally migrates entities from HashMap-based spatial tracking to Parent/Child hierarchy.
/// Processes a budget of entities per tick (default: 50) to maintain 10 TPS performance.
///
/// Uses change detection to only process entities with TilePosition changes, combined with
/// budget control to distribute work across multiple ticks.
///
/// Phase 4.2: Initial reparenting for entities without SpatiallyParented marker
pub fn reparent_entities_to_cells(
    mut commands: Commands,
    grid: Res<SpatialCellGrid>,
    entities: Query<
        (Entity, &crate::entities::movement::TilePosition),
        (Without<SpatiallyParented>, Changed<crate::entities::movement::TilePosition>),
    >,
) {
    const BUDGET: usize = 50; // Process 50 entities per tick to maintain 10 TPS

    let total_pending = entities.iter().count();
    if total_pending == 0 {
        return; // No work to do
    }

    let mut processed = 0;
    for (entity, pos) in entities.iter().take(BUDGET) {
        let chunk_coord = grid.chunk_coord_for_position(pos.tile);

        // Only reparent if cell exists in grid
        if let Some(cell_entity) = grid.get_cell(chunk_coord) {
            // Add entity as child of spatial cell
            commands.entity(cell_entity).add_child(entity);
            // Mark as reparented
            commands.entity(entity).insert(SpatiallyParented);
            processed += 1;
        }
    }

    // Log progress periodically (every 10 ticks worth)
    if total_pending <= BUDGET || total_pending % (BUDGET * 10) < BUDGET {
        debug!(
            "Reparenting progress: processed {} entities this tick, {} remaining (with change detection)",
            processed,
            total_pending - processed
        );
    }
}

/// Update spatial parent when entity moves to a different chunk
///
/// Detects when an entity's position changes and reparents it to the new spatial cell.
/// Only processes entities that are already spatially parented.
///
/// Phase 4.2: Movement tracking to maintain correct parent relationships
pub fn update_spatial_parent_on_movement(
    mut commands: Commands,
    grid: Res<SpatialCellGrid>,
    moved: Query<
        (Entity, &crate::entities::movement::TilePosition, &ChildOf),
        (Changed<crate::entities::movement::TilePosition>, With<SpatiallyParented>),
    >,
    cells: Query<&SpatialCell>,
) {
    let mut reparented_count = 0;

    for (entity, pos, child_of) in moved.iter() {
        let new_chunk = grid.chunk_coord_for_position(pos.tile);

        // Check if still in same cell
        let current_parent = child_of.parent();
        if let Ok(cell) = cells.get(current_parent) {
            if cell.chunk_coord == new_chunk {
                continue; // Still in same cell, no reparenting needed
            }
        }

        // Need to reparent to new cell
        if let Some(new_cell_entity) = grid.get_cell(new_chunk) {
            commands.entity(new_cell_entity).add_child(entity);
            reparented_count += 1;
        }
    }

    // Log if significant movement occurred
    if reparented_count > 0 {
        trace!(
            "Updated spatial parents for {} moved entities",
            reparented_count
        );
    }
}

// ============================================================================
// QUERY HELPERS - Children-based spatial queries
// ============================================================================

/// Query entities within radius using Bevy Children component
///
/// This replaces HashMap-based SpatialEntityIndex queries with hierarchical
/// Parent/Child queries for O(k) performance where k = nearby chunks.
///
/// Phase 4.3: Children component-based spatial queries
///
/// # Arguments
/// * `grid` - SpatialCellGrid resource for chunk lookups
/// * `cells` - Query for Children components on SpatialCell entities
/// * `center` - Center position in world coordinates
/// * `radius` - Search radius in tiles
///
/// # Returns
/// Vec of all entities that are children of spatial cells within the radius
pub fn entities_in_radius_via_children(
    grid: &SpatialCellGrid,
    cells: &Query<&Children, With<SpatialCell>>,
    center: IVec2,
    radius: f32,
) -> Vec<Entity> {
    let radius_chunks = (radius / grid.chunk_size() as f32).ceil() as i32;
    let center_chunk = grid.chunk_coord_for_position(center);

    let mut result = Vec::new();

    // Query nearby chunks within radius
    for dx in -radius_chunks..=radius_chunks {
        for dy in -radius_chunks..=radius_chunks {
            let chunk = center_chunk + IVec2::new(dx, dy);

            if let Some(cell_entity) = grid.get_cell(chunk) {
                // Query Children component
                if let Ok(children) = cells.get(cell_entity) {
                    // Iterate through children and add to results
                    // Children.iter() yields Entity directly, not &Entity
                    for child in children.iter() {
                        result.push(child);
                    }
                }
            }
        }
    }

    result
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_coord_for_position_examples() {
        let grid = SpatialCellGrid::new(CHUNK_SIZE);

        // Origin
        assert_eq!(
            grid.chunk_coord_for_position(IVec2::new(0, 0)),
            IVec2::new(0, 0)
        );

        // Within first chunk
        assert_eq!(
            grid.chunk_coord_for_position(IVec2::new(15, 15)),
            IVec2::new(0, 0)
        );

        // Boundary
        assert_eq!(
            grid.chunk_coord_for_position(IVec2::new(16, 16)),
            IVec2::new(1, 1)
        );

        // Negative coordinates
        assert_eq!(
            grid.chunk_coord_for_position(IVec2::new(-1, -1)),
            IVec2::new(-1, -1)
        );
    }

    #[test]
    fn test_is_in_bounds() {
        let grid = SpatialCellGrid::new(CHUNK_SIZE);

        // In bounds
        assert!(grid.is_in_bounds(IVec2::new(0, 0)));
        assert!(grid.is_in_bounds(IVec2::new(-32, -32)));
        assert!(grid.is_in_bounds(IVec2::new(31, 31)));

        // Out of bounds
        assert!(!grid.is_in_bounds(IVec2::new(-33, 0)));
        assert!(!grid.is_in_bounds(IVec2::new(32, 0)));
        assert!(!grid.is_in_bounds(IVec2::new(0, 32)));
    }
}
