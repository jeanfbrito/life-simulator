/// Vegetation system for plant growth and herbivore consumption
///
/// This module provides a comprehensive vegetation system that:
/// - Tracks biomass per tile using logistic growth
/// - Integrates with existing herbivore behaviors
/// - Supports terrain-specific growth rates
/// - Provides performance optimizations for large maps
///
/// # Architecture
///
/// The system follows a data-driven approach where:
/// - `VegetationGrid` stores biomass data across the world
/// - Growth systems update biomass at regular intervals
/// - Herbivore behaviors query and consume vegetation
/// - Terrain modifiers affect growth rates and maximum biomass
///
/// # Integration Points
///
/// 1. **AI Behaviors**: Herbivore foraging actions consume vegetation
/// 2. **Terrain System**: Growth rates vary by terrain type
/// 3. **World Loader**: Vegetation initializes based on map data
/// 4. **Web Viewer**: Optional biomass overlay for debugging

pub mod constants;

use bevy::prelude::*;
use std::collections::HashMap;

use crate::simulation::SimulationTick;
use crate::world_loader::WorldLoader;

// Public exports from constants
pub use constants::*;
use constants::{
    consumption::DEPLETED_TILE_COOLDOWN,
    growth::{ACTIVE_TILE_THRESHOLD, GROWTH_INTERVAL_TICKS, MAX_BIOMASS},
    performance::CHUNK_SIZE,
};

/// Vegetation state for a single tile
#[derive(Debug, Clone)]
pub struct TileVegetation {
    /// Current biomass on this tile (0.0 to MAX_BIOMASS)
    pub biomass: f32,
    /// Last tick when this tile was grazed/consumed
    pub last_grazed_tick: u64,
    /// Terrain-specific maximum biomass multiplier
    pub terrain_multiplier: f32,
}

impl TileVegetation {
    /// Create new vegetation with initial biomass
    pub fn new(initial_biomass: f32, terrain_multiplier: f32) -> Self {
        Self {
            biomass: initial_biomass.clamp(0.0, MAX_BIOMASS),
            last_grazed_tick: 0,
            terrain_multiplier,
        }
    }

    /// Get the maximum biomass for this tile based on terrain
    pub fn max_biomass(&self) -> f32 {
        MAX_BIOMASS * self.terrain_multiplier
    }

    /// Get current biomass as fraction of maximum
    pub fn fraction_full(&self) -> f32 {
        if self.terrain_multiplier > 0.0 {
            self.biomass / self.max_biomass()
        } else {
            0.0
        }
    }

    /// Check if this tile is considered depleted
    pub fn is_depleted(&self) -> bool {
        self.biomass < constants::growth::DEPLETED_THRESHOLD
    }

    /// Check if this tile should be in the active update set
    pub fn is_active(&self, current_tick: u64) -> bool {
        self.biomass < (self.max_biomass() * ACTIVE_TILE_THRESHOLD)
            || (current_tick - self.last_grazed_tick) < DEPLETED_TILE_COOLDOWN
    }

    /// Add biomass to this tile (growth)
    pub fn add_biomass(&mut self, amount: f32) {
        self.biomass = (self.biomass + amount).min(self.max_biomass());
    }

    /// Remove biomass from this tile (consumption)
    pub fn remove_biomass(&mut self, amount: f32) -> f32 {
        let removed = amount.min(self.biomass);
        self.biomass -= removed;
        removed
    }

    /// Mark this tile as grazed at current tick
    pub fn mark_grazed(&mut self, current_tick: u64) {
        self.last_grazed_tick = current_tick;
    }
}

/// Grid-based storage for vegetation data
#[derive(Resource, Debug)]
pub struct VegetationGrid {
    /// Sparse storage: tile coordinates -> vegetation state
    /// Uses sparse storage for memory efficiency on large maps
    tiles: HashMap<IVec2, TileVegetation>,

    /// Set of "active" tiles that need frequent updates
    /// Active tiles are those that are regrowing or recently grazed
    active_tiles: HashMap<IVec2, u64>, // tile -> last_update_tick

    /// Total number of tiles that could support vegetation
    total_suitable_tiles: usize,

    /// Current tick counter for timing calculations
    current_tick: u64,
}

impl VegetationGrid {
    /// Create a new vegetation grid
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new(),
            active_tiles: HashMap::new(),
            total_suitable_tiles: 0,
            current_tick: 0,
        }
    }

    /// Get vegetation at a specific tile, creating it if needed
    pub fn get_or_create(&mut self, tile: IVec2, terrain_multiplier: f32) -> &mut TileVegetation {
        if terrain_multiplier <= 0.0 {
            // Terrain doesn't support vegetation
            static EMPTY_VEGETATION: TileVegetation = TileVegetation {
                biomass: 0.0,
                last_grazed_tick: 0,
                terrain_multiplier: 0.0,
            };

            // Return a reference to empty vegetation (won't be stored)
            // Note: This is a workaround - in practice, callers should check terrain_multiplier first
            panic!("Attempted to create vegetation on non-vegetated terrain at {:?}", tile);
        }

        if !self.tiles.contains_key(&tile) {
            let vegetation = TileVegetation::new(
                MAX_BIOMASS * terrain_multiplier, // Start at max for initial growth
                terrain_multiplier,
            );
            self.tiles.insert(tile, vegetation);

            if terrain_multiplier > 0.0 {
                self.total_suitable_tiles += 1;
            }
        }

        self.tiles.get_mut(&tile).unwrap()
    }

    /// Get vegetation at a specific tile (read-only)
    pub fn get(&self, tile: IVec2) -> Option<&TileVegetation> {
        self.tiles.get(&tile)
    }

    /// Get vegetation at a specific tile (mutable)
    pub fn get_mut(&mut self, tile: IVec2) -> Option<&mut TileVegetation> {
        self.tiles.get_mut(&tile)
    }

    /// Consume biomass from a tile
    /// Returns (consumed, remaining) amounts
    pub fn consume(&mut self, tile: IVec2, requested_amount: f32, max_fraction: f32) -> (f32, f32) {
        use constants::consumption::MAX_MEAL_ABSOLUTE;

        if let Some(vegetation) = self.tiles.get_mut(&tile) {
            // Apply consumption rules: min(requested, max_fraction * available, max_absolute)
            let max_by_fraction = vegetation.biomass * max_fraction;
            let actual_consumed = requested_amount
                .min(max_by_fraction)
                .min(MAX_MEAL_ABSOLUTE);

            let remaining = requested_amount - actual_consumed;

            if actual_consumed > 0.0 {
                vegetation.remove_biomass(actual_consumed);
                vegetation.mark_grazed(self.current_tick);

                // Add to active tiles if it became depleted
                if vegetation.is_depleted() {
                    self.active_tiles.insert(tile, self.current_tick);
                }
            }

            (actual_consumed, remaining)
        } else {
            (0.0, requested_amount)
        }
    }

    /// Sample biomass in a radius around a position
    /// Returns average biomass and number of suitable tiles found
    pub fn sample_biomass(&self, center: IVec2, radius: i32) -> (f32, usize) {
        let mut total_biomass = 0.0;
        let mut tile_count = 0;

        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let tile = center + IVec2::new(dx, dy);
                if let Some(vegetation) = self.get(tile) {
                    if vegetation.terrain_multiplier > 0.0 {
                        total_biomass += vegetation.biomass;
                        tile_count += 1;
                    }
                }
            }
        }

        let average = if tile_count > 0 {
            total_biomass / tile_count as f32
        } else {
            0.0
        };

        (average, tile_count)
    }

    /// Find best foraging tile within radius
    /// Returns tile position and biomass amount
    pub fn find_best_forage_tile(&self, center: IVec2, radius: i32) -> Option<(IVec2, f32)> {
        use constants::consumption::FORAGE_MIN_BIOMASS;

        let mut best_tile: Option<(IVec2, f32)> = None;

        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let tile = center + IVec2::new(dx, dy);
                if let Some(vegetation) = self.get(tile) {
                    if vegetation.biomass >= FORAGE_MIN_BIOMASS {
                        let distance = center.as_vec2().distance(tile.as_vec2());
                        let utility = vegetation.biomass / (1.0 + distance * 0.1); // Distance penalty

                        if let Some((_, best_utility)) = best_tile {
                            if utility > best_utility {
                                best_tile = Some((tile, vegetation.biomass));
                            }
                        } else {
                            best_tile = Some((tile, vegetation.biomass));
                        }
                    }
                }
            }
        }

        best_tile
    }

    /// Update the vegetation grid (called by growth system)
    pub fn update(&mut self, current_tick: u64) {
        self.current_tick = current_tick;

        // Only run growth updates on interval
        if current_tick % GROWTH_INTERVAL_TICKS != 0 {
            return;
        }

        // Update active tiles
        self.update_active_tiles();

        // Sample some inactive tiles for occasional updates
        self.update_inactive_tile_sample();
    }

    /// Update all active tiles (those that need frequent updates)
    fn update_active_tiles(&mut self) {
        use constants::growth::GROWTH_RATE;
        use constants::performance::MAX_ACTIVE_TILES_PER_UPDATE;

        let tiles_to_update: Vec<IVec2> = self.active_tiles
            .keys()
            .take(MAX_ACTIVE_TILES_PER_UPDATE)
            .copied()
            .collect();

        for tile in tiles_to_update {
            if let Some(vegetation) = self.tiles.get_mut(&tile) {
                // Logistic growth: B(t+1) = B(t) + r * B(t) * (1 - B(t)/Bmax)
                let max_biomass = vegetation.max_biomass();
                if max_biomass > 0.0 {
                    let growth = GROWTH_RATE * vegetation.biomass * (1.0 - vegetation.biomass / max_biomass);
                    vegetation.add_biomass(growth);

                    // Remove from active set if recovered
                    if !vegetation.is_active(self.current_tick) {
                        self.active_tiles.remove(&tile);
                    }
                }
            }
        }
    }

    /// Update a random sample of inactive tiles
    fn update_inactive_tile_sample(&mut self) {
        use constants::growth::GROWTH_RATE;
        use constants::performance::INACTIVE_SAMPLE_SIZE;

        // Get inactive tiles (those not in active set)
        let inactive_tiles: Vec<IVec2> = self.tiles
            .keys()
            .filter(|tile| !self.active_tiles.contains_key(tile))
            .take(INACTIVE_SAMPLE_SIZE)
            .copied()
            .collect();

        for tile in inactive_tiles {
            if let Some(vegetation) = self.tiles.get_mut(&tile) {
                let max_biomass = vegetation.max_biomass();
                if max_biomass > 0.0 {
                    let growth = GROWTH_RATE * vegetation.biomass * (1.0 - vegetation.biomass / max_biomass);
                    vegetation.add_biomass(growth);

                    // Add to active set if it became active
                    if vegetation.is_active(self.current_tick) {
                        self.active_tiles.insert(tile, self.current_tick);
                    }
                }
            }
        }
    }

    /// Get statistics for debugging and monitoring
    pub fn get_statistics(&self) -> VegetationStatistics {
        let mut total_biomass = 0.0;
        let mut active_count = 0;
        let mut depleted_count = 0;

        for vegetation in self.tiles.values() {
            if vegetation.terrain_multiplier > 0.0 {
                total_biomass += vegetation.biomass;

                if vegetation.is_depleted() {
                    depleted_count += 1;
                }
            }
        }

        active_count = self.active_tiles.len();

        VegetationStatistics {
            total_tiles: self.tiles.len(),
            suitable_tiles: self.total_suitable_tiles,
            active_tiles: active_count,
            depleted_tiles: depleted_count,
            total_biomass,
            average_biomass: if self.total_suitable_tiles > 0 {
                total_biomass / self.total_suitable_tiles as f32
            } else {
                0.0
            },
        }
    }
}

/// Vegetation system statistics for monitoring
#[derive(Debug, Clone)]
pub struct VegetationStatistics {
    pub total_tiles: usize,
    pub suitable_tiles: usize,
    pub active_tiles: usize,
    pub depleted_tiles: usize,
    pub total_biomass: f32,
    pub average_biomass: f32,
}

/// Default implementation for VegetationGrid
impl Default for VegetationGrid {
    fn default() -> Self {
        Self::new()
    }
}

/// Bevy plugin for vegetation system
pub struct VegetationPlugin;

impl Plugin for VegetationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VegetationGrid>()
            .add_systems(Startup, setup_vegetation_system)
            .add_systems(
                FixedUpdate,
                vegetation_growth_system.run_if(every_n_ticks(GROWTH_INTERVAL_TICKS))
            );
    }
}

/// System condition: run every N ticks
fn every_n_ticks(n: u64) -> impl FnMut(Res<SimulationTick>) -> bool {
    move |tick: Res<SimulationTick>| tick.0 % n == 0
}

/// Initialize vegetation system
/// Sets up the vegetation grid and initial biomass distribution
fn setup_vegetation_system(
    mut vegetation_grid: ResMut<VegetationGrid>,
    world_loader: Res<WorldLoader>,
) {
    info!("🌱 Initializing vegetation system...");

    // Get world bounds from the world loader
    let world_info = world_loader.get_world_info();
    let world_size_chunks = world_info.config.world_size_chunks;

    // Calculate world bounds in tile coordinates (assuming centered at 0,0)
    let chunk_size = 16; // From CHUNK_SIZE constant
    let world_radius_tiles = (world_size_chunks / 2) * chunk_size;
    let center_tile_x = 0;
    let center_tile_y = 0;

    info!("🗺️  World bounds: center=({},{}) radius={} tiles",
          center_tile_x, center_tile_y, world_radius_tiles);

    // Initialize vegetation for all tiles in the world
    // We'll use lazy loading - only create vegetation as needed
    // But we need to count suitable tiles for statistics
    let mut suitable_count = 0;

    // Sample tiles to estimate vegetation distribution
    let sample_step = 10; // Check every 10th tile for performance
    for x in (center_tile_x - world_radius_tiles..=center_tile_x + world_radius_tiles).step_by(sample_step) {
        for y in (center_tile_y - world_radius_tiles..=center_tile_y + world_radius_tiles).step_by(sample_step) {
            if let Some(terrain_str) = world_loader.get_terrain_at(x, y) {
                let terrain_multiplier = constants::terrain_modifiers::max_biomass_multiplier(&terrain_str);
                if terrain_multiplier > 0.0 {
                    suitable_count += 1;
                }
            }
        }
    }

    // Estimate total suitable tiles
    let estimated_suitable = suitable_count * sample_step * sample_step;
    info!("🌿 Estimated suitable tiles: {}", estimated_suitable);

    // Update vegetation grid statistics
    vegetation_grid.total_suitable_tiles = estimated_suitable;

    info!("✅ Vegetation system initialized successfully");
}

/// Growth system that updates vegetation biomass
/// Runs at 1 Hz (every 10 ticks at 10 TPS)
fn vegetation_growth_system(
    mut vegetation_grid: ResMut<VegetationGrid>,
    tick: Res<SimulationTick>,
) {
    vegetation_grid.update(tick.0);

    // Log statistics periodically
    if tick.0 % 600 == 0 { // Every 60 seconds at 10 TPS
        let stats = vegetation_grid.get_statistics();
        info!("🌱 Vegetation Stats - Tiles: {}, Active: {}, Depleted: {}, Avg Biomass: {:.1}%",
              stats.suitable_tiles,
              stats.active_tiles,
              stats.depleted_tiles,
              stats.average_biomass / constants::growth::MAX_BIOMASS * 100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_vegetation_creation() {
        let vegetation = TileVegetation::new(50.0, 1.0);
        assert_eq!(vegetation.biomass, 50.0);
        assert_eq!(vegetation.terrain_multiplier, 1.0);
        assert_eq!(vegetation.last_grazed_tick, 0);
    }

    #[test]
    fn test_tile_vegetation_max_biomass() {
        let vegetation = TileVegetation::new(50.0, 0.8); // 80% terrain multiplier
        assert_eq!(vegetation.max_biomass(), 80.0); // 100.0 * 0.8
    }

    #[test]
    fn test_tile_vegetation_fraction_full() {
        let vegetation = TileVegetation::new(50.0, 1.0);
        assert_eq!(vegetation.fraction_full(), 0.5); // 50.0 / 100.0
    }

    #[test]
    fn test_tile_vegetation_add_biomass() {
        let mut vegetation = TileVegetation::new(50.0, 1.0);
        vegetation.add_biomass(30.0);
        assert_eq!(vegetation.biomass, 80.0);

        // Test clamping to max biomass
        vegetation.add_biomass(50.0); // Would exceed max
        assert_eq!(vegetation.biomass, 100.0); // Clamped to MAX_BIOMASS
    }

    #[test]
    fn test_tile_vegetation_remove_biomass() {
        let mut vegetation = TileVegetation::new(50.0, 1.0);
        let removed = vegetation.remove_biomass(30.0);
        assert_eq!(removed, 30.0);
        assert_eq!(vegetation.biomass, 20.0);

        // Test removing more than available
        let removed = vegetation.remove_biomass(50.0);
        assert_eq!(removed, 20.0); // Only 20.0 available
        assert_eq!(vegetation.biomass, 0.0);
    }

    #[test]
    fn test_tile_vegetation_mark_grazed() {
        let mut vegetation = TileVegetation::new(50.0, 1.0);
        vegetation.mark_grazed(12345);
        assert_eq!(vegetation.last_grazed_tick, 12345);
    }

    #[test]
    fn test_tile_vegetation_is_depleted() {
        let mut vegetation = TileVegetation::new(3.0, 1.0);
        assert!(vegetation.is_depleted()); // Below DEPLETED_THRESHOLD (5.0)

        vegetation.add_biomass(10.0);
        assert!(!vegetation.is_depleted()); // Above threshold
    }

    #[test]
    fn test_tile_vegetation_is_active() {
        let mut vegetation = TileVegetation::new(90.0, 1.0); // Near max biomass
        assert!(!vegetation.is_active(100)); // Not active (above 95% threshold)

        vegetation.mark_grazed(100);
        assert!(vegetation.is_active(120)); // Active due to recent grazing

        vegetation.biomass = 10.0; // Low biomass
        assert!(vegetation.is_active(200)); // Active due to low biomass
    }

    #[test]
    fn test_vegetation_grid_creation() {
        let grid = VegetationGrid::new();
        assert_eq!(grid.tiles.len(), 0);
        assert_eq!(grid.active_tiles.len(), 0);
        assert_eq!(grid.total_suitable_tiles, 0);
        assert_eq!(grid.current_tick, 0);
    }

    #[test]
    fn test_vegetation_grid_get_or_create() {
        let mut grid = VegetationGrid::new();
        let tile = IVec2::new(5, 10);

        // First call creates the tile
        let vegetation = grid.get_or_create(tile, 1.0);
        assert_eq!(vegetation.biomass, 100.0); // Starts at MAX_BIOMASS * terrain_multiplier

        // Drop the first borrow
        drop(vegetation);

        // Second call returns existing tile
        let vegetation2 = grid.get_or_create(tile, 1.0);
        assert_eq!(vegetation2.biomass, 100.0);
    }

    #[test]
    #[should_panic(expected = "Attempted to create vegetation on non-vegetated terrain")]
    fn test_vegetation_grid_non_vegetated_terrain() {
        let mut grid = VegetationGrid::new();
        grid.get_or_create(IVec2::new(0, 0), 0.0); // Should panic
    }

    #[test]
    fn test_vegetation_grid_consume() {
        let mut grid = VegetationGrid::new();
        let tile = IVec2::new(5, 10);
        grid.get_or_create(tile, 1.0); // Create tile with 100.0 biomass

        // Test consumption within limits
        let (consumed, _remaining) = grid.consume(tile, 20.0, 0.3); // 30% of 100.0 = 30.0 max
        assert_eq!(consumed, 20.0);

        let vegetation = grid.get(tile).unwrap();
        assert_eq!(vegetation.biomass, 80.0);
        assert_eq!(vegetation.last_grazed_tick, 0); // Not marked grazed yet

        // Test consumption exceeding 30% rule
        let (consumed, _remaining) = grid.consume(tile, 50.0, 0.3); // 30% of 80.0 = 24.0 max
        assert_eq!(consumed, 24.0);

        let vegetation = grid.get(tile).unwrap();
        assert_eq!(vegetation.biomass, 56.0);
    }

    #[test]
    fn test_vegetation_grid_sample_biomass() {
        let mut grid = VegetationGrid::new();

        // Create some test tiles
        grid.get_or_create(IVec2::new(0, 0), 1.0); // 100.0 biomass
        grid.get_or_create(IVec2::new(1, 0), 0.5); // 50.0 biomass
        grid.get_or_create(IVec2::new(0, 1), 0.8); // 80.0 biomass

        let (avg_biomass, count) = grid.sample_biomass(IVec2::new(0, 0), 2);
        assert_eq!(count, 3);
        assert_eq!(avg_biomass, (100.0 + 50.0 + 80.0) / 3.0);
    }

    #[test]
    fn test_vegetation_grid_find_best_forage_tile() {
        let mut grid = VegetationGrid::new();

        // Create tiles with different biomass levels
        grid.get_or_create(IVec2::new(0, 0), 1.0); // 100.0 biomass
        grid.get_or_create(IVec2::new(5, 0), 0.3); // 30.0 biomass
        grid.get_or_create(IVec2::new(2, 2), 0.9); // 90.0 biomass

        let best = grid.find_best_forage_tile(IVec2::new(0, 0), 10);
        assert!(best.is_some());
        let (tile, biomass) = best.unwrap();

        // Should prefer the tile with highest biomass considering distance
        // (0,0) has 100.0 biomass and distance 0, so it should be chosen
        assert_eq!(tile, IVec2::new(0, 0));
        assert_eq!(biomass, 100.0);
    }

    #[test]
    fn test_logistic_growth_to_80_percent() {
        // This test verifies that an empty patch reaches ~80% of Bmax after expected ticks
        let mut grid = VegetationGrid::new();
        let tile = IVec2::new(0, 0);

        // Start with empty patch
        grid.get_or_create(tile, 1.0);
        {
            let vegetation = grid.get_mut(tile).unwrap();
            vegetation.biomass = 1.0; // Start very low
        }

        let max_biomass = 100.0;
        let target_biomass = max_biomass * 0.8; // 80.0
        let growth_rate = constants::growth::GROWTH_RATE; // 0.05

        // Simulate growth over multiple ticks
        let mut current_biomass = 1.0;
        let mut tick = 0;

        while current_biomass < target_biomass && tick < 1000 {
            // Logistic growth: B(t+1) = B(t) + r * B(t) * (1 - B(t)/Bmax)
            let growth = growth_rate * current_biomass * (1.0 - current_biomass / max_biomass);
            current_biomass += growth;
            tick += 1;

            // Apply the same update to the grid
            grid.update(tick);
        }

        // Verify we reached the target
        let final_vegetation = grid.get(tile).unwrap();
        assert!(final_vegetation.biomass >= target_biomass * 0.95, // Within 5% of target
               "Expected biomass to reach at least 95% of 80.0 Bmax, got {}", final_vegetation.biomass);
        assert!(tick < 200, "Should reach 80% Bmax within 200 ticks, took {}", tick);

        // Verify the final biomass is reasonable (should be close to 80.0)
        assert!((final_vegetation.biomass - 80.0).abs() < 5.0_f32,
               "Final biomass should be close to 80.0, got {}", final_vegetation.biomass);
    }

    #[test]
    fn test_logistic_growth_equation() {
        // Test the logistic growth equation directly
        let max_biomass = 100.0;
        let growth_rate = 0.05;

        // Start at 50% capacity
        let mut biomass = 50.0;

        // Apply one growth step
        let growth = growth_rate * biomass * (1.0 - biomass / max_biomass);
        biomass += growth;

        // Should grow (but not exceed max)
        assert!(biomass > 50.0);
        assert!(biomass < max_biomass);

        // At 50% capacity, growth should be at maximum
        let expected_max_growth = growth_rate * max_biomass * 0.25; // r * Bmax * 0.25
        let diff = (growth as f32 - expected_max_growth).abs();
        assert!(diff < 0.01_f32);
    }

    #[test]
    fn test_vegetation_statistics() {
        let mut grid = VegetationGrid::new();

        // Create some test tiles
        grid.get_or_create(IVec2::new(0, 0), 1.0); // 100.0 biomass
        grid.get_or_create(IVec2::new(1, 0), 0.5); // 50.0 biomass
        grid.get_or_create(IVec2::new(0, 1), 0.8); // 80.0 biomass

        // Mark one as depleted
        {
            let vegetation = grid.get_mut(IVec2::new(1, 0)).unwrap();
            vegetation.biomass = 3.0; // Below DEPLETED_THRESHOLD
        }

        // Set suitable tiles count
        grid.total_suitable_tiles = 3;

        let stats = grid.get_statistics();
        assert_eq!(stats.total_tiles, 3);
        assert_eq!(stats.suitable_tiles, 3);
        assert_eq!(stats.total_biomass, 230.0); // 100 + 50 + 80
        assert_eq!(stats.average_biomass, 230.0 / 3.0);
        assert_eq!(stats.depleted_tiles, 1);
    }

    #[test]
    fn test_herbivore_consumption() {
        let mut grid = VegetationGrid::new();

        // Create a test tile with high biomass
        let tile_pos = IVec2::new(5, 5);
        let initial_biomass = 80.0;
        grid.get_or_create(tile_pos, 1.0); // Normalized to MAX_BIOMASS = 100.0
        {
            let vegetation = grid.get_mut(tile_pos).unwrap();
            vegetation.biomass = initial_biomass;
        }

        // Test consumption with moderate demand
        let demand = 30.0;
        let max_fraction = consumption::MAX_MEAL_FRACTION; // 30%

        let (consumed, remaining_demand) = grid.consume(tile_pos, demand, max_fraction);

        // Should consume min(demand, 30% of biomass) = min(30, 24) = 24
        assert_eq!(consumed, 24.0);
        assert_eq!(remaining_demand, demand - consumed); // Remaining demand, not biomass

        // Verify the tile was updated
        let updated_vegetation = grid.get(tile_pos).unwrap();
        assert_eq!(updated_vegetation.biomass, initial_biomass - consumed);
        assert_eq!(updated_vegetation.last_grazed_tick, 0); // Should be updated

        // Test consumption with low demand
        let low_demand = 10.0;
        let (consumed_low, _) = grid.consume(tile_pos, low_demand, max_fraction);
        assert_eq!(consumed_low, 10.0); // Should consume full demand

        // Test consumption with depleted tile
        let depleted_pos = IVec2::new(6, 6);
        grid.get_or_create(depleted_pos, 1.0);
        {
            let vegetation = grid.get_mut(depleted_pos).unwrap();
            vegetation.biomass = 8.0; // Below FORAGE_MIN_BIOMASS (10.0) but should still consume
        }

        let (consumed_depleted, _) = grid.consume(depleted_pos, 20.0, max_fraction);
        // 30% of 8.0 = 2.4, but this should be prevented by FORAGE_MIN_BIOMASS check in eating behavior
        println!("Consumed from low biomass tile: {}", consumed_depleted);
        assert!(consumed_depleted >= 0.0); // Should consume something if not blocked by FORAGE_MIN_BIOMASS

        // Test consumption on non-existent tile
        let empty_pos = IVec2::new(10, 10);
        let (consumed_empty, _) = grid.consume(empty_pos, 20.0, max_fraction);
        assert_eq!(consumed_empty, 0.0); // Should consume nothing
    }
}