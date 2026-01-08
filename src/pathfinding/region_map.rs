/// Region Map - Connected Components for O(1) Reachability Checks
///
/// Implements the Dwarf Fortress connected components pattern:
/// - Flood-fill assigns unique region IDs to each connected walkable area
/// - O(1) `are_connected(a, b)` check by comparing region IDs
/// - Rebuilt when terrain changes (rare)
///
/// This allows instant rejection of unreachable destinations before
/// expensive A* pathfinding, dramatically improving performance.

use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

use super::PathfindingGrid;

/// Resource: Maps each walkable tile to its connected region ID
///
/// # Performance
/// - Build time: O(n) where n = number of walkable tiles
/// - Memory: ~40 bytes per walkable tile (IVec2 key + u32 value + HashMap overhead)
/// - Lookup: O(1) for `are_connected` checks
///
/// # Usage
/// ```ignore
/// // Before attempting expensive A* pathfinding:
/// if !region_map.are_connected(start, goal) {
///     return None; // Instant rejection - no path possible
/// }
/// // Only run A* if regions match
/// find_path(start, goal, &grid, ...)
/// ```
#[derive(Resource, Default)]
pub struct RegionMap {
    /// Tile position -> Region ID mapping
    /// Only walkable tiles are included
    regions: HashMap<IVec2, u32>,

    /// Total number of distinct regions
    region_count: u32,

    /// Build statistics for debugging
    pub tiles_mapped: usize,
    pub build_time_ms: f64,
}

impl RegionMap {
    /// Create a new empty RegionMap
    pub fn new() -> Self {
        Self {
            regions: HashMap::new(),
            region_count: 0,
            tiles_mapped: 0,
            build_time_ms: 0.0,
        }
    }

    /// O(1) check if two positions are in the same connected region
    ///
    /// Returns `true` if both positions are walkable AND connected.
    /// Returns `false` if either position is unwalkable OR they're in different regions.
    ///
    /// # Example
    /// ```ignore
    /// // Instant rejection for unreachable destinations
    /// if !region_map.are_connected(entity_pos, target_pos) {
    ///     // No path possible - skip A* entirely
    ///     return Err(PathError::Unreachable);
    /// }
    /// ```
    #[inline]
    pub fn are_connected(&self, a: IVec2, b: IVec2) -> bool {
        match (self.regions.get(&a), self.regions.get(&b)) {
            (Some(region_a), Some(region_b)) => region_a == region_b,
            _ => false, // One or both positions not in any region (unwalkable)
        }
    }

    /// Get the region ID for a position (None if unwalkable)
    #[inline]
    pub fn get_region(&self, pos: IVec2) -> Option<u32> {
        self.regions.get(&pos).copied()
    }

    /// Check if a position is in any walkable region
    #[inline]
    pub fn is_mapped(&self, pos: IVec2) -> bool {
        self.regions.contains_key(&pos)
    }

    /// Get total number of distinct connected regions
    #[inline]
    pub fn region_count(&self) -> u32 {
        self.region_count
    }

    /// Get total number of mapped (walkable) tiles
    #[inline]
    pub fn tile_count(&self) -> usize {
        self.regions.len()
    }

    /// Clear all region data (call before rebuild)
    pub fn clear(&mut self) {
        self.regions.clear();
        self.region_count = 0;
        self.tiles_mapped = 0;
        self.build_time_ms = 0.0;
    }

    /// Build region map from PathfindingGrid using flood-fill
    ///
    /// # Algorithm
    /// 1. Iterate all tiles in the grid
    /// 2. For each unvisited walkable tile, start a new region
    /// 3. Flood-fill (BFS) to mark all connected walkable tiles with same region ID
    /// 4. Repeat until all walkable tiles are assigned
    ///
    /// # Complexity
    /// - Time: O(n) where n = total tiles (each tile visited once)
    /// - Space: O(w) where w = walkable tiles
    pub fn build_from_grid(&mut self, grid: &PathfindingGrid, bounds: (IVec2, IVec2)) {
        let start_time = std::time::Instant::now();

        self.clear();

        let (min, max) = bounds;
        let mut current_region: u32 = 0;

        // Iterate all tiles in bounds
        for y in min.y..=max.y {
            for x in min.x..=max.x {
                let pos = IVec2::new(x, y);

                // Skip if already assigned or not walkable
                if self.regions.contains_key(&pos) {
                    continue;
                }

                if !grid.is_walkable(pos) {
                    continue;
                }

                // Found unvisited walkable tile - start new region
                self.flood_fill(pos, current_region, grid);
                current_region += 1;
            }
        }

        self.region_count = current_region;
        self.tiles_mapped = self.regions.len();
        self.build_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
    }

    /// Flood-fill from starting position, marking all connected walkable tiles
    /// Uses 8-directional connectivity with corner-cutting prevention to match pathfinding
    fn flood_fill(&mut self, start: IVec2, region_id: u32, grid: &PathfindingGrid) {
        let mut queue: VecDeque<IVec2> = VecDeque::new();
        queue.push_back(start);

        while let Some(pos) = queue.pop_front() {
            // Skip if already visited
            if self.regions.contains_key(&pos) {
                continue;
            }

            // Skip if not walkable
            if !grid.is_walkable(pos) {
                continue;
            }

            // Mark this tile with the region ID
            self.regions.insert(pos, region_id);

            // Cardinal directions
            let north = pos + IVec2::new(0, 1);
            let east = pos + IVec2::new(1, 0);
            let south = pos + IVec2::new(0, -1);
            let west = pos + IVec2::new(-1, 0);

            // Always add cardinal neighbors
            let mut neighbors = vec![north, east, south, west];

            // Add diagonal neighbors with corner-cutting prevention
            // Only allow diagonal if BOTH adjacent cardinals are walkable (matches pathfinding)

            // NE: requires North AND East to be walkable
            if grid.is_walkable(north) && grid.is_walkable(east) {
                neighbors.push(pos + IVec2::new(1, 1));
            }

            // SE: requires South AND East to be walkable
            if grid.is_walkable(south) && grid.is_walkable(east) {
                neighbors.push(pos + IVec2::new(1, -1));
            }

            // SW: requires South AND West to be walkable
            if grid.is_walkable(south) && grid.is_walkable(west) {
                neighbors.push(pos + IVec2::new(-1, -1));
            }

            // NW: requires North AND West to be walkable
            if grid.is_walkable(north) && grid.is_walkable(west) {
                neighbors.push(pos + IVec2::new(-1, 1));
            }

            for neighbor in neighbors {
                if !self.regions.contains_key(&neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }
    }

    /// Get statistics string for logging
    pub fn stats_string(&self) -> String {
        format!(
            "{} regions, {} tiles, built in {:.2}ms",
            self.region_count, self.tiles_mapped, self.build_time_ms
        )
    }
}

/// System: Build RegionMap after PathfindingGrid is ready
///
/// This should run in Startup schedule after the pathfinding grid is built.
/// The system extracts bounds from the grid and performs flood-fill.
pub fn build_region_map(
    mut region_map: ResMut<RegionMap>,
    grid: Res<PathfindingGrid>,
) {
    info!("🗺️ RegionMap: Building connected components...");

    // Extract bounds from PathfindingGrid
    // We need to determine the actual bounds from the grid's data
    let bounds = extract_grid_bounds(&grid);

    region_map.build_from_grid(&grid, bounds);

    info!(
        "✅ RegionMap: Built successfully - {}",
        region_map.stats_string()
    );

    // Log region distribution for debugging
    if region_map.region_count() > 1 {
        info!(
            "   📊 Multiple disconnected regions detected ({} total)",
            region_map.region_count()
        );
    }
}

/// Extract the actual bounds of the PathfindingGrid
///
/// Since PathfindingGrid uses a HashMap internally, we iterate
/// to find min/max coordinates. This is O(n) but only runs once at startup.
fn extract_grid_bounds(grid: &PathfindingGrid) -> (IVec2, IVec2) {
    // Access the internal costs HashMap through is_walkable checks
    // We'll sample a large range and find actual bounds
    //
    // Note: This is a workaround since PathfindingGrid doesn't expose its keys.
    // In a production system, PathfindingGrid should expose an iter() or bounds() method.

    // For now, we use a reasonable default based on typical world sizes
    // The main.rs shows bounds are calculated from chunk coordinates * 16
    // A typical world is ~256x256 tiles or larger

    // Use environment-based bounds or sensible defaults
    let default_size = 512; // Cover most typical world sizes
    let half = default_size / 2;

    // Check if we can detect actual bounds by sampling
    // This is a heuristic approach
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    // Sample grid in a reasonable range to find actual bounds
    for y in -half..=half {
        for x in -half..=half {
            let pos = IVec2::new(x, y);
            if grid.is_walkable(pos) || grid.get_cost(pos) != u32::MAX {
                // This tile has been set (either walkable or explicitly blocked)
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }

    // If no tiles found, return empty bounds
    if min_x > max_x {
        warn!("RegionMap: No tiles found in PathfindingGrid!");
        return (IVec2::ZERO, IVec2::ZERO);
    }

    // Add small padding to ensure we catch edge tiles
    let padding = 1;
    (
        IVec2::new(min_x - padding, min_y - padding),
        IVec2::new(max_x + padding, max_y + padding),
    )
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_grid(width: i32, height: i32) -> PathfindingGrid {
        let mut grid = PathfindingGrid::new();
        for y in 0..height {
            for x in 0..width {
                grid.set_cost(IVec2::new(x, y), 1); // All walkable
            }
        }
        grid
    }

    #[test]
    fn test_single_connected_region() {
        let grid = create_test_grid(10, 10);
        let mut region_map = RegionMap::new();

        region_map.build_from_grid(&grid, (IVec2::ZERO, IVec2::new(9, 9)));

        assert_eq!(region_map.region_count(), 1);
        assert_eq!(region_map.tile_count(), 100);

        // All tiles should be connected
        assert!(region_map.are_connected(IVec2::new(0, 0), IVec2::new(9, 9)));
        assert!(region_map.are_connected(IVec2::new(5, 5), IVec2::new(0, 0)));
    }

    #[test]
    fn test_two_disconnected_regions() {
        let mut grid = PathfindingGrid::new();

        // Create two separate 3x3 regions
        // Region 1: (0,0) to (2,2)
        for y in 0..3 {
            for x in 0..3 {
                grid.set_cost(IVec2::new(x, y), 1);
            }
        }

        // Region 2: (5,5) to (7,7) - gap of 2 tiles
        for y in 5..8 {
            for x in 5..8 {
                grid.set_cost(IVec2::new(x, y), 1);
            }
        }

        let mut region_map = RegionMap::new();
        region_map.build_from_grid(&grid, (IVec2::ZERO, IVec2::new(9, 9)));

        assert_eq!(region_map.region_count(), 2);
        assert_eq!(region_map.tile_count(), 18); // 9 + 9

        // Tiles within same region should be connected
        assert!(region_map.are_connected(IVec2::new(0, 0), IVec2::new(2, 2)));
        assert!(region_map.are_connected(IVec2::new(5, 5), IVec2::new(7, 7)));

        // Tiles in different regions should NOT be connected
        assert!(!region_map.are_connected(IVec2::new(0, 0), IVec2::new(5, 5)));
        assert!(!region_map.are_connected(IVec2::new(2, 2), IVec2::new(7, 7)));
    }

    #[test]
    fn test_unwalkable_positions_not_connected() {
        let grid = create_test_grid(5, 5);
        let mut region_map = RegionMap::new();
        region_map.build_from_grid(&grid, (IVec2::ZERO, IVec2::new(4, 4)));

        // Position outside grid (not set) should not be connected
        assert!(!region_map.are_connected(IVec2::new(0, 0), IVec2::new(100, 100)));

        // Both unwalkable positions should not be connected
        assert!(!region_map.are_connected(IVec2::new(100, 100), IVec2::new(200, 200)));
    }

    #[test]
    fn test_wall_creates_disconnection() {
        let mut grid = PathfindingGrid::new();

        // Create 10x10 grid with vertical wall at x=5
        for y in 0..10 {
            for x in 0..10 {
                if x == 5 {
                    grid.set_cost(IVec2::new(x, y), u32::MAX); // Wall
                } else {
                    grid.set_cost(IVec2::new(x, y), 1); // Walkable
                }
            }
        }

        let mut region_map = RegionMap::new();
        region_map.build_from_grid(&grid, (IVec2::ZERO, IVec2::new(9, 9)));

        assert_eq!(region_map.region_count(), 2);

        // Left side connected
        assert!(region_map.are_connected(IVec2::new(0, 0), IVec2::new(4, 9)));

        // Right side connected
        assert!(region_map.are_connected(IVec2::new(6, 0), IVec2::new(9, 9)));

        // Left and right NOT connected
        assert!(!region_map.are_connected(IVec2::new(0, 0), IVec2::new(9, 9)));
    }

    #[test]
    fn test_get_region() {
        let grid = create_test_grid(5, 5);
        let mut region_map = RegionMap::new();
        region_map.build_from_grid(&grid, (IVec2::ZERO, IVec2::new(4, 4)));

        // All tiles should have region 0
        assert_eq!(region_map.get_region(IVec2::new(0, 0)), Some(0));
        assert_eq!(region_map.get_region(IVec2::new(4, 4)), Some(0));

        // Outside bounds should be None
        assert_eq!(region_map.get_region(IVec2::new(100, 100)), None);
    }

    #[test]
    fn test_is_mapped() {
        let grid = create_test_grid(5, 5);
        let mut region_map = RegionMap::new();
        region_map.build_from_grid(&grid, (IVec2::ZERO, IVec2::new(4, 4)));

        assert!(region_map.is_mapped(IVec2::new(0, 0)));
        assert!(region_map.is_mapped(IVec2::new(4, 4)));
        assert!(!region_map.is_mapped(IVec2::new(100, 100)));
    }

    #[test]
    fn test_clear() {
        let grid = create_test_grid(5, 5);
        let mut region_map = RegionMap::new();
        region_map.build_from_grid(&grid, (IVec2::ZERO, IVec2::new(4, 4)));

        assert!(region_map.tile_count() > 0);

        region_map.clear();

        assert_eq!(region_map.tile_count(), 0);
        assert_eq!(region_map.region_count(), 0);
    }

    #[test]
    fn test_empty_grid() {
        let grid = PathfindingGrid::new(); // Empty grid
        let mut region_map = RegionMap::new();
        region_map.build_from_grid(&grid, (IVec2::ZERO, IVec2::new(9, 9)));

        assert_eq!(region_map.region_count(), 0);
        assert_eq!(region_map.tile_count(), 0);
    }

    #[test]
    fn test_single_tile_region() {
        let mut grid = PathfindingGrid::new();
        grid.set_cost(IVec2::new(5, 5), 1); // Single walkable tile

        let mut region_map = RegionMap::new();
        region_map.build_from_grid(&grid, (IVec2::ZERO, IVec2::new(9, 9)));

        assert_eq!(region_map.region_count(), 1);
        assert_eq!(region_map.tile_count(), 1);
        assert!(region_map.is_mapped(IVec2::new(5, 5)));
    }

    #[test]
    fn test_diagonal_not_connected() {
        let mut grid = PathfindingGrid::new();

        // Two tiles that are diagonally adjacent but not orthogonally connected
        grid.set_cost(IVec2::new(0, 0), 1);
        grid.set_cost(IVec2::new(1, 1), 1);

        let mut region_map = RegionMap::new();
        region_map.build_from_grid(&grid, (IVec2::ZERO, IVec2::new(2, 2)));

        // Should be 2 separate regions due to corner-cutting prevention:
        // Diagonal from (0,0) to (1,1) requires BOTH (0,1) AND (1,0) to be walkable
        // Since neither adjacent cardinal is walkable, the diagonal is blocked
        assert_eq!(region_map.region_count(), 2);
        assert!(!region_map.are_connected(IVec2::new(0, 0), IVec2::new(1, 1)));
    }

    #[test]
    fn test_diagonal_connected_with_cardinals() {
        let mut grid = PathfindingGrid::new();

        // Create L-shape that allows diagonal movement with corner-cutting prevention
        // . X
        // X X
        grid.set_cost(IVec2::new(0, 0), 1);  // SW corner
        grid.set_cost(IVec2::new(1, 0), 1);  // SE corner (East of SW)
        grid.set_cost(IVec2::new(1, 1), 1);  // NE corner (North of SE)

        let mut region_map = RegionMap::new();
        region_map.build_from_grid(&grid, (IVec2::ZERO, IVec2::new(2, 2)));

        // Should be 1 connected region:
        // (0,0) connects to (1,0) via East
        // (1,0) connects to (1,1) via North
        // (0,0) can also reach (1,1) diagonally because:
        //   - North (0,1) is NOT walkable BUT
        //   - We can go East to (1,0), then North to (1,1)
        //   - Flood-fill will mark all as same region
        assert_eq!(region_map.region_count(), 1);
        assert!(region_map.are_connected(IVec2::new(0, 0), IVec2::new(1, 1)));
    }
}

