# Spatial Cell Quick Reference

**Phase**: 4.1 Complete
**Status**: Production-ready infrastructure
**Purpose**: Foundation for Parent/Child spatial hierarchy migration

---

## Quick Usage

```rust
use life_simulator::entities::{SpatialCell, SpatialCellGrid, CHUNK_SIZE};

// Get spatial grid resource
let grid = world.get_resource::<SpatialCellGrid>().unwrap();

// Convert world position to chunk coordinate
let world_pos = IVec2::new(100, 100);
let chunk_coord = grid.chunk_coord_for_position(world_pos);

// Get cell entity for that chunk
if let Some(cell_entity) = grid.get_cell(chunk_coord) {
    // Use cell entity (e.g., query children in Phase 4.2+)
}

// Query all spatial cells
let cells = world.query::<&SpatialCell>();
```

---

## Key Components

### SpatialCell (Component)
Marker component for spatial grid cell entities.

```rust
#[derive(Component)]
pub struct SpatialCell {
    pub chunk_coord: IVec2,  // Which chunk this cell represents
}
```

### SpatialCellGrid (Resource)
O(1) lookup from chunk coordinates to cell entities.

```rust
#[derive(Resource)]
pub struct SpatialCellGrid {
    // Internal HashMap
}

impl SpatialCellGrid {
    pub fn get_cell(&self, chunk_coord: IVec2) -> Option<Entity>;
    pub fn chunk_coord_for_position(&self, world_pos: IVec2) -> IVec2;
    pub fn chunk_size(&self) -> i32;
    pub fn cell_count(&self) -> usize;
    pub fn is_in_bounds(&self, chunk_coord: IVec2) -> bool;
}
```

---

## Grid Specifications

- **Chunk Size**: 16 tiles
- **Grid Range**: -32 to +32 chunks (64×64 = 4096 cells)
- **World Coverage**: -512 to +512 tiles (1024×1024)
- **Performance**: O(1) lookups via HashMap

---

## Example Queries (Phase 4.2+ Preview)

```rust
// Phase 4.2+: Query entities in a specific cell
fn query_entities_in_cell(
    grid: Res<SpatialCellGrid>,
    cells: Query<&Children, With<SpatialCell>>,
    entities: Query<&Species>,
) {
    let chunk_coord = grid.chunk_coord_for_position(IVec2::new(100, 100));

    if let Some(cell_entity) = grid.get_cell(chunk_coord) {
        if let Ok(children) = cells.get(cell_entity) {
            for &child in children.iter() {
                if let Ok(species) = entities.get(child) {
                    // Process entity
                }
            }
        }
    }
}
```

---

## Current Status

**Phase 4.1**: Infrastructure only - cells spawned, no Parent/Child yet
**Phase 4.2+**: Will add Parent/Child relationships and migrate queries

**Tests**: 8/8 passing, 276/276 library tests passing
**Performance**: 10 TPS maintained, no regressions

---

## Files

- `src/entities/spatial_cell.rs` - Implementation
- `tests/spatial_hierarchy_test.rs` - Test suite
- `PHASE4.1_DELIVERY_REPORT.md` - Full delivery report
