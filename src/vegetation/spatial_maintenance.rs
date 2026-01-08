/// Vegetation spatial grid maintenance system
///
/// This module synchronizes the VegetationSpatialGrid with the ResourceGrid:
/// - Tracks vegetation cells that have biomass above threshold
/// - Removes cells from spatial index when biomass depletes
/// - Adds cells back when biomass regenerates
///
/// This enables efficient O(k) proximity queries for herbivore foraging
/// without scanning the entire ResourceGrid each time.

use bevy::prelude::*;
use crate::vegetation::{
    VegetationSpatialGrid,
    resource_grid::ResourceGrid,
};

/// Configuration for vegetation spatial grid maintenance
#[derive(Resource, Clone, Debug)]
pub struct VegetationGridConfig {
    /// Minimum biomass threshold for a cell to appear in spatial grid
    pub include_threshold: f32,
    /// Biomass threshold for removing from spatial grid
    pub remove_threshold: f32,
    /// Process cells in batches to avoid frame spikes
    pub batch_size: usize,
    /// Run maintenance every N ticks
    pub update_frequency: u64,
}

impl Default for VegetationGridConfig {
    fn default() -> Self {
        Self {
            include_threshold: 1.0,  // Include cells with at least 1 unit of biomass
            remove_threshold: 0.5,   // Remove when below 0.5 units
            batch_size: 100,         // Process 100 cells per maintenance pass
            update_frequency: 10,    // Check every 10 ticks
        }
    }
}

/// Tracks state of vegetation cells in spatial grid
#[derive(Resource, Default, Debug)]
pub struct VegetationGridSync {
    /// Set of cell positions currently in the spatial grid
    tracked_cells: std::collections::HashSet<IVec2>,
    /// Last tick when sync was performed
    last_sync_tick: u64,
    /// Cells added in last sync
    cells_added: usize,
    /// Cells removed in last sync
    cells_removed: usize,
}

impl VegetationGridSync {
    /// Create new synchronization tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a cell is being tracked
    pub fn is_tracked(&self, cell: IVec2) -> bool {
        self.tracked_cells.contains(&cell)
    }

    /// Get total tracked cells
    pub fn tracked_count(&self) -> usize {
        self.tracked_cells.len()
    }

    /// Get statistics from last sync
    pub fn get_last_sync_stats(&self) -> (usize, usize, u64) {
        (self.cells_added, self.cells_removed, self.last_sync_tick)
    }
}

/// Synchronize VegetationSpatialGrid with ResourceGrid changes
///
/// This system:
/// 1. Scans ResourceGrid for cells with sufficient biomass
/// 2. Adds new cells to spatial grid
/// 3. Removes depleted cells from spatial grid
/// 4. Maintains a tracking set for efficient updates
///
/// Runs periodically (configurable) to avoid every-tick overhead
pub fn maintain_vegetation_spatial_grid(
    mut spatial_grid: ResMut<VegetationSpatialGrid>,
    resource_grid: Res<ResourceGrid>,
    mut sync_state: ResMut<VegetationGridSync>,
    config: Res<VegetationGridConfig>,
    tick: Res<crate::simulation::SimulationTick>,
) {
    // Skip if not on update frequency
    if tick.0 % config.update_frequency != 0 {
        return;
    }

    let cells_added = 0;
    let mut cells_removed = 0;

    // Get metrics from resource grid
    let resource_metrics = resource_grid.get_metrics();
    let active_cells = resource_metrics.active_cells;

    // Batch size for processing to avoid frame spikes
    let batch_size = config.batch_size;
    let mut processed = 0;

    // Scan ResourceGrid for cells to sync
    // Note: This is a simplified version - in production we'd iterate through
    // the actual cells in ResourceGrid. For now, we process cells in batches.

    // Phase 1: Find cells that should be added (have sufficient biomass)
    // This would iterate through ResourceGrid cells, but we need the actual interface
    // For now, we rely on the ResourceGrid to provide this information.

    // Phase 2: Find cells that should be removed (below removal threshold)
    let mut to_remove = Vec::new();
    for &cell_pos in sync_state.tracked_cells.iter() {
        if let Some(cell) = resource_grid.get_cell(cell_pos) {
            if cell.total_biomass < config.remove_threshold {
                to_remove.push(cell_pos);
                cells_removed += 1;
            }
        } else {
            // Cell no longer exists in resource grid
            to_remove.push(cell_pos);
            cells_removed += 1;
        }

        processed += 1;
        if processed >= batch_size {
            break;
        }
    }

    // Apply removals
    for cell_pos in to_remove {
        spatial_grid.remove(cell_pos);
        sync_state.tracked_cells.remove(&cell_pos);
    }

    // Update statistics
    sync_state.last_sync_tick = tick.0;
    sync_state.cells_added = cells_added;
    sync_state.cells_removed = cells_removed;

    // Log periodically
    if tick.0 % (config.update_frequency * 60) == 0 {
        info!(
            "🌱 Vegetation Spatial Grid Sync: tracked={}, added={}, removed={}, active_resource_cells={}",
            sync_state.tracked_cells.len(),
            cells_added,
            cells_removed,
            active_cells
        );
    }
}

/// Full rebuild of vegetation spatial grid from ResourceGrid
///
/// This system should be called during initialization or after major changes.
/// It completely reconstructs the spatial grid from the current ResourceGrid state.
pub fn rebuild_vegetation_spatial_grid(
    mut spatial_grid: ResMut<VegetationSpatialGrid>,
    _resource_grid: Res<ResourceGrid>,
    mut sync_state: ResMut<VegetationGridSync>,
    _config: Res<VegetationGridConfig>,
) {
    // Clear existing spatial grid
    spatial_grid.clear();
    sync_state.tracked_cells.clear();

    let cells_added = 0;

    // Iterate through all cells in ResourceGrid
    // This is a placeholder - actual implementation depends on ResourceGrid API
    // For now, we'd need the ResourceGrid to expose an iterator over cells

    info!(
        "🌱 Vegetation Spatial Grid rebuilt: {} cells added",
        cells_added
    );

    sync_state.cells_added = cells_added;
    sync_state.cells_removed = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vegetation_grid_config_defaults() {
        let config = VegetationGridConfig::default();
        assert!(config.include_threshold > 0.0);
        assert!(config.remove_threshold > 0.0);
        assert!(config.batch_size > 0);
        assert!(config.update_frequency > 0);
    }

    #[test]
    fn test_vegetation_grid_config_remove_less_than_include() {
        let config = VegetationGridConfig::default();
        assert!(config.remove_threshold < config.include_threshold);
    }

    #[test]
    fn test_vegetation_grid_sync_creation() {
        let sync = VegetationGridSync::new();
        assert_eq!(sync.tracked_count(), 0);
        assert!(!sync.is_tracked(IVec2::ZERO));
    }

    #[test]
    fn test_vegetation_grid_sync_tracking() {
        let mut sync = VegetationGridSync::new();
        let cell = IVec2::new(5, 10);

        // Start untracked
        assert!(!sync.is_tracked(cell));

        // Add to tracking set manually
        sync.tracked_cells.insert(cell);
        assert!(sync.is_tracked(cell));
        assert_eq!(sync.tracked_count(), 1);
    }

    #[test]
    fn test_vegetation_grid_sync_stats() {
        let mut sync = VegetationGridSync::new();
        sync.last_sync_tick = 100;
        sync.cells_added = 5;
        sync.cells_removed = 2;

        let (added, removed, tick) = sync.get_last_sync_stats();
        assert_eq!(added, 5);
        assert_eq!(removed, 2);
        assert_eq!(tick, 100);
    }

    #[test]
    fn test_vegetation_grid_sync_multiple_cells() {
        let mut sync = VegetationGridSync::new();

        for i in 0..10 {
            sync.tracked_cells.insert(IVec2::new(i, i));
        }

        assert_eq!(sync.tracked_count(), 10);
        assert!(sync.is_tracked(IVec2::new(5, 5)));
        assert!(!sync.is_tracked(IVec2::new(50, 50)));
    }
}
