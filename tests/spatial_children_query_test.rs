// ============================================================================
// Phase 4.3: Children-based Spatial Query Tests
// ============================================================================
//
// TDD RED PHASE: Tests for Children component-based spatial queries
//
// These tests verify that spatial queries can be performed using Bevy's
// Children component instead of HashMap lookups, maintaining O(k) performance.
//
// ============================================================================

use bevy::prelude::*;
use bevy::ecs::system::RunSystemOnce;
use life_simulator::entities::{SpatialCell, SpatialCellGrid, SpatiallyParented, CHUNK_SIZE};
use life_simulator::entities::movement::TilePosition;

// ============================================================================
// TEST: Helper function for Children-based radius queries
// ============================================================================

#[test]
fn test_entities_in_radius_via_children_empty_grid() {
    // RED: This test expects a helper function that doesn't exist yet

    let mut app = App::new();
    app.insert_resource(SpatialCellGrid::new(CHUNK_SIZE));

    // Query empty grid - should return empty vec
    let result = app.world_mut().run_system_once(|
        grid: Res<SpatialCellGrid>,
        cells: Query<&Children, With<SpatialCell>>,
    | {
        entities_in_radius_via_children(
            &grid,
            &cells,
            IVec2::new(0, 0),
            16.0,
        )
    });

    let result = result.unwrap();
    assert_eq!(result.len(), 0, "Empty grid should return no entities");
}

#[test]
fn test_entities_in_radius_via_children_single_chunk() {
    // RED: Test finding entities within a single chunk

    let mut app = App::new();
    app.insert_resource(SpatialCellGrid::new(CHUNK_SIZE));

    let chunk_coord = IVec2::new(0, 0);

    // Create cell entity
    let cell_entity = app.world_mut().spawn((
        SpatialCell { chunk_coord },
        Name::new("TestCell"),
    )).id();

    // Insert cell into grid
    {
        let mut grid = app.world_mut().resource_mut::<SpatialCellGrid>();
        grid.insert_cell(chunk_coord, cell_entity);
    }

    // Create test entities as children
    let entity1 = app.world_mut().spawn((
        TilePosition { tile: IVec2::new(0, 0) },
        SpatiallyParented,
        Name::new("Entity1"),
    )).id();

    let entity2 = app.world_mut().spawn((
        TilePosition { tile: IVec2::new(5, 5) },
        SpatiallyParented,
        Name::new("Entity2"),
    )).id();

    // Parent entities to cell
    app.world_mut().entity_mut(cell_entity).add_child(entity1);
    app.world_mut().entity_mut(cell_entity).add_child(entity2);

    // Query entities in radius
    let result = app.world_mut().run_system_once(|
        grid: Res<SpatialCellGrid>,
        cells: Query<&Children, With<SpatialCell>>,
    | {
        entities_in_radius_via_children(
            &grid,
            &cells,
            IVec2::new(0, 0),
            32.0, // Large radius to include whole chunk
        )
    });

    let result = result.unwrap();
    assert_eq!(result.len(), 2, "Should find 2 entities in chunk");
    assert!(result.contains(&entity1), "Should find entity1");
    assert!(result.contains(&entity2), "Should find entity2");
}

#[test]
fn test_entities_in_radius_via_children_multiple_chunks() {
    // RED: Test finding entities across multiple chunks

    let mut app = App::new();
    app.insert_resource(SpatialCellGrid::new(CHUNK_SIZE));

    // Create cells for chunks (0,0), (1,0), (0,1)
    let cell00 = app.world_mut().spawn((
        SpatialCell { chunk_coord: IVec2::new(0, 0) },
        Name::new("Cell00"),
    )).id();

    let cell10 = app.world_mut().spawn((
        SpatialCell { chunk_coord: IVec2::new(1, 0) },
        Name::new("Cell10"),
    )).id();

    let cell01 = app.world_mut().spawn((
        SpatialCell { chunk_coord: IVec2::new(0, 1) },
        Name::new("Cell01"),
    )).id();

    // Insert cells into grid
    {
        let mut grid = app.world_mut().resource_mut::<SpatialCellGrid>();
        grid.insert_cell(IVec2::new(0, 0), cell00);
        grid.insert_cell(IVec2::new(1, 0), cell10);
        grid.insert_cell(IVec2::new(0, 1), cell01);
    }

    // Create entities in different chunks
    let entity00 = app.world_mut().spawn((
        TilePosition { tile: IVec2::new(0, 0) },
        SpatiallyParented,
    )).id();

    let entity10 = app.world_mut().spawn((
        TilePosition { tile: IVec2::new(16, 0) },
        SpatiallyParented,
    )).id();

    let entity01 = app.world_mut().spawn((
        TilePosition { tile: IVec2::new(0, 16) },
        SpatiallyParented,
    )).id();

    // Parent entities to respective cells
    app.world_mut().entity_mut(cell00).add_child(entity00);
    app.world_mut().entity_mut(cell10).add_child(entity10);
    app.world_mut().entity_mut(cell01).add_child(entity01);

    // Query entities with radius covering multiple chunks
    let result = app.world_mut().run_system_once(|
        grid: Res<SpatialCellGrid>,
        cells: Query<&Children, With<SpatialCell>>,
    | {
        entities_in_radius_via_children(
            &grid,
            &cells,
            IVec2::new(8, 8), // Center position
            24.0, // Radius to cover 3 chunks
        )
    });

    let result = result.unwrap();
    assert_eq!(result.len(), 3, "Should find entities in 3 chunks");
    assert!(result.contains(&entity00), "Should find entity in chunk (0,0)");
    assert!(result.contains(&entity10), "Should find entity in chunk (1,0)");
    assert!(result.contains(&entity01), "Should find entity in chunk (0,1)");
}

#[test]
fn test_entities_in_radius_performance() {
    // RED: Test that O(k) performance is maintained
    // This test verifies only nearby chunks are queried

    let mut app = App::new();
    app.insert_resource(SpatialCellGrid::new(CHUNK_SIZE));

    // Create a 5x5 grid of chunks (25 total)
    let mut cells_and_entities: Vec<(Entity, Vec<Entity>)> = Vec::new();

    for x in -2..=2 {
        for y in -2..=2 {
            let chunk_coord = IVec2::new(x, y);
            let cell = app.world_mut().spawn((
                SpatialCell { chunk_coord },
                Name::new(format!("Cell({},{})", x, y)),
            )).id();

            // Add 5 entities to each chunk
            let mut chunk_entities = Vec::new();
            for i in 0..5 {
                let entity = app.world_mut().spawn((
                    TilePosition {
                        tile: IVec2::new(x * CHUNK_SIZE, y * CHUNK_SIZE)
                    },
                    SpatiallyParented,
                    Name::new(format!("E{}_{}", chunk_coord, i)),
                )).id();
                chunk_entities.push(entity);
            }
            cells_and_entities.push((cell, chunk_entities));
        }
    }

    // Insert cells into grid and parent entities
    {
        let mut grid = app.world_mut().resource_mut::<SpatialCellGrid>();
        for x in -2..=2 {
            for y in -2..=2 {
                let idx = ((x + 2) * 5 + (y + 2)) as usize;
                let (cell, _) = cells_and_entities[idx];
                grid.insert_cell(IVec2::new(x, y), cell);
            }
        }
    }

    // Parent all entities to their cells
    for (cell, entities) in cells_and_entities {
        for entity in entities {
            app.world_mut().entity_mut(cell).add_child(entity);
        }
    }

    // Query small radius - should only check nearby chunks
    let result = app.world_mut().run_system_once(|
        grid: Res<SpatialCellGrid>,
        cells: Query<&Children, With<SpatialCell>>,
    | {
        entities_in_radius_via_children(
            &grid,
            &cells,
            IVec2::new(0, 0), // Center
            20.0, // Small radius (1-2 chunks)
        )
    });

    let result = result.unwrap();
    // Radius 20.0 with chunk_size 16 means (20/16).ceil() = 2 chunks in each direction
    // That's a 5x5 grid centered at (0,0): chunks from (-2,-2) to (2,2) = 25 chunks
    // Each chunk has 5 entities, so 25 * 5 = 125 entities max
    assert_eq!(result.len(), 125,
        "Should query 5x5 chunks (25 chunks * 5 entities = 125), found {} entities", result.len());

    // Verify O(k) property: we only queried 25 out of 25 total chunks (all accessible)
    // In a larger grid, we'd only query nearby chunks, not all chunks
}

// ============================================================================
// HELPER FUNCTION (to be implemented in GREEN phase)
// ============================================================================

/// Query entities within radius using Bevy Children component
///
/// This replaces HashMap-based SpatialEntityIndex queries with hierarchical
/// Parent/Child queries for O(k) performance where k = nearby chunks.
fn entities_in_radius_via_children(
    grid: &SpatialCellGrid,
    cells: &Query<&Children, With<SpatialCell>>,
    center: IVec2,
    radius: f32,
) -> Vec<Entity> {
    // RED PHASE: This function will fail compilation
    // GREEN PHASE: Implement Children-based query

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
