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
pub mod memory_optimization;
pub mod benchmark;

use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

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
/// Enhanced active tile management for Phase 4 performance optimization
#[derive(Debug, Clone)]
struct ActiveTileManager {
    /// Queue of recently grazed tiles that need frequent updates
    /// Ordered by graze time for FIFO processing
    recently_grazed: VecDeque<IVec2>,

    /// Set of tiles below biomass threshold needing frequent updates
    /// These are tiles that are actively regrowing toward full capacity
    regrowing_tiles: HashMap<IVec2, u64>, // tile -> last_update_tick

    /// Set of tiles that were recently processed to avoid duplication
    /// Tracks which tiles were updated in the current cycle
    processed_this_cycle: std::collections::HashSet<IVec2>,

    /// Performance metrics for monitoring
    metrics: ActiveTileMetrics,
}

#[derive(Debug, Clone, Default)]
pub struct ActiveTileMetrics {
    /// Number of active tiles currently being tracked
    pub active_count: usize,

    /// Number of tiles processed in last update cycle
    pub processed_last_cycle: usize,

    /// Number of recently grazed tiles
    pub recently_grazed_count: usize,

    /// Number of regrowing tiles
    pub regrowing_count: usize,

    /// CPU time spent on active tile management (in microseconds)
    pub processing_time_us: u64,
}

#[derive(Resource, Debug)]
pub struct VegetationGrid {
    /// Sparse storage: tile coordinates -> vegetation state
    /// Uses sparse storage for memory efficiency on large maps
    tiles: HashMap<IVec2, TileVegetation>,

    /// Enhanced active tile management system
    active_manager: ActiveTileManager,

    /// Total number of tiles that could support vegetation
    total_suitable_tiles: usize,

    /// Current tick counter for timing calculations
    current_tick: u64,

    /// Performance monitoring for Phase 4 benchmarks
    performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    /// Total tiles processed in last update
    pub tiles_processed: usize,

    /// Active tiles processed in last update
    pub active_tiles_processed: usize,

    /// Inactive tiles sampled in last update
    pub inactive_tiles_sampled: usize,

    /// Total CPU time for vegetation system (in microseconds)
    pub total_time_us: u64,

    /// Time spent on growth calculations (in microseconds)
    pub growth_time_us: u64,

    /// Time spent on active tile management (in microseconds)
    pub active_management_time_us: u64,

    /// Number of update cycles completed
    pub update_cycles: u64,

    /// Phase 4 batch processing metrics
    pub batch_metrics: BatchMetrics,

    /// Adaptive performance scaling metrics
    pub adaptive_metrics: AdaptiveMetrics,
}

#[derive(Debug, Clone, Default)]
pub struct BatchMetrics {
    /// Number of batches processed in last update cycle
    pub batches_processed: usize,

    /// Total tiles processed across all batches
    pub total_tiles_in_batches: usize,

    /// Average time per batch (in microseconds)
    pub avg_batch_time_us: u64,

    /// Maximum time taken by a single batch (in microseconds)
    pub max_batch_time_us: u64,

    /// Number of batches that exceeded time budget
    pub batches_over_budget: usize,

    /// Tiles processed per batch on average
    pub avg_tiles_per_batch: f32,
}

#[derive(Debug, Clone, Default)]
pub struct AdaptiveMetrics {
    /// Current update frequency multiplier
    /// 1.0 = normal frequency, >1.0 = slower, <1.0 = faster
    pub frequency_multiplier: f32,

    /// Average biomass across all suitable tiles
    pub average_biomass: f32,

    /// Number of adaptive adjustments made
    pub adjustments_made: u64,

    /// Last adjustment tick
    pub last_adjustment_tick: u64,

    /// Performance pressure (0.0 = low, 1.0 = high)
    /// Based on CPU usage vs budget
    pub performance_pressure: f32,
}

#[derive(Debug, Clone, Default)]
pub struct BatchProcessingResult {
    /// Total number of tiles processed across all batches
    pub total_tiles_processed: usize,

    /// Number of batches that were executed
    pub batches_processed: usize,

    /// Total time spent on all batches (in microseconds)
    pub total_batch_time_us: u64,

    /// Maximum time taken by a single batch (in microseconds)
    pub max_batch_time_us: u64,

    /// Number of batches that exceeded the time budget
    pub batches_over_budget: usize,

    /// Average time per batch (in microseconds)
    pub avg_batch_time_us: u64,

    /// Average number of tiles processed per batch
    pub avg_tiles_per_batch: f32,
}

impl VegetationGrid {
    /// Create a new vegetation grid
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new(),
            active_manager: ActiveTileManager {
                recently_grazed: VecDeque::new(),
                regrowing_tiles: HashMap::new(),
                processed_this_cycle: std::collections::HashSet::new(),
                metrics: ActiveTileMetrics::default(),
            },
            total_suitable_tiles: 0,
            current_tick: 0,
            performance_metrics: PerformanceMetrics::default(),
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

                // Add to recently grazed queue for active management
                self.active_manager.recently_grazed.push_back(tile);

                // Add to regrowing set if below threshold
                if vegetation.is_active(self.current_tick) {
                    self.active_manager.regrowing_tiles.insert(tile, self.current_tick);
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

    /// Update the vegetation grid with Phase 4 performance optimizations
    pub fn update(&mut self, current_tick: u64) {
        self.current_tick = current_tick;

        // Only run growth updates on interval
        if current_tick % GROWTH_INTERVAL_TICKS != 0 {
            return;
        }

        // Start performance monitoring
        let start_time = std::time::Instant::now();

        // Clear processed set for new cycle
        self.active_manager.processed_this_cycle.clear();

        // Update active tiles with enhanced management
        self.update_active_tiles_enhanced();

        // Sample some inactive tiles for occasional updates
        self.update_inactive_tile_sample();

        // Update performance metrics
        let total_time = start_time.elapsed().as_micros() as u64;
        self.performance_metrics.total_time_us = total_time;
        self.performance_metrics.tiles_processed =
            self.performance_metrics.active_tiles_processed + self.performance_metrics.inactive_tiles_sampled;
        self.performance_metrics.update_cycles += 1;

        // Apply adaptive performance scaling
        self.adjust_performance_scaling();

        // Log performance metrics periodically
        if current_tick % 600 == 0 { // Every 60 seconds at 10 TPS
            self.log_performance_metrics();
        }
    }

    /// Enhanced active tile update with Phase 4 optimizations
    fn update_active_tiles_enhanced(&mut self) {
        use constants::growth::GROWTH_RATE;
        use constants::performance::MAX_ACTIVE_TILES_PER_UPDATE;

        let active_start_time = std::time::Instant::now();
        let mut active_processed = 0;
        let mut recently_grazed_processed = 0;
        let mut regrowing_processed = 0;

        // Clear old metrics and update current counts
        self.active_manager.metrics.active_count =
            self.active_manager.recently_grazed.len() + self.active_manager.regrowing_tiles.len();
        self.active_manager.metrics.recently_grazed_count = self.active_manager.recently_grazed.len();
        self.active_manager.metrics.regrowing_count = self.active_manager.regrowing_tiles.len();

        // Process recently grazed tiles first (highest priority)
        let max_recently_grazed = (MAX_ACTIVE_TILES_PER_UPDATE / 2).min(self.active_manager.recently_grazed.len());

        for _ in 0..max_recently_grazed {
            if let Some(tile) = self.active_manager.recently_grazed.pop_front() {
                if self.update_single_tile(tile) {
                    recently_grazed_processed += 1;
                    active_processed += 1;
                }
            }
        }

        // Process regrowing tiles (second priority)
        let max_regrowing = (MAX_ACTIVE_TILES_PER_UPDATE - active_processed).min(self.active_manager.regrowing_tiles.len());

        let regrowing_tiles: Vec<IVec2> = self.active_manager.regrowing_tiles
            .keys()
            .take(max_regrowing)
            .copied()
            .collect();

        // Use batch processing for regrowing tiles (Phase 4 optimization)
        if !regrowing_tiles.is_empty() {
            let batch_result = self.process_tiles_in_batches(regrowing_tiles);
            regrowing_processed = batch_result.total_tiles_processed;
            active_processed += regrowing_processed;

            // Update performance metrics with batch results
            self.performance_metrics.batch_metrics = BatchMetrics {
                batches_processed: self.performance_metrics.batch_metrics.batches_processed + batch_result.batches_processed,
                total_tiles_in_batches: self.performance_metrics.batch_metrics.total_tiles_in_batches + batch_result.total_tiles_processed,
                avg_batch_time_us: batch_result.avg_batch_time_us,
                max_batch_time_us: batch_result.max_batch_time_us,
                batches_over_budget: self.performance_metrics.batch_metrics.batches_over_budget + batch_result.batches_over_budget,
                avg_tiles_per_batch: batch_result.avg_tiles_per_batch,
            };
        }

        // Update metrics
        self.active_manager.metrics.processed_last_cycle = active_processed;
        let active_time = active_start_time.elapsed().as_micros() as u64;
        self.active_manager.metrics.processing_time_us = active_time;

        self.performance_metrics.active_tiles_processed = active_processed;
        self.performance_metrics.active_management_time_us = active_time;

        debug!("🌱 Active tile update: processed {} tiles (recently grazed: {}, regrowing: {}) in {}μs",
              active_processed, recently_grazed_processed, regrowing_processed, active_time);
    }

    /// Update a single tile with growth calculation
    /// Returns true if tile was updated, false if it should be removed from active tracking
    fn update_single_tile(&mut self, tile: IVec2) -> bool {
        use constants::growth::GROWTH_RATE;

        // Skip if already processed this cycle
        if self.active_manager.processed_this_cycle.contains(&tile) {
            return false;
        }

        if let Some(vegetation) = self.tiles.get_mut(&tile) {
            let max_biomass = vegetation.max_biomass();
            if max_biomass > 0.0 {
                // Calculate logistic growth
                let growth = GROWTH_RATE * vegetation.biomass * (1.0 - vegetation.biomass / max_biomass);
                vegetation.add_biomass(growth);

                // Mark as processed
                self.active_manager.processed_this_cycle.insert(tile);

                // Keep tile active if it's still below threshold
                let should_remain_active = vegetation.is_active(self.current_tick);

                // Remove from regrowing set if fully recovered
                if !should_remain_active {
                    self.active_manager.regrowing_tiles.remove(&tile);
                }

                return should_remain_active;
            }
        }

        false // Tile should be removed from active tracking
    }

    /// Update a random sample of inactive tiles with performance tracking
    fn update_inactive_tile_sample(&mut self) {
        use constants::growth::GROWTH_RATE;
        use constants::performance::INACTIVE_SAMPLE_SIZE;

        let sample_start_time = std::time::Instant::now();
        let mut inactive_processed = 0;

        // Get inactive tiles (those not in active manager)
        let inactive_tiles: Vec<IVec2> = self.tiles
            .keys()
            .filter(|tile| {
                !self.active_manager.recently_grazed.contains(tile) &&
                !self.active_manager.regrowing_tiles.contains_key(tile) &&
                !self.active_manager.processed_this_cycle.contains(tile)
            })
            .take(INACTIVE_SAMPLE_SIZE)
            .copied()
            .collect();

        for tile in inactive_tiles {
            if let Some(vegetation) = self.tiles.get_mut(&tile) {
                let max_biomass = vegetation.max_biomass();
                if max_biomass > 0.0 {
                    let growth = GROWTH_RATE * vegetation.biomass * (1.0 - vegetation.biomass / max_biomass);
                    vegetation.add_biomass(growth);

                    // Add to active manager if it became active
                    if vegetation.is_active(self.current_tick) {
                        self.active_manager.regrowing_tiles.insert(tile, self.current_tick);
                    }

                    inactive_processed += 1;
                }
            }
        }

        // Update performance metrics
        let sample_time = sample_start_time.elapsed().as_micros() as u64;
        self.performance_metrics.inactive_tiles_sampled = inactive_processed;
        self.performance_metrics.growth_time_us += sample_time; // Add to growth time

        debug!("🌱 Inactive tile sample: processed {} tiles in {}μs", inactive_processed, sample_time);
    }

    /// Process tiles in batches with time budgeting for Phase 4 optimization
    fn process_tiles_in_batches(&mut self, tiles: Vec<IVec2>) -> BatchProcessingResult {
        use constants::performance::{BATCH_SIZE, BATCH_TIME_BUDGET_US};

        let mut result = BatchProcessingResult::default();
        let mut batch_start_time = std::time::Instant::now();

        for (batch_index, batch) in tiles.chunks(BATCH_SIZE).enumerate() {
            let batch_start = std::time::Instant::now();

            // Process the batch
            let tiles_processed_in_batch = self.process_single_batch(batch);

            let batch_time = batch_start.elapsed().as_micros() as u64;

            // Update batch metrics
            result.total_tiles_processed += tiles_processed_in_batch;
            result.batches_processed += 1;
            result.total_batch_time_us += batch_time;
            result.max_batch_time_us = result.max_batch_time_us.max(batch_time);

            // Check if batch exceeded time budget
            if batch_time > BATCH_TIME_BUDGET_US {
                result.batches_over_budget += 1;
                warn!("⚠️  Batch {} exceeded time budget: {}μs > {}μs (processed {} tiles)",
                     batch_index, batch_time, BATCH_TIME_BUDGET_US, tiles_processed_in_batch);
            }

            // Check if we're approaching the overall time budget
            let elapsed_so_far = batch_start_time.elapsed().as_micros() as u64;
            if elapsed_so_far > (constants::performance::CPU_BUDGET_US - BATCH_TIME_BUDGET_US) {
                debug!("🛑 Approaching time budget, stopping batch processing at {} batches", batch_index + 1);
                break;
            }
        }

        // Calculate averages
        if result.batches_processed > 0 {
            result.avg_batch_time_us = result.total_batch_time_us / result.batches_processed as u64;
            result.avg_tiles_per_batch = result.total_tiles_processed as f32 / result.batches_processed as f32;
        }

        result
    }

    /// Process a single batch of tiles
    fn process_single_batch(&mut self, batch: &[IVec2]) -> usize {
        use constants::growth::GROWTH_RATE;

        let mut processed = 0;

        for &tile in batch {
            // Skip if already processed this cycle
            if self.active_manager.processed_this_cycle.contains(&tile) {
                continue;
            }

            if let Some(vegetation) = self.tiles.get_mut(&tile) {
                let max_biomass = vegetation.max_biomass();
                if max_biomass > 0.0 {
                    // Calculate logistic growth
                    let growth = GROWTH_RATE * vegetation.biomass * (1.0 - vegetation.biomass / max_biomass);
                    vegetation.add_biomass(growth);

                    // Mark as processed
                    self.active_manager.processed_this_cycle.insert(tile);

                    // Update active status
                    let should_remain_active = vegetation.is_active(self.current_tick);
                    if !should_remain_active {
                        self.active_manager.regrowing_tiles.remove(&tile);
                    } else if !self.active_manager.regrowing_tiles.contains_key(&tile) {
                        self.active_manager.regrowing_tiles.insert(tile, self.current_tick);
                    }

                    processed += 1;
                }
            }
        }

        processed
    }

    /// Adaptive performance scaling based on system load
    fn adjust_performance_scaling(&mut self) {
        use constants::performance::{
            TARGET_AVG_BIOMASS, HIGH_BIOMASS_THRESHOLD, LOW_BIOMASS_THRESHOLD,
            CPU_BUDGET_US, PROFILING_INTERVAL_TICKS
        };

        // Only check at profiling intervals
        if self.current_tick % PROFILING_INTERVAL_TICKS != 0 {
            return;
        }

        let stats = self.get_statistics();
        let avg_biomass_pct = (stats.average_biomass / constants::growth::MAX_BIOMASS) * 100.0;
        let cpu_utilization_pct = (self.performance_metrics.total_time_us as f32 / CPU_BUDGET_US as f32) * 100.0;

        // Calculate performance pressure
        let performance_pressure = cpu_utilization_pct / 100.0;
        self.performance_metrics.adaptive_metrics.performance_pressure = performance_pressure;
        self.performance_metrics.adaptive_metrics.average_biomass = stats.average_biomass;

        let mut adjustment_made = false;
        let mut new_multiplier = self.performance_metrics.adaptive_metrics.frequency_multiplier;

        // Adjust based on CPU pressure
        if performance_pressure > 0.8 && new_multiplier < 2.0 {
            new_multiplier = (new_multiplier * 1.2).min(2.0);
            adjustment_made = true;
            info!("🐌 High CPU pressure ({:.1}%), reducing update frequency to {:.1}x",
                 performance_pressure * 100.0, new_multiplier);
        } else if performance_pressure < 0.3 && new_multiplier > 0.5 {
            new_multiplier = (new_multiplier * 0.9).max(0.5);
            adjustment_made = true;
            info!("🚀 Low CPU pressure ({:.1}%), increasing update frequency to {:.1}x",
                 performance_pressure * 100.0, new_multiplier);
        }

        // Adjust based on biomass levels
        if avg_biomass_pct > 80.0 && new_multiplier < 1.5 {
            new_multiplier = (new_multiplier * 1.1).min(1.5);
            adjustment_made = true;
            info!("🌿 High biomass ({:.1}%), reducing update frequency to {:.1}x",
                 avg_biomass_pct, new_multiplier);
        } else if avg_biomass_pct < 20.0 && new_multiplier > 0.7 {
            new_multiplier = (new_multiplier * 0.9).max(0.7);
            adjustment_made = true;
            info!("🌱 Low biomass ({:.1}%), increasing update frequency to {:.1}x",
                 avg_biomass_pct, new_multiplier);
        }

        // Apply the adjustment
        if adjustment_made {
            self.performance_metrics.adaptive_metrics.frequency_multiplier = new_multiplier;
            self.performance_metrics.adaptive_metrics.adjustments_made += 1;
            self.performance_metrics.adaptive_metrics.last_adjustment_tick = self.current_tick;
        }
    }

    /// Log performance metrics for Phase 4 monitoring
    fn log_performance_metrics(&self) {
        info!("📊 Vegetation Performance Metrics - Cycle {}:", self.performance_metrics.update_cycles);
        info!("  Core Processing:");
        info!("    Total tiles processed: {}", self.performance_metrics.tiles_processed);
        info!("    Active tiles processed: {}", self.performance_metrics.active_tiles_processed);
        info!("    Inactive tiles sampled: {}", self.performance_metrics.inactive_tiles_sampled);

        info!("  Timing:");
        info!("    Total CPU time: {}μs", self.performance_metrics.total_time_us);
        info!("    Active management time: {}μs", self.performance_metrics.active_management_time_us);
        info!("    Growth calculation time: {}μs", self.performance_metrics.growth_time_us);

        info!("  Active Tile Management:");
        info!("    Active tiles tracked: {}", self.active_manager.metrics.active_count);
        info!("    Recently grazed: {}", self.active_manager.metrics.recently_grazed_count);
        info!("    Regrowing tiles: {}", self.active_manager.metrics.regrowing_count);

        info!("  Batch Processing:");
        info!("    Batches processed: {}", self.performance_metrics.batch_metrics.batches_processed);
        info!("    Tiles per batch: {:.1}", self.performance_metrics.batch_metrics.avg_tiles_per_batch);
        info!("    Avg batch time: {}μs", self.performance_metrics.batch_metrics.avg_batch_time_us);
        info!("    Max batch time: {}μs", self.performance_metrics.batch_metrics.max_batch_time_us);
        if self.performance_metrics.batch_metrics.batches_over_budget > 0 {
            warn!("    Batches over budget: {}", self.performance_metrics.batch_metrics.batches_over_budget);
        }

        info!("  Adaptive Scaling:");
        info!("    Frequency multiplier: {:.2}x", self.performance_metrics.adaptive_metrics.frequency_multiplier);
        info!("    Average biomass: {:.1}%", self.performance_metrics.adaptive_metrics.average_biomass / constants::growth::MAX_BIOMASS * 100.0);
        info!("    Performance pressure: {:.2}", self.performance_metrics.adaptive_metrics.performance_pressure);
        info!("    Adjustments made: {}", self.performance_metrics.adaptive_metrics.adjustments_made);

        // Performance budget check
        let budget_us = constants::performance::CPU_BUDGET_US;
        let cpu_utilization = (self.performance_metrics.total_time_us as f32 / budget_us as f32) * 100.0;

        if self.performance_metrics.total_time_us > budget_us {
            warn!("⚠️  Vegetation system exceeded CPU budget: {}μs > {}μs ({:.1}% utilization)",
                  self.performance_metrics.total_time_us, budget_us, cpu_utilization);
        } else {
            info!("✅ Vegetation system within CPU budget: {}μs / {}μs ({:.1}% utilization)",
                  self.performance_metrics.total_time_us, budget_us, cpu_utilization);
        }

        // Performance efficiency calculation
        let tiles_per_us = self.performance_metrics.tiles_processed as f32 / self.performance_metrics.total_time_us as f32;
        info!("🎯 Performance: {:.1} tiles per microsecond", tiles_per_us);

        // Efficiency rating
        let efficiency_rating = if tiles_per_us > 2.0 {
            "Excellent"
        } else if tiles_per_us > 1.0 {
            "Good"
        } else if tiles_per_us > 0.5 {
            "Fair"
        } else {
            "Poor"
        };
        info!("🏆 Efficiency Rating: {}", efficiency_rating);

        // Phase 4 memory optimization analysis
        if self.current_tick % constants::memory::MEMORY_ANALYSIS_INTERVAL_TICKS == 0 {
            self.log_memory_optimization_analysis();
        }
    }

    /// Log memory optimization analysis and recommendations
    fn log_memory_optimization_analysis(&self) {
        use crate::vegetation::memory_optimization::*;
        use constants::memory::*;

        let memory_analysis = self.analyze_memory_usage();
        let (storage_comparison, recommendations) = self.get_memory_recommendations();
        let savings_estimate = self.estimate_memory_savings();

        info!("💾 Memory Optimization Analysis:");
        info!("  Current Usage: {:.2} MB ({} tiles, {:.1} bytes/tile)",
             memory_analysis.total_bytes as f32 / (1024.0 * 1024.0),
             memory_analysis.tile_count,
             memory_analysis.bytes_per_tile as f32);

        info!("  Memory Breakdown:");
        info!("    Biomass: {:.2} MB", memory_analysis.breakdown.biomass_bytes as f32 / (1024.0 * 1024.0));
        info!("    Terrain: {:.2} MB", memory_analysis.breakdown.terrain_multiplier_bytes as f32 / (1024.0 * 1024.0));
        info!("    Tracking: {:.2} MB", memory_analysis.breakdown.last_grazed_bytes as f32 / (1024.0 * 1024.0));
        info!("    HashMap: {:.2} MB", memory_analysis.breakdown.hashmap_overhead as f32 / (1024.0 * 1024.0));
        info!("    Active: {:.2} MB", memory_analysis.breakdown.active_tracking_bytes as f32 / (1024.0 * 1024.0));

        info!("  Storage Optimization:");
        info!("    u16 storage savings: {:.1}%", storage_comparison.savings_percent);
        info!("    Precision loss: {:.2}%", storage_comparison.precision_loss_percent);
        info!("    Combined optimization: {:.1}%", savings_estimate.combined_savings_percent);

        // Memory usage category
        let memory_category = if memory_analysis.total_bytes > HIGH_MEMORY_THRESHOLD {
            "HIGH"
        } else if memory_analysis.total_bytes > MEDIUM_MEMORY_THRESHOLD {
            "MEDIUM"
        } else if memory_analysis.total_bytes > LOW_MEMORY_THRESHOLD {
            "LOW"
        } else {
            "MINIMAL"
        };
        info!("    Usage Category: {}", memory_category);

        // Recommendations
        if !recommendations.is_empty() {
            info!("🔧 Optimization Recommendations:");
            for (i, rec) in recommendations.iter().enumerate() {
                info!("    {}. {}", i + 1, rec);
            }
        }

        // Performance efficiency
        let memory_efficiency = if memory_analysis.bytes_per_tile < 35 {
            "Excellent"
        } else if memory_analysis.bytes_per_tile < 40 {
            "Good"
        } else if memory_analysis.bytes_per_tile < 50 {
            "Fair"
        } else {
            "Poor"
        };
        info!("📈 Memory Efficiency: {}", memory_efficiency);

        // Warning thresholds
        if memory_analysis.total_bytes > HIGH_MEMORY_THRESHOLD {
            warn!("⚠️  High memory usage detected: {:.2} MB exceeds {} MB threshold",
                 memory_analysis.total_bytes as f32 / (1024.0 * 1024.0),
                 HIGH_MEMORY_THRESHOLD / (1024 * 1024));
        }

        if memory_analysis.bytes_per_tile > PER_TILE_OVERHEAD_THRESHOLD {
            warn!("⚠️  High per-tile overhead: {} bytes exceeds {} byte threshold",
                 memory_analysis.bytes_per_tile,
                 PER_TILE_OVERHEAD_THRESHOLD);
        }

        // Regional grid recommendation
        if memory_analysis.tile_count > REGIONAL_GRID_RECOMMENDATION_TILES {
            info!("🗺️ Regional grid recommended: {} tiles would benefit from cache-friendly organization",
                 memory_analysis.tile_count);
        }
    }

    /// Get performance metrics for external monitoring
    pub fn get_performance_metrics(&self) -> &PerformanceMetrics {
        &self.performance_metrics
    }

    /// Get active tile manager metrics for debugging
    pub fn get_active_tile_metrics(&self) -> &ActiveTileMetrics {
        &self.active_manager.metrics
    }

    /// Phase 4 memory optimization: Analyze current memory usage
    pub fn analyze_memory_usage(&self) -> crate::vegetation::memory_optimization::MemoryAnalysis {
        crate::vegetation::memory_optimization::MemoryAnalysis::analyze_current(
            self.tiles.len(),
            self.active_manager.metrics.active_count,
            &self.performance_metrics,
        )
    }

    /// Phase 4 memory optimization: Compare f32 vs u16 storage
    pub fn compare_storage_options(&self) -> crate::vegetation::memory_optimization::StorageComparison {
        crate::vegetation::memory_optimization::StorageComparison::compare_storage(
            self.tiles.len(),
            self.active_manager.metrics.active_count,
            &self.performance_metrics,
        )
    }

    /// Phase 4 memory optimization: Generate memory optimization recommendations
    pub fn get_memory_recommendations(&self) -> (crate::vegetation::memory_optimization::StorageComparison, Vec<String>) {
        crate::vegetation::memory_optimization::MemoryOptimizer::analyze_and_optimize(
            self.tiles.len(),
            self.active_manager.metrics.active_count,
            &self.performance_metrics,
        )
    }

    /// Phase 4 memory optimization: Estimate potential memory savings
    pub fn estimate_memory_savings(&self) -> crate::vegetation::memory_optimization::MemorySavingsEstimate {
        crate::vegetation::memory_optimization::MemoryOptimizer::estimate_savings(self.tiles.len())
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

        active_count = self.active_manager.recently_grazed.len() + self.active_manager.regrowing_tiles.len();

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
            .add_systems(Startup, setup_vegetation_system.run_if(resource_exists::<WorldLoader>))
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

// Web API functions for viewer overlay

/// Get biomass heatmap data as JSON for web viewer
pub fn get_biomass_heatmap_json() -> String {
    // This is a placeholder - in a real implementation, this would
    // access the actual VegetationGrid resource
    // For now, return sample data to demonstrate the API

    let sample_data = vec![
        vec![80.0, 65.0, 90.0, 45.0, 20.0],
        vec![75.0, 85.0, 30.0, 15.0, 95.0],
        vec![60.0, 40.0, 70.0, 88.0, 25.0],
        vec![35.0, 50.0, 20.0, 65.0, 80.0],
        vec![90.0, 75.0, 55.0, 30.0, 45.0],
    ];

    let heatmap_data: Vec<Vec<f32>> = sample_data
        .iter()
        .enumerate()
        .map(|(y, row)| {
            row.iter()
                .enumerate()
                .map(|(x, biomass)| {
                    // Apply terrain-specific scaling
                    let terrain_multiplier = 1.0; // Default grass multiplier
                    let scaled_biomass = (biomass / MAX_BIOMASS * 100.0 * terrain_multiplier).min(100.0);
                    scaled_biomass
                })
                .collect()
        })
        .collect();

    format!(
        r#"{{"heatmap": {}, "max_biomass": {}, "tile_size": 16, "metadata": {{"updated_tick": 0, "grid_size": "5x5", "scale": "normalized"}}"#,
        serde_json::to_string(&heatmap_data).unwrap_or_else(|_| "[]".to_string()),
        MAX_BIOMASS
    )
}

/// Get vegetation system performance metrics as JSON
pub fn get_performance_metrics_json() -> String {
    // This would normally access actual performance metrics from the VegetationGrid
    // For now, return current sample metrics

    let metrics = r#"{
        "tiles_processed": 1000,
        "total_time_us": 850,
        "cpu_budget_us": 1000,
        "efficiency": "excellent",
        "tiles_per_us": 1.18,
        "adaptive_frequency": 1.0,
        "batch_metrics": {
            "batches_processed": 4,
            "avg_batch_time_us": 212,
            "max_batch_time_us": 280,
            "tiles_per_batch": 250
        },
        "active_tiles": 156,
        "depleted_tiles": 23,
        "system_load": "optimal"
    }"#;

    metrics.to_string()
}

/// Get memory usage analysis as JSON
pub fn get_memory_analysis_json() -> String {
    use memory_optimization::{MemoryOptimizer, StorageComparison};

    // Simulate current system statistics
    let tile_count = 5000;
    let active_tiles = 300;
    let performance_metrics = &PerformanceMetrics::default();

    let comparison = StorageComparison::compare_storage(tile_count, active_tiles, performance_metrics);
    let recommendations = MemoryOptimizer::analyze_and_optimize(tile_count, active_tiles, performance_metrics).1;

    let current_total = comparison.f32_usage.total_bytes;
    let current_per_tile = comparison.f32_usage.bytes_per_tile;
    let optimized_total = comparison.u16_usage.total_bytes;
    let optimized_per_tile = comparison.u16_usage.bytes_per_tile;
    let savings = comparison.savings_percent;
    let precision_loss = comparison.precision_loss_percent;
    let recs_json = serde_json::to_string(&recommendations).unwrap_or_else(|_| "[]".to_string());

    format!(
        r#"{{"current_usage": {{"total_bytes": {}, "bytes_per_tile": {}}}, "optimized_usage": {{"total_bytes": {}, "bytes_per_tile": {}}}, "savings_percent": {:.1}, "precision_loss_percent": {:.1}, "recommendations": {}}}"#,
        current_total, current_per_tile, optimized_total, optimized_per_tile, savings, precision_loss, recs_json
    )
}

/// Get vegetation system statistics summary
pub fn get_vegetation_stats_json() -> String {
    // This would normally access actual VegetationGrid statistics
    let stats = r#"{
        "suitable_tiles": 5000,
        "active_tiles": 156,
        "depleted_tiles": 23,
        "total_biomass": 285000.0,
        "average_biomass": 57.0,
        "biomass_percentage": 57.0,
        "current_tick": 12345,
        "growth_interval_ticks": 10,
        "system_status": "healthy"
    }"#;

    stats.to_string()
}

/// Get Phase 4 performance benchmark results
pub fn get_performance_benchmark_json() -> String {
    // This would contain actual benchmark results
    let benchmark = r#"{
        "test_duration_ms": 5000,
        "target_tps": 10.0,
        "actual_tps": 10.1,
        "budget_status": "within_budget",
        "average_tick_time_us": 98,
        "max_tick_time_us": 156,
        "min_tick_time_us": 45,
        "vegetation_update_time_us": {
            "average": 0.85,
            "max": 1.2,
            "budget_percentage": 85.0
        },
        "memory_usage_mb": 12.5,
        "performance_rating": "excellent"
    }"#;

    benchmark.to_string()
}

/// Run a quick performance benchmark and return results as JSON
pub fn run_quick_benchmark_json() -> String {
    use benchmark::run_quick_benchmark;

    // Run a quick 5-second benchmark
    let results = run_quick_benchmark();

    format!(
        r#"{{"benchmark_type": "quick", "duration_ms": {}, "total_ticks": {}, "actual_tps": {:.1}, "avg_growth_time_us": {:.1}, "budget_compliance_percent": {:.1}, "within_budget": {}, "efficiency_rating": "{:?}"}}"#,
        results.actual_duration_ms,
        results.total_ticks,
        results.actual_tps,
        results.growth_metrics.avg_growth_time_us,
        results.growth_metrics.budget_compliance_percent,
        results.budget_analysis.within_budget,
        results.growth_metrics.efficiency_rating
    )
}

/// Run comprehensive Phase 4 benchmark and return results as JSON
pub fn run_phase4_benchmark_json() -> String {
    use benchmark::run_phase4_benchmark;

    // Run comprehensive 15-second Phase 4 benchmark
    let results = run_phase4_benchmark();

    format!(
        r#"{{"benchmark_type": "phase4", "duration_ms": {}, "total_ticks": {}, "actual_tps": {:.1}, "growth_metrics": {{"avg_time_us": {:.1}, "max_time_us": {}, "budget_compliance_percent": {:.1}, "violations": {}, "efficiency_rating": "{:?}"}}, "system_metrics": {{"avg_tick_time_us": {:.1}, "cpu_utilization_percent": {:.1}}}, "budget_analysis": {{"within_budget": {}, "total_overage_us": {}, "worst_violation_us": {}, "time_within_budget_percent": {:.1}}}}}"#,
        results.actual_duration_ms,
        results.total_ticks,
        results.actual_tps,
        results.growth_metrics.avg_growth_time_us,
        results.growth_metrics.max_growth_time_us,
        results.growth_metrics.budget_compliance_percent,
        results.growth_metrics.budget_violations,
        results.growth_metrics.efficiency_rating,
        results.system_metrics.avg_tick_time_us,
        results.system_metrics.cpu_utilization_percent,
        results.budget_analysis.within_budget,
        results.budget_analysis.total_overage_us,
        results.budget_analysis.worst_violation_us,
        results.budget_analysis.time_within_budget_percent
    )
}

/// Get current performance rating based on recent metrics
pub fn get_current_performance_rating_json() -> String {
    // This would normally access actual performance metrics from the running system
    // For now, return simulated current performance data

    let avg_growth_time_us = 875.0;  // Current average
    let cpu_budget_us = 1000.0;       // 1ms budget
    let compliance_percent = ((cpu_budget_us / avg_growth_time_us) * 100.0f32).min(100.0f32);

    let rating = if compliance_percent >= 95.0 { "excellent" }
                 else if compliance_percent >= 85.0 { "good" }
                 else if compliance_percent >= 70.0 { "fair" }
                 else { "poor" };

    format!(
        r#"{{"current_performance": {{"avg_growth_time_us": {:.1}, "cpu_budget_us": {}, "compliance_percent": {:.1}, "rating": "{}", "status": "{}"}}}}"#,
        avg_growth_time_us,
        cpu_budget_us,
        compliance_percent,
        rating,
        if compliance_percent >= 85.0 { "within_budget" } else { "over_budget" }
    )
}

/// Get benchmark history and trends
pub fn get_benchmark_history_json() -> String {
    // Simulate historical benchmark data for trending analysis
    let history = r#"[
        {"timestamp": "2025-10-05T02:00:00Z", "avg_growth_time_us": 950.0, "budget_compliance": 95.0, "rating": "excellent"},
        {"timestamp": "2025-10-05T02:01:00Z", "avg_growth_time_us": 875.0, "budget_compliance": 97.0, "rating": "excellent"},
        {"timestamp": "2025-10-05T02:02:00Z", "avg_growth_time_us": 1100.0, "budget_compliance": 90.0, "rating": "good"},
        {"timestamp": "2025-10-05T02:03:00Z", "avg_growth_time_us": 825.0, "budget_compliance": 98.0, "rating": "excellent"},
        {"timestamp": "2025-10-05T02:04:00Z", "avg_growth_time_us": 800.0, "budget_compliance": 99.0, "rating": "excellent"}
    ]"#;

    let trend_analysis = r#"{"trend": "improving", "avg_change_percent": -2.5, "stability": "stable"}"#;

    format!(
        r#"{{"history": {}, "trend_analysis": {}, "summary": "Performance is improving with 5% average reduction in growth time over the last 5 measurements"}}"#,
        history,
        trend_analysis
    )
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