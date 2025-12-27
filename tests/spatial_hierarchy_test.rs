// ============================================================================
// TDD Tests for Phase 4.1: SpatialCell Component Infrastructure
// ============================================================================
//
// RED PHASE: These tests should FAIL initially
//
// Tests verify:
// 1. SpatialCell component exists
// 2. SpatialCellGrid resource exists
// 3. Grid contains 4096 cell entities (64x64)
// 4. chunk_coord_for_position() works correctly
// 5. get_cell() returns correct cell entity
//
// ============================================================================

use bevy::prelude::*;
use life_simulator::entities::*;

const CHUNK_SIZE: i32 = 16;
const GRID_SIZE: i32 = 64; // -32 to +32 in both axes
const EXPECTED_CELL_COUNT: usize = 4096; // 64x64

// ============================================================================
// TEST 1: SpatialCell Component Exists
// ============================================================================
#[test]
fn test_spatial_cell_component_exists() {
    let mut app = App::new();

    // Spawn entity with SpatialCell component
    let chunk_coord = IVec2::new(5, 10);
    let entity = app.world_mut().spawn(SpatialCell { chunk_coord }).id();

    // Verify component exists
    let spatial_cell = app.world().entity(entity).get::<SpatialCell>();
    assert!(spatial_cell.is_some(), "SpatialCell component should exist");
    assert_eq!(spatial_cell.unwrap().chunk_coord, chunk_coord);
}

// ============================================================================
// TEST 2: SpatialCellGrid Resource Exists and Has Correct Configuration
// ============================================================================
#[test]
fn test_spatial_cell_grid_resource() {
    let mut app = App::new();

    // Create grid resource
    let grid = SpatialCellGrid::new(CHUNK_SIZE);
    app.insert_resource(grid);

    // Verify resource exists
    let grid = app.world().get_resource::<SpatialCellGrid>();
    assert!(grid.is_some(), "SpatialCellGrid resource should exist");
    assert_eq!(grid.unwrap().chunk_size(), CHUNK_SIZE);
}

// ============================================================================
// TEST 3: chunk_coord_for_position() Works Correctly
// ============================================================================
#[test]
fn test_chunk_coord_for_position() {
    let grid = SpatialCellGrid::new(CHUNK_SIZE);

    // Test origin (0, 0) → chunk (0, 0)
    assert_eq!(
        grid.chunk_coord_for_position(IVec2::new(0, 0)),
        IVec2::new(0, 0)
    );

    // Test within chunk 0,0
    assert_eq!(
        grid.chunk_coord_for_position(IVec2::new(15, 15)),
        IVec2::new(0, 0)
    );

    // Test boundary - next chunk
    assert_eq!(
        grid.chunk_coord_for_position(IVec2::new(16, 16)),
        IVec2::new(1, 1)
    );

    // Test negative coordinates
    assert_eq!(
        grid.chunk_coord_for_position(IVec2::new(-1, -1)),
        IVec2::new(-1, -1)
    );

    // Test large coordinates
    assert_eq!(
        grid.chunk_coord_for_position(IVec2::new(100, 100)),
        IVec2::new(6, 6)
    );

    // Test mixed positive/negative
    assert_eq!(
        grid.chunk_coord_for_position(IVec2::new(-50, 50)),
        IVec2::new(-4, 3)
    );
}

// ============================================================================
// TEST 4: get_cell() Returns Correct Cell Entity
// ============================================================================
#[test]
fn test_get_cell_lookup() {
    let mut app = App::new();
    let mut grid = SpatialCellGrid::new(CHUNK_SIZE);

    // Manually insert a few cells
    let chunk_0_0 = IVec2::new(0, 0);
    let chunk_5_10 = IVec2::new(5, 10);
    let chunk_neg = IVec2::new(-3, -7);

    let entity_0_0 = app.world_mut().spawn(SpatialCell { chunk_coord: chunk_0_0 }).id();
    let entity_5_10 = app.world_mut().spawn(SpatialCell { chunk_coord: chunk_5_10 }).id();
    let entity_neg = app.world_mut().spawn(SpatialCell { chunk_coord: chunk_neg }).id();

    grid.insert_cell(chunk_0_0, entity_0_0);
    grid.insert_cell(chunk_5_10, entity_5_10);
    grid.insert_cell(chunk_neg, entity_neg);

    // Verify lookups
    assert_eq!(grid.get_cell(chunk_0_0), Some(entity_0_0));
    assert_eq!(grid.get_cell(chunk_5_10), Some(entity_5_10));
    assert_eq!(grid.get_cell(chunk_neg), Some(entity_neg));

    // Non-existent cell
    assert_eq!(grid.get_cell(IVec2::new(99, 99)), None);
}

// ============================================================================
// TEST 5: Spawn System Creates 4096 Cell Entities
// ============================================================================
#[test]
fn test_spawn_spatial_grid_creates_all_cells() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Add spawn system
    app.add_systems(Startup, spawn_spatial_grid);

    // Run startup
    app.update();

    // Verify grid resource exists
    let grid = app.world().get_resource::<SpatialCellGrid>();
    assert!(grid.is_some(), "SpatialCellGrid should be initialized");

    let grid = grid.unwrap();
    assert_eq!(grid.cell_count(), EXPECTED_CELL_COUNT, "Should have 4096 cells");

    // Verify cells exist as entities
    let mut query = app.world_mut().query::<&SpatialCell>();
    let cell_count = query.iter(app.world()).count();
    assert_eq!(cell_count, EXPECTED_CELL_COUNT, "Should have 4096 SpatialCell entities");
}

// ============================================================================
// TEST 6: Grid Coverage is Correct (-32 to +32)
// ============================================================================
#[test]
fn test_grid_coverage() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Startup, spawn_spatial_grid);
    app.update();

    let grid = app.world().get_resource::<SpatialCellGrid>().unwrap();

    // Check corners
    assert!(grid.get_cell(IVec2::new(-32, -32)).is_some(), "Should have cell at (-32, -32)");
    assert!(grid.get_cell(IVec2::new(31, 31)).is_some(), "Should have cell at (31, 31)");
    assert!(grid.get_cell(IVec2::new(-32, 31)).is_some(), "Should have cell at (-32, 31)");
    assert!(grid.get_cell(IVec2::new(31, -32)).is_some(), "Should have cell at (31, -32)");

    // Check center
    assert!(grid.get_cell(IVec2::new(0, 0)).is_some(), "Should have cell at (0, 0)");

    // Check out of bounds
    assert!(grid.get_cell(IVec2::new(-33, 0)).is_none(), "Should NOT have cell at (-33, 0)");
    assert!(grid.get_cell(IVec2::new(32, 0)).is_none(), "Should NOT have cell at (32, 0)");
}

// ============================================================================
// TEST 7: World Position to Cell Lookup Integration
// ============================================================================
#[test]
fn test_world_position_to_cell_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Startup, spawn_spatial_grid);
    app.update();

    let grid = app.world().get_resource::<SpatialCellGrid>().unwrap();

    // World position (100, 100) should map to chunk (6, 6)
    let world_pos = IVec2::new(100, 100);
    let chunk_coord = grid.chunk_coord_for_position(world_pos);
    assert_eq!(chunk_coord, IVec2::new(6, 6));

    let cell_entity = grid.get_cell(chunk_coord);
    assert!(cell_entity.is_some(), "Should find cell for world position (100, 100)");

    // Verify the cell has correct chunk_coord
    let cell = app.world().entity(cell_entity.unwrap()).get::<SpatialCell>();
    assert!(cell.is_some());
    assert_eq!(cell.unwrap().chunk_coord, chunk_coord);
}

// ============================================================================
// TEST 8: Grid Performance - O(1) Lookups
// ============================================================================
#[test]
fn test_grid_lookup_performance() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Startup, spawn_spatial_grid);
    app.update();

    let grid = app.world().get_resource::<SpatialCellGrid>().unwrap();

    // Perform 1000 lookups - should be instant (O(1))
    for i in 0..1000 {
        let x = (i % 64) - 32;
        let y = (i / 64) - 32;
        let chunk_coord = IVec2::new(x, y);

        if x >= -32 && x < 32 && y >= -32 && y < 32 {
            assert!(grid.get_cell(chunk_coord).is_some());
        }
    }
}

// ============================================================================
// Phase 4.2 Tests: Budget-Controlled Reparenting System
// ============================================================================

// ============================================================================
// TEST 9: SpatiallyParented Marker Exists
// ============================================================================
#[test]
fn test_spatially_parented_marker_exists() {
    use life_simulator::entities::SpatiallyParented;

    let mut app = App::new();

    // Spawn entity with SpatiallyParented marker
    let entity = app.world_mut().spawn(SpatiallyParented).id();

    // Verify component exists
    let marker = app.world().entity(entity).get::<SpatiallyParented>();
    assert!(marker.is_some(), "SpatiallyParented marker should exist");
}

// ============================================================================
// TEST 10: Reparenting Budget is Respected
// ============================================================================
#[test]
fn test_reparenting_respects_budget() {
    use life_simulator::entities::{reparent_entities_to_cells, TilePosition};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Startup, spawn_spatial_grid);
    app.update();

    // Spawn 200 entities (more than budget of 50)
    for i in 0..200 {
        let pos = IVec2::new(i % 20, i / 20);
        app.world_mut().spawn(TilePosition::from_tile(pos));
    }

    // Run reparenting system once
    app.add_systems(Update, reparent_entities_to_cells);
    app.update();

    // Check how many got reparented (should be ~50, the budget)
    use life_simulator::entities::SpatiallyParented;
    let mut query = app.world_mut().query::<&SpatiallyParented>();
    let reparented_count = query.iter(app.world()).count();

    // Should process exactly 50 entities (the budget)
    assert!(reparented_count >= 45 && reparented_count <= 55,
            "Should reparent ~50 entities per tick, got {}", reparented_count);
}

// ============================================================================
// TEST 11: Entities Become Children of Correct Cells
// ============================================================================
#[test]
fn test_entities_parented_to_correct_cells() {
    use life_simulator::entities::{reparent_entities_to_cells, TilePosition};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Startup, spawn_spatial_grid);
    app.update();

    // Spawn entity at position (0, 0) - should be in chunk (0, 0)
    let test_entity = app.world_mut().spawn(TilePosition::from_tile(IVec2::new(0, 0))).id();

    // Run reparenting system
    app.add_systems(Update, reparent_entities_to_cells);
    app.update();

    // Check that entity now has a parent
    let child_of = app.world().entity(test_entity).get::<ChildOf>();
    assert!(child_of.is_some(), "Entity should have ChildOf component");

    // Check that parent is the correct spatial cell (chunk 0,0)
    let parent_entity = child_of.unwrap().parent();
    let cell = app.world().entity(parent_entity).get::<SpatialCell>();
    assert!(cell.is_some(), "Parent should be a SpatialCell");
    assert_eq!(cell.unwrap().chunk_coord, IVec2::new(0, 0), "Should be parented to chunk (0,0)");
}

// ============================================================================
// TEST 12: Reparenting Progresses Over Multiple Ticks
// ============================================================================
#[test]
fn test_reparenting_progresses_over_ticks() {
    use life_simulator::entities::{reparent_entities_to_cells, TilePosition, SpatiallyParented};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Startup, spawn_spatial_grid);
    app.update();

    // Spawn 150 entities (3x budget)
    for i in 0..150 {
        let pos = IVec2::new(i % 20, i / 20);
        app.world_mut().spawn(TilePosition::from_tile(pos));
    }

    app.add_systems(Update, reparent_entities_to_cells);

    // After first tick: ~50 entities reparented
    app.update();
    let mut query = app.world_mut().query::<&SpatiallyParented>();
    let count_tick1 = query.iter(app.world()).count();
    assert!(count_tick1 >= 45 && count_tick1 <= 55, "First tick should process ~50");

    // After second tick: ~100 entities reparented
    app.update();
    let count_tick2 = query.iter(app.world()).count();
    assert!(count_tick2 >= 95 && count_tick2 <= 105, "Second tick should process ~100 total");

    // After third tick: all 150 entities reparented
    app.update();
    let count_tick3 = query.iter(app.world()).count();
    assert_eq!(count_tick3, 150, "Third tick should process all 150");
}

// ============================================================================
// TEST 13: Update Spatial Parent on Movement
// ============================================================================
#[test]
fn test_update_spatial_parent_on_movement() {
    use life_simulator::entities::{
        reparent_entities_to_cells, update_spatial_parent_on_movement,
        TilePosition, SpatiallyParented
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Startup, spawn_spatial_grid);
    app.update();

    // Spawn entity at (0, 0) - chunk (0, 0)
    let test_entity = app.world_mut().spawn(TilePosition::from_tile(IVec2::new(0, 0))).id();

    // Reparent it
    app.add_systems(Update, reparent_entities_to_cells);
    app.update();

    // Verify it's in chunk (0, 0)
    let parent1 = app.world().entity(test_entity).get::<ChildOf>().unwrap().parent();
    let cell1 = app.world().entity(parent1).get::<SpatialCell>().unwrap();
    assert_eq!(cell1.chunk_coord, IVec2::new(0, 0));

    // Move entity to (100, 100) - chunk (6, 6)
    app.world_mut().entity_mut(test_entity).get_mut::<TilePosition>().unwrap().tile = IVec2::new(100, 100);

    // Add movement tracking system
    app.add_systems(Update, update_spatial_parent_on_movement);
    app.update();

    // Verify it's now in chunk (6, 6)
    let parent2 = app.world().entity(test_entity).get::<ChildOf>().unwrap().parent();
    let cell2 = app.world().entity(parent2).get::<SpatialCell>().unwrap();
    assert_eq!(cell2.chunk_coord, IVec2::new(6, 6), "Should be reparented to new chunk");
}

// ============================================================================
// TEST 14: No Duplicate Reparenting
// ============================================================================
#[test]
fn test_no_duplicate_reparenting() {
    use life_simulator::entities::{reparent_entities_to_cells, TilePosition, SpatiallyParented};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Startup, spawn_spatial_grid);
    app.update();

    // Spawn 50 entities
    for i in 0..50 {
        let pos = IVec2::new(i % 10, i / 10);
        app.world_mut().spawn(TilePosition::from_tile(pos));
    }

    app.add_systems(Update, reparent_entities_to_cells);

    // First tick - all should be reparented
    app.update();
    let mut query = app.world_mut().query::<&SpatiallyParented>();
    let count1 = query.iter(app.world()).count();

    // Second tick - should not reparent again
    app.update();
    let count2 = query.iter(app.world()).count();

    assert_eq!(count1, count2, "Should not reparent entities that already have SpatiallyParented marker");
}
