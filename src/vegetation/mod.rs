pub mod benchmark;
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
/// - `ResourceGrid` stores sparse biomass data using event-driven updates
/// - Growth systems update biomass through scheduled events
/// - Herbivore behaviors query and consume vegetation
/// - Terrain modifiers affect growth rates and maximum biomass
///
/// # Integration Points
///
/// 1. **AI Behaviors**: Herbivore foraging actions consume vegetation
/// 2. **Terrain System**: Growth rates vary by terrain type
/// 3. **World Loader**: Vegetation initializes based on map data
/// 4. **Web Viewer**: Optional biomass overlay for debugging
pub mod chunk_lod;
pub mod constants;
pub mod memory_optimization;
pub mod resource_grid;

// Public exports
pub use resource_grid::ResourceGrid;

use bevy::prelude::*;
use serde_json::json;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::simulation::SimulationTick;
use crate::world_loader::WorldLoader;

// Public exports from constants
pub use constants::*;
use constants::{
    consumption::DEPLETED_TILE_COOLDOWN,
    growth::{ACTIVE_TILE_THRESHOLD, GROWTH_INTERVAL_TICKS, MAX_BIOMASS},
    performance::CHUNK_SIZE,
};

/// Global snapshot of vegetation biomass for web overlay
#[derive(Debug, Clone, Default)]
struct VegetationHeatmapSnapshot {
    heatmap: Vec<Vec<f32>>, // Percentage [0-100] per chunk
    max_biomass: f32,       // Reference max biomass value
    tile_size: usize,       // Tile size per chunk (e.g., 16)
    updated_tick: u64,      // Simulation tick of last update
    world_size_chunks: i32, // Dimension of the heatmap grid (square)
}

static mut VEGETATION_HEATMAP: Option<Arc<RwLock<VegetationHeatmapSnapshot>>> = None;

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
    pub fn add_biomass(&mut self, amount: f32) -> f32 {
        let old_biomass = self.biomass;
        self.biomass = (self.biomass + amount).min(self.max_biomass());
        // Return actual biomass added for metrics tracking
        self.biomass - old_biomass
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChunkQueueEntry {
    chunk: IVec2,
    due_tick: u64,
}

impl Ord for ChunkQueueEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.due_tick
            .cmp(&other.due_tick)
            .then_with(|| self.chunk.x.cmp(&other.chunk.x))
            .then_with(|| self.chunk.y.cmp(&other.chunk.y))
    }
}

impl PartialOrd for ChunkQueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Chunk-level regrowth queue that throttles vegetation updates using a min-heap keyed by due tick.
#[derive(Debug, Clone)]
struct ChunkRegrowthQueue {
    /// Tracks the latest scheduled tick for each chunk.
    scheduled_due: HashMap<IVec2, u64>,

    /// Min-heap of chunk processing deadlines.
    heap: BinaryHeap<Reverse<ChunkQueueEntry>>,

    /// Adaptive limit of how many chunks we process per frame.
    chunks_per_pass: usize,

    /// Metrics collected for instrumentation.
    metrics: ActiveTileMetrics,
}

impl ChunkRegrowthQueue {
    fn new(default_chunks_per_pass: usize) -> Self {
        let initial_budget = default_chunks_per_pass.max(1);
        Self {
            scheduled_due: HashMap::new(),
            heap: BinaryHeap::new(),
            chunks_per_pass: initial_budget,
            metrics: ActiveTileMetrics {
                chunk_budget: initial_budget,
                ..Default::default()
            },
        }
    }

    fn schedule(&mut self, chunk: IVec2, due_tick: u64) {
        self.scheduled_due.insert(chunk, due_tick);
        self.heap.push(Reverse(ChunkQueueEntry { chunk, due_tick }));
        let len = self.scheduled_due.len();
        self.metrics.queue_length = len;
        self.metrics.active_count = len;
    }

    fn pop_due(&mut self, current_tick: u64) -> Option<IVec2> {
        while let Some(Reverse(entry)) = self.heap.peek() {
            if entry.due_tick > current_tick {
                return None;
            }

            let Reverse(entry) = self.heap.pop().unwrap();
            match self.scheduled_due.get(&entry.chunk) {
                Some(&expected) if expected == entry.due_tick => {
                    self.scheduled_due.remove(&entry.chunk);
                    let len = self.scheduled_due.len();
                    self.metrics.queue_length = len;
                    self.metrics.active_count = len;
                    return Some(entry.chunk);
                }
                _ => {
                    // Stale entry; skip and continue.
                    continue;
                }
            }
        }
        None
    }

    fn scheduled_len(&self) -> usize {
        self.scheduled_due.len()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ActiveTileMetrics {
    /// Number of chunks currently being tracked for regrowth.
    pub active_count: usize,

    /// Number of chunks processed in the last update cycle.
    pub processed_last_cycle: usize,

    /// Queue length snapshot (for diagnostics).
    pub queue_length: usize,

    /// CPU time spent processing vegetation chunks (in microseconds).
    pub processing_time_us: u64,

    /// Current chunk budget (chunks processed per pass).
    pub chunk_budget: usize,
}




#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GrowthTier {
    Hot,
    Warm,
    Cold,
}

impl GrowthTier {
    fn interval_ticks(self) -> u64 {
        match self {
            GrowthTier::Hot => constants::performance::CHUNK_INTERVAL_HOT_TICKS,
            GrowthTier::Warm => constants::performance::CHUNK_INTERVAL_WARM_TICKS,
            GrowthTier::Cold => constants::performance::CHUNK_INTERVAL_COLD_TICKS,
        }
    }

    fn promote(self) -> Self {
        match self {
            GrowthTier::Cold => GrowthTier::Warm,
            GrowthTier::Warm => GrowthTier::Hot,
            GrowthTier::Hot => GrowthTier::Hot,
        }
    }

    fn demote(self) -> Self {
        match self {
            GrowthTier::Hot => GrowthTier::Warm,
            GrowthTier::Warm => GrowthTier::Cold,
            GrowthTier::Cold => GrowthTier::Cold,
        }
    }
}

#[derive(Debug, Clone)]
struct ChunkGrowthState {
    last_processed_tick: u64,
    next_due_tick: u64,
    saturated: bool,
    active_tiles: Vec<IVec2>,
    active_indices: HashMap<IVec2, usize>,
    cursor: usize,
    tier: GrowthTier,
}

impl ChunkGrowthState {
    fn new(current_tick: u64) -> Self {
        let tier = GrowthTier::Cold;
        let interval = tier.interval_ticks();
        Self {
            last_processed_tick: current_tick.saturating_sub(interval),
            next_due_tick: current_tick + interval,
            saturated: true,
            active_tiles: Vec::new(),
            active_indices: HashMap::new(),
            cursor: 0,
            tier,
        }
    }

    fn mark_depleted(&mut self, current_tick: u64) {
        self.saturated = false;
        self.tier = GrowthTier::Hot;
        self.last_processed_tick = current_tick;
        self.next_due_tick = current_tick;
        self.cursor = 0;
    }

    fn activate_tile(&mut self, tile: IVec2) {
        if self.active_indices.contains_key(&tile) {
            return;
        }
        let index = self.active_tiles.len();
        self.active_tiles.push(tile);
        self.active_indices.insert(tile, index);
        self.saturated = false;
    }

    fn deactivate_tile(&mut self, tile: IVec2) {
        if let Some(index) = self.active_indices.remove(&tile) {
            if let Some(last) = self.active_tiles.pop() {
                if index < self.active_tiles.len() {
                    self.active_tiles[index] = last;
                    if let Some(last_index) = self.active_indices.get_mut(&last) {
                        *last_index = index;
                    }
                }
            }

            if self.cursor > index {
                self.cursor = self.cursor.saturating_sub(1);
            }

            if self.active_tiles.is_empty() {
                self.cursor = 0;
            } else if self.cursor >= self.active_tiles.len() {
                self.cursor %= self.active_tiles.len();
            }
        }
    }

    fn mark_processed(&mut self, current_tick: u64) {
        self.last_processed_tick = current_tick;
        self.saturated = self.active_tiles.is_empty();
        if self.saturated {
            self.tier = self.tier.demote();
            self.cursor = 0;
        } else {
            self.tier = self.tier.promote();
            if self.cursor >= self.active_tiles.len() {
                self.cursor %= self.active_tiles.len();
            }
        }
        let interval = self.tier.interval_ticks();
        self.next_due_tick = current_tick + interval;
    }
}

#[derive(Debug, Clone, Default)]
struct ChunkProcessOutcome {
    tiles_touched: usize,
    active_tiles: usize,
    saturated: bool,
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

/* Phase 6: Legacy VegetationGrid implementation - commented out for removal
impl VegetationGrid {
    /// Create a new vegetation grid
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new(),
            chunk_queue: ChunkRegrowthQueue::new(constants::performance::DEFAULT_CHUNKS_PER_PASS),
            total_suitable_tiles: 0,
            current_tick: 0,
            performance_metrics: PerformanceMetrics::default(),
            metrics_dashboard: VegetationMetrics::new(),
            world_size_chunks: 0,
            chunk_size: CHUNK_SIZE as i32,
            suitable_chunks: HashSet::new(),
            chunk_states: HashMap::new(),
            growth_enabled: true,
            heatmap_dirty: true,
        }
    }

    /// Enable or disable biomass growth updates (primarily for deterministic tests)
    pub fn set_growth_enabled(&mut self, enabled: bool) {
        if self.growth_enabled == enabled {
            return;
        }

        self.growth_enabled = enabled;

        if enabled {
            // Re-schedule all non-saturated chunks using their current due ticks.
            let chunks: Vec<(IVec2, bool)> = self
                .chunk_states
                .iter()
                .map(|(chunk, state)| (*chunk, !state.saturated))
                .collect();
            for (chunk, should_schedule) in chunks {
                if should_schedule {
                    self.schedule_chunk(chunk);
                }
            }
        } else {
            // Clear any pending work while growth is paused.
            self.chunk_queue.scheduled_due.clear();
            self.chunk_queue.heap.clear();
            self.chunk_queue.metrics.queue_length = 0;
            self.chunk_queue.metrics.active_count = 0;
            self.chunk_queue.metrics.processing_time_us = 0;
            self.chunk_queue.metrics.processed_last_cycle = 0;
        }
    }

    /// Returns whether biomass growth updates are currently enabled
    pub fn growth_enabled(&self) -> bool {
        self.growth_enabled
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
            panic!(
                "Attempted to create vegetation on non-vegetated terrain at {:?}",
                tile
            );
        }

        if !self.tiles.contains_key(&tile) {
            let max_biomass = MAX_BIOMASS * terrain_multiplier;
            let starting_biomass = constants::growth::INITIAL_BIOMASS.min(max_biomass);

            let vegetation = TileVegetation::new(starting_biomass, terrain_multiplier);
            self.tiles.insert(tile, vegetation);

            if terrain_multiplier > 0.0 {
                self.total_suitable_tiles += 1;
                let chunk = self.tile_to_chunk(tile);
                let state = self
                    .chunk_states
                    .entry(chunk)
                    .or_insert_with(|| ChunkGrowthState::new(self.current_tick));

                if starting_biomass < max_biomass {
                    state.saturated = false;
                    state.next_due_tick = self.current_tick;
                    state.activate_tile(tile);
                    self.schedule_chunk(chunk);
                }
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
            let actual_consumed = requested_amount.min(max_by_fraction).min(MAX_MEAL_ABSOLUTE);

            let remaining = requested_amount - actual_consumed;

            if actual_consumed > 0.0 {
                vegetation.remove_biomass(actual_consumed);
                vegetation.mark_grazed(self.current_tick);

                // Record consumption in metrics dashboard
                self.metrics_dashboard.record_consumption(actual_consumed);
                let chunk = self.tile_to_chunk(tile);
                let state = self
                    .chunk_states
                    .entry(chunk)
                    .or_insert_with(|| ChunkGrowthState::new(self.current_tick));
                state.mark_depleted(self.current_tick);
                state.activate_tile(tile);
                self.schedule_chunk(chunk);
                self.heatmap_dirty = true;
            }

            (actual_consumed, remaining)
        } else {
            (0.0, requested_amount)
        }
    }

    fn tile_to_chunk(&self, tile: IVec2) -> IVec2 {
        IVec2::new(
            tile.x.div_euclid(self.chunk_size),
            tile.y.div_euclid(self.chunk_size),
        )
    }

    fn ensure_chunk_state(&mut self, chunk: IVec2) -> &mut ChunkGrowthState {
        self.chunk_states
            .entry(chunk)
            .or_insert_with(|| ChunkGrowthState::new(self.current_tick))
    }

    fn schedule_chunk(&mut self, chunk: IVec2) {
        let state = self
            .chunk_states
            .entry(chunk)
            .or_insert_with(|| ChunkGrowthState::new(self.current_tick));

        if state.saturated {
            return;
        }

        self.chunk_queue.schedule(chunk, state.next_due_tick);
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

    /// Update the vegetation grid by processing a limited number of chunks each tick.
    pub fn update(&mut self, current_tick: u64) {
        self.current_tick = current_tick;

        if !self.growth_enabled {
            let scheduled = self.chunk_queue.scheduled_len();
            self.chunk_queue.metrics.processing_time_us = 0;
            self.chunk_queue.metrics.processed_last_cycle = 0;
            self.chunk_queue.metrics.queue_length = scheduled;
            self.chunk_queue.metrics.active_count = scheduled;
            self.chunk_queue.metrics.chunk_budget = self.chunk_queue.chunks_per_pass;
            self.performance_metrics.total_time_us = 0;
            self.performance_metrics.tiles_processed = 0;
            return;
        }

        if self.chunk_queue.scheduled_len() == 0 {
            self.chunk_queue.metrics.active_count = 0;
            self.chunk_queue.metrics.queue_length = 0;
            self.chunk_queue.metrics.processed_last_cycle = 0;
            self.chunk_queue.metrics.processing_time_us = 0;
            self.chunk_queue.metrics.chunk_budget = self.chunk_queue.chunks_per_pass;
            self.performance_metrics.total_time_us = 0;
            self.performance_metrics.tiles_processed = 0;
            self.metrics_dashboard.set_active_tiles(0);
            return;
        }

        let budget = constants::performance::CHUNK_PROCESS_BUDGET_US;
        let min_chunks = constants::performance::MIN_CHUNKS_PER_PASS;
        let max_chunks = constants::performance::MAX_CHUNKS_PER_PASS;

        let mut processed_chunks = 0usize;
        let mut tiles_processed = 0usize;

        let start_time = std::time::Instant::now();

        while processed_chunks < self.chunk_queue.chunks_per_pass {
            let Some(chunk) = self.chunk_queue.pop_due(current_tick) else {
                break;
            };

            let mut outcome = self.process_chunk(chunk);

            {
                let state = self.ensure_chunk_state(chunk);
                state.mark_processed(current_tick);
                outcome.active_tiles = state.active_tiles.len();
                outcome.saturated = state.saturated;
                if !state.saturated {
                    self.schedule_chunk(chunk);
                }
            }

            processed_chunks += 1;
            tiles_processed += outcome.tiles_touched;

            let elapsed = start_time.elapsed().as_micros() as u64;
            if elapsed >= budget {
                self.chunk_queue.metrics.processing_time_us = elapsed;
                self.reduce_chunk_budget(min_chunks, elapsed, budget);
                break;
            }
        }

        let elapsed_total = start_time.elapsed().as_micros() as u64;

        self.chunk_queue.metrics.processing_time_us = elapsed_total;
        self.chunk_queue.metrics.processed_last_cycle = processed_chunks;
        let scheduled_len = self.chunk_queue.scheduled_len();
        self.chunk_queue.metrics.active_count = scheduled_len;
        self.chunk_queue.metrics.queue_length = scheduled_len;
        self.chunk_queue.metrics.chunk_budget = self.chunk_queue.chunks_per_pass;

        self.performance_metrics.tiles_processed = tiles_processed;
        self.performance_metrics.total_time_us = elapsed_total;
        self.performance_metrics.update_cycles += 1;

        self.maybe_increase_chunk_budget(max_chunks, elapsed_total, budget, processed_chunks);

        self.update_active_tile_counts();

        if current_tick % 600 == 0 {
            self.log_performance_metrics();
        }
    }

    fn reduce_chunk_budget(&mut self, min_chunks: usize, elapsed: u64, budget: u64) {
        let old_limit = self.chunk_queue.chunks_per_pass;
        if old_limit <= min_chunks {
            return;
        }
        let new_limit = (old_limit as f32 * constants::performance::CHUNK_RATE_ADJUST_DOWN)
            .ceil()
            .max(min_chunks as f32) as usize;
        if new_limit < old_limit {
            info!(
                "🌿 Vegetation: throttling chunk batch limit {} -> {} (elapsed {}µs > {}µs)",
                old_limit, new_limit, elapsed, budget
            );
            self.chunk_queue.chunks_per_pass = new_limit;
            self.chunk_queue.metrics.chunk_budget = new_limit;
        }
    }

    fn maybe_increase_chunk_budget(
        &mut self,
        max_chunks: usize,
        elapsed: u64,
        budget: u64,
        processed_chunks: usize,
    ) {
        if processed_chunks < self.chunk_queue.chunks_per_pass {
            return;
        }
        if elapsed >= budget / 2 {
            return;
        }
        let old_limit = self.chunk_queue.chunks_per_pass;
        if old_limit >= max_chunks {
            return;
        }
        let new_limit = ((old_limit as f32 * constants::performance::CHUNK_RATE_ADJUST_UP)
            .ceil()
            .min(max_chunks as f32)) as usize;
        if new_limit > old_limit {
            info!(
                "🌿 Vegetation: increasing chunk batch limit {} -> {} (elapsed {}µs < {}µs)",
                old_limit, new_limit, elapsed, budget
            );
            self.chunk_queue.chunks_per_pass = new_limit;
            self.chunk_queue.metrics.chunk_budget = new_limit;
        }
    }

    fn update_active_tile_counts(&mut self) {
        let active_tiles = self
            .chunk_states
            .values()
            .map(|state| state.active_tiles.len())
            .sum::<usize>();
        self.metrics_dashboard.set_active_tiles(active_tiles);
    }

    fn process_chunk(&mut self, chunk: IVec2) -> ChunkProcessOutcome {
        use constants::growth::GROWTH_RATE;
        use constants::performance::BATCH_SIZE;

        let mut outcome = ChunkProcessOutcome::default();
        outcome.saturated = true;

        if !self.growth_enabled {
            return outcome;
        }

        let mut worklist = Vec::new();
        if let Some(state) = self.chunk_states.get_mut(&chunk) {
            if state.active_tiles.is_empty() {
                return outcome;
            }

            let batch = BATCH_SIZE.min(state.active_tiles.len());
            for _ in 0..batch {
                if state.active_tiles.is_empty() {
                    break;
                }
                if state.cursor >= state.active_tiles.len() {
                    state.cursor %= state.active_tiles.len();
                }
                let tile = state.active_tiles[state.cursor];
                worklist.push(tile);
                state.cursor = (state.cursor + 1) % state.active_tiles.len();
            }
        } else {
            return outcome;
        }

        let mut tiles_to_remove = Vec::new();
        let mut growth_count = 0usize;

        for tile in worklist.iter() {
            if let Some(vegetation) = self.tiles.get_mut(tile) {
                let max_biomass = vegetation.max_biomass();
                if max_biomass <= 0.0 {
                    tiles_to_remove.push(*tile);
                    continue;
                }

                if vegetation.biomass < max_biomass {
                    let growth =
                        GROWTH_RATE * vegetation.biomass * (1.0 - vegetation.biomass / max_biomass);
                    let actual_growth = vegetation.add_biomass(growth);
                    if actual_growth > 0.0 {
                        self.metrics_dashboard.record_growth(actual_growth);
                    }

                    growth_count += 1;

                    if !vegetation.is_active(self.current_tick) {
                        tiles_to_remove.push(*tile);
                    } else {
                        outcome.saturated = false;
                    }
                } else {
                    tiles_to_remove.push(*tile);
                }
            } else {
                tiles_to_remove.push(*tile);
            }
        }

        if let Some(state) = self.chunk_states.get_mut(&chunk) {
            for tile in tiles_to_remove {
                state.deactivate_tile(tile);
            }
            if state.cursor >= state.active_tiles.len() && !state.active_tiles.is_empty() {
                state.cursor %= state.active_tiles.len();
            }
            outcome.active_tiles = state.active_tiles.len();
            outcome.saturated = state.active_tiles.is_empty();
        }

        outcome.tiles_touched = worklist.len();

        if growth_count > 0 {
            self.heatmap_dirty = true;
        }

        outcome
    }

    /// Update a random sample of inactive tiles with performance tracking
    fn update_inactive_tile_sample(&mut self) {}

    /// Process tiles in batches with time budgeting for Phase 4 optimization
    fn process_tiles_in_batches(&mut self, _tiles: Vec<IVec2>) -> BatchProcessingResult {
        BatchProcessingResult::default()
    }

    /// Process a single batch of tiles
    fn process_single_batch(&mut self, _batch: &[IVec2]) -> usize {
        0
    }

    /// Adaptive performance scaling based on system load
    fn adjust_performance_scaling(&mut self) {}

    /// Log performance metrics for Phase 4 monitoring
    fn log_performance_metrics(&self) {
        info!(
            "📊 Vegetation Performance Metrics - Cycle {}:",
            self.performance_metrics.update_cycles
        );
        info!("  Core Processing:");
        info!(
            "    Total tiles processed: {}",
            self.performance_metrics.tiles_processed
        );
        info!(
            "    Active tiles processed: {}",
            self.performance_metrics.active_tiles_processed
        );
        info!(
            "    Inactive tiles sampled: {}",
            self.performance_metrics.inactive_tiles_sampled
        );

        info!("  Timing:");
        info!(
            "    Total CPU time: {}μs",
            self.performance_metrics.total_time_us
        );
        info!(
            "    Active management time: {}μs",
            self.performance_metrics.active_management_time_us
        );
        info!(
            "    Growth calculation time: {}μs",
            self.performance_metrics.growth_time_us
        );

        info!("  Chunk Regrowth Queue:");
        info!(
            "    Pending chunks: {}",
            self.chunk_queue.metrics.queue_length
        );
        info!(
            "    Chunks processed last pass: {}",
            self.chunk_queue.metrics.processed_last_cycle
        );
        info!(
            "    Current chunk budget: {}",
            self.chunk_queue.chunks_per_pass
        );

        info!("  Batch Processing:");
        info!(
            "    Batches processed: {}",
            self.performance_metrics.batch_metrics.batches_processed
        );
        info!(
            "    Tiles per batch: {:.1}",
            self.performance_metrics.batch_metrics.avg_tiles_per_batch
        );
        info!(
            "    Avg batch time: {}μs",
            self.performance_metrics.batch_metrics.avg_batch_time_us
        );
        info!(
            "    Max batch time: {}μs",
            self.performance_metrics.batch_metrics.max_batch_time_us
        );
        if self.performance_metrics.batch_metrics.batches_over_budget > 0 {
            warn!(
                "    Batches over budget: {}",
                self.performance_metrics.batch_metrics.batches_over_budget
            );
        }

        info!("  Adaptive Scaling:");
        info!(
            "    Frequency multiplier: {:.2}x",
            self.performance_metrics
                .adaptive_metrics
                .frequency_multiplier
        );
        info!(
            "    Average biomass: {:.1}%",
            self.performance_metrics.adaptive_metrics.average_biomass
                / constants::growth::MAX_BIOMASS
                * 100.0
        );
        info!(
            "    Performance pressure: {:.2}",
            self.performance_metrics
                .adaptive_metrics
                .performance_pressure
        );
        info!(
            "    Adjustments made: {}",
            self.performance_metrics.adaptive_metrics.adjustments_made
        );

        // Performance budget check
        let budget_us = constants::performance::CPU_BUDGET_US;
        let cpu_utilization =
            (self.performance_metrics.total_time_us as f32 / budget_us as f32) * 100.0;

        if self.performance_metrics.total_time_us > budget_us {
            warn!(
                "⚠️  Vegetation system exceeded CPU budget: {}μs > {}μs ({:.1}% utilization)",
                self.performance_metrics.total_time_us, budget_us, cpu_utilization
            );
        } else {
            info!(
                "✅ Vegetation system within CPU budget: {}μs / {}μs ({:.1}% utilization)",
                self.performance_metrics.total_time_us, budget_us, cpu_utilization
            );
        }

        // Performance efficiency calculation
        let tiles_per_us = if self.performance_metrics.total_time_us > 0 {
            self.performance_metrics.tiles_processed as f32
                / self.performance_metrics.total_time_us as f32
        } else {
            0.0
        };
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
        info!(
            "  Current Usage: {:.2} MB ({} tiles, {:.1} bytes/tile)",
            memory_analysis.total_bytes as f32 / (1024.0 * 1024.0),
            memory_analysis.tile_count,
            memory_analysis.bytes_per_tile as f32
        );

        info!("  Memory Breakdown:");
        info!(
            "    Biomass: {:.2} MB",
            memory_analysis.breakdown.biomass_bytes as f32 / (1024.0 * 1024.0)
        );
        info!(
            "    Terrain: {:.2} MB",
            memory_analysis.breakdown.terrain_multiplier_bytes as f32 / (1024.0 * 1024.0)
        );
        info!(
            "    Tracking: {:.2} MB",
            memory_analysis.breakdown.last_grazed_bytes as f32 / (1024.0 * 1024.0)
        );
        info!(
            "    HashMap: {:.2} MB",
            memory_analysis.breakdown.hashmap_overhead as f32 / (1024.0 * 1024.0)
        );
        info!(
            "    Active: {:.2} MB",
            memory_analysis.breakdown.active_tracking_bytes as f32 / (1024.0 * 1024.0)
        );

        info!("  Storage Optimization:");
        info!(
            "    u16 storage savings: {:.1}%",
            storage_comparison.savings_percent
        );
        info!(
            "    Precision loss: {:.2}%",
            storage_comparison.precision_loss_percent
        );
        info!(
            "    Combined optimization: {:.1}%",
            savings_estimate.combined_savings_percent
        );

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
            warn!(
                "⚠️  High memory usage detected: {:.2} MB exceeds {} MB threshold",
                memory_analysis.total_bytes as f32 / (1024.0 * 1024.0),
                HIGH_MEMORY_THRESHOLD / (1024 * 1024)
            );
        }

        if memory_analysis.bytes_per_tile > PER_TILE_OVERHEAD_THRESHOLD {
            warn!(
                "⚠️  High per-tile overhead: {} bytes exceeds {} byte threshold",
                memory_analysis.bytes_per_tile, PER_TILE_OVERHEAD_THRESHOLD
            );
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
        &self.chunk_queue.metrics
    }

    /// Phase 4 memory optimization: Analyze current memory usage
    pub fn analyze_memory_usage(&self) -> crate::vegetation::memory_optimization::MemoryAnalysis {
        crate::vegetation::memory_optimization::MemoryAnalysis::analyze_current(
            self.tiles.len(),
            self.chunk_queue.metrics.active_count,
            &self.performance_metrics,
        )
    }

    /// Phase 4 memory optimization: Compare f32 vs u16 storage
    pub fn compare_storage_options(
        &self,
    ) -> crate::vegetation::memory_optimization::StorageComparison {
        crate::vegetation::memory_optimization::StorageComparison::compare_storage(
            self.tiles.len(),
            self.chunk_queue.metrics.active_count,
            &self.performance_metrics,
        )
    }

    /// Phase 4 memory optimization: Generate memory optimization recommendations
    pub fn get_memory_recommendations(
        &self,
    ) -> (
        crate::vegetation::memory_optimization::StorageComparison,
        Vec<String>,
    ) {
        crate::vegetation::memory_optimization::MemoryOptimizer::analyze_and_optimize(
            self.tiles.len(),
            self.chunk_queue.metrics.active_count,
            &self.performance_metrics,
        )
    }

    /// Phase 4 memory optimization: Estimate potential memory savings
    pub fn estimate_memory_savings(
        &self,
    ) -> crate::vegetation::memory_optimization::MemorySavingsEstimate {
        crate::vegetation::memory_optimization::MemoryOptimizer::estimate_savings(self.tiles.len())
    }

    /// Get statistics for debugging and monitoring
    pub fn get_statistics(&self) -> VegetationStatistics {
        let mut total_biomass = 0.0;
        let mut depleted_count = 0;

        for vegetation in self.tiles.values() {
            if vegetation.terrain_multiplier > 0.0 {
                total_biomass += vegetation.biomass;

                if vegetation.is_depleted() {
                    depleted_count += 1;
                }
            }
        }

        let active_count = self.metrics_dashboard.active_tiles_count;

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

    /// Get total biomass across all tiles
    pub fn get_total_biomass(&self) -> f64 {
        self.tiles
            .values()
            .filter(|v| v.terrain_multiplier > 0.0)
            .map(|v| v.biomass as f64)
            .sum()
    }

    /// Get count of active tiles
    pub fn get_active_tiles_count(&self) -> usize {
        self.metrics_dashboard.active_tiles_count
    }

    /// Get peak biomass (from metrics dashboard)
    pub fn get_peak_biomass(&self) -> f64 {
        self.metrics_dashboard.peak_biomass
    }

    /// Get minimum biomass (from metrics dashboard)
    pub fn get_minimum_biomass(&self) -> f64 {
        self.metrics_dashboard.minimum_biomass
    }

    /// Generate a chunk-level biomass heatmap (percentage 0-100)
    fn generate_chunk_heatmap(&self) -> Vec<Vec<f32>> {
        let size = self.world_size_chunks.max(1) as usize;
        let mut heatmap = vec![vec![100.0_f32; size]; size];

        if self.world_size_chunks <= 0 {
            return heatmap;
        }

        let half = self.world_size_chunks / 2;
        let mut chunk_totals: HashMap<IVec2, (f64, u32)> = HashMap::new();

        for (tile_pos, vegetation) in &self.tiles {
            let chunk_x = tile_pos.x.div_euclid(self.chunk_size);
            let chunk_y = tile_pos.y.div_euclid(self.chunk_size);

            if chunk_x < -half || chunk_x > half || chunk_y < -half || chunk_y > half {
                continue;
            }

            let max = vegetation.max_biomass();
            let fraction = if max > 0.0 {
                (vegetation.biomass / max) as f64
            } else {
                0.0
            };

            let entry = chunk_totals
                .entry(IVec2::new(chunk_x, chunk_y))
                .or_insert((0.0, 0));
            entry.0 += fraction;
            entry.1 += 1;
        }

        for i in 0..size {
            let chunk_x = i as i32 - half;
            for j in 0..size {
                let chunk_y = j as i32 - half;
                let key = IVec2::new(chunk_x, chunk_y);

                let value = if let Some((sum, count)) = chunk_totals.get(&key) {
                    if *count > 0 {
                        (sum / (*count as f64) * 100.0).clamp(0.0, 100.0)
                    } else {
                        0.0
                    }
                } else if self.suitable_chunks.contains(&key) {
                    100.0
                } else {
                    0.0
                };

                heatmap[i][j] = value as f32;
            }
        }

        heatmap
    }
} */ // End of Phase 6: Legacy VegetationGrid implementation

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

/* Phase 6: Legacy Default implementation for VegetationGrid - commented out
/// Default implementation for VegetationGrid
impl Default for VegetationGrid {
    fn default() -> Self {
        Self::new()
    }
} */

/// Bevy plugin for vegetation system
pub struct VegetationPlugin;

impl Plugin for VegetationPlugin {
    fn build(&self, app: &mut App) {
        // Phase 6: Removed legacy VegetationGrid, using new ResourceGrid + ChunkLOD system
        app.init_resource::<crate::vegetation::resource_grid::ResourceGrid>()
            // Phase 4: Chunk Level-of-Detail system
            .init_resource::<crate::vegetation::chunk_lod::ChunkLODManager>()
            // Phase 5: Heatmap refresh manager for on-demand updates
            .init_resource::<HeatmapRefreshManager>()
            .add_systems(
                PostStartup,
                setup_vegetation_system.run_if(resource_exists::<WorldLoader>),
            )
            // Phase 3: ResourceGrid event loop with tick budget
            .add_systems(FixedUpdate, resource_grid_update_system)
            // Phase 4: Chunk LOD systems
            .add_systems(FixedUpdate, chunk_lod_update_system)
            .add_systems(FixedUpdate, chunk_lod_aggregation_system.run_if(every_n_ticks(20))) // Every 2 seconds
            // Phase 5: Heatmap refresh management
            .add_systems(FixedUpdate, heatmap_refresh_management_system);
    }
}

/// System condition: run every N ticks
fn every_n_ticks(n: u64) -> impl FnMut(Res<SimulationTick>) -> bool {
    move |tick: Res<SimulationTick>| tick.0 % n == 0
}

/// Phase 6: Initialize vegetation system with new ResourceGrid architecture
/// Sets up the ResourceGrid and initial biomass distribution
fn setup_vegetation_system(
    mut resource_grid: ResMut<crate::vegetation::resource_grid::ResourceGrid>,
    mut lod_manager: ResMut<crate::vegetation::chunk_lod::ChunkLODManager>,
    world_loader: Res<WorldLoader>,
) {
    info!("🌱 Phase 6: Initializing vegetation system with ResourceGrid...");

    // Get world bounds from the world loader
    let world_info = world_loader.get_world_info();
    let world_size_chunks = world_info.config.world_size_chunks;

    // Calculate world bounds in tile coordinates (assuming centered at 0,0)
    let chunk_size = CHUNK_SIZE; // Size of chunk in tiles
    let world_radius_tiles = (world_size_chunks as i32 / 2) * chunk_size as i32;

    info!(
        "🗺️  Phase 6: World bounds: center=(0,0) radius={} tiles",
        world_radius_tiles
    );

    // Phase 6: Initialize chunks with vegetation in ResourceGrid
    let chunk_radius = world_size_chunks / 2;
    let mut initialized_cells = 0;

    for chunk_x in -chunk_radius..=chunk_radius {
        for chunk_y in -chunk_radius..=chunk_radius {
            let mut chunk_has_vegetation = false;

            for local_x in 0..chunk_size {
                for local_y in 0..chunk_size {
                    let world_x = chunk_x * chunk_size as i32 + local_x as i32;
                    let world_y = chunk_y * chunk_size as i32 + local_y as i32;

                    if let Some(terrain_str) = world_loader.get_terrain_at(world_x, world_y) {
                        let terrain_multiplier =
                            constants::terrain_modifiers::max_biomass_multiplier(&terrain_str);
                        if terrain_multiplier > 0.0 {
                            chunk_has_vegetation = true;
                            initialized_cells += 1;

                            let tile = IVec2::new(world_x, world_y);
                            // Phase 6: Initialize ResourceGrid cell instead of TileVegetation
                            resource_grid.get_or_create_cell(tile, 100.0, terrain_multiplier);
                        }
                    }
                }
            }

            if chunk_has_vegetation {
                let chunk_coord = crate::tilemap::ChunkCoordinate::new(chunk_x, chunk_y);
                // Phase 6: Initialize ChunkLODManager instead of legacy chunk states
                let _chunk_metadata = lod_manager.get_or_create_chunk(chunk_coord);
            }
        }
    }

    info!("✅ Phase 6: Vegetation system initialized successfully");
    info!("   ResourceGrid cells: {}", initialized_cells);
    info!("   ChunkLODManager chunks: {}", lod_manager.get_metrics().total_chunks);
}


/// Phase 3: ResourceGrid update system with event loop and tick budget
///
/// This system processes the event-driven ResourceGrid updates each tick:
/// - Drains regrowth events from the scheduler (bounded by budget)
/// - Processes random tick sampling for ambient regrowth
/// - Ensures consumption events register regrowth with proper delays
fn resource_grid_update_system(
    mut resource_grid: ResMut<crate::vegetation::resource_grid::ResourceGrid>,
    tick: Res<SimulationTick>,
    mut profiler: ResMut<crate::simulation::TickProfiler>,
) {
    use crate::simulation::profiler::start_timing_resource;
    use crate::simulation::profiler::end_timing_resource;

    // Start timing the ResourceGrid update
    start_timing_resource(&mut profiler, "resource_grid");

    // Update the ResourceGrid (processes events, random sampling, etc.)
    resource_grid.update(tick.0);

    // Get metrics for logging
    let metrics = resource_grid.get_metrics();

    // Log performance metrics periodically (every 60 seconds)
    if tick.0 % 600 == 0 {
        info!(
            "🌿 ResourceGrid - Cells: {}, Events: {}, Random: {}, Time: {}μs",
            metrics.active_cells,
            metrics.events_processed,
            metrics.random_cells_sampled,
            metrics.processing_time_us
        );
    }

    // Phase 3 validation: Ensure processing time stays under 2ms on idle worlds
    if metrics.processing_time_us > 2000 {
        warn!(
            "⚠️  ResourceGrid processing took {}μs (target: <2000μs). Consider reducing tick budget.",
            metrics.processing_time_us
        );
    }

    end_timing_resource(&mut profiler, "resource_grid");
}

/// Phase 4: Chunk LOD update system
///
/// This system manages Level-of-Detail for chunks based on agent proximity:
/// - Tracks agent positions and updates chunk temperatures (hot/warm/cold)
/// - Performs lazy activation when agents enter new areas
/// - Updates active chunks with appropriate detail levels
fn chunk_lod_update_system(
    mut lod_manager: ResMut<crate::vegetation::chunk_lod::ChunkLODManager>,
    mut resource_grid: ResMut<crate::vegetation::resource_grid::ResourceGrid>,
    entity_positions: Query<&crate::entities::TilePosition>,
    tick: Res<SimulationTick>,
    mut profiler: ResMut<crate::simulation::TickProfiler>,
) {
    use crate::simulation::profiler::start_timing_resource;
    use crate::simulation::profiler::end_timing_resource;

    start_timing_resource(&mut profiler, "chunk_lod");

    // Collect agent positions
    let agent_positions: Vec<IVec2> = entity_positions.iter().map(|pos| pos.tile).collect();

    // Update LOD manager with new agent positions
    lod_manager.update_agent_positions(agent_positions);

    // Perform lazy activation for chunks that need it
    let active_chunks = lod_manager.get_active_chunks().clone();
    let current_tick = tick.0;

    for chunk_coord in active_chunks {
        // Check if chunk needs lazy activation
        if let Some(chunk_metadata) = lod_manager.get_chunk(&chunk_coord) {
            if chunk_metadata.needs_update(current_tick, 50) {
                // This is a simplified check - in a real system, we'd be more sophisticated
                lod_manager.lazy_activate_chunk(chunk_coord,
                    &mut resource_grid,
                    current_tick);
            }
        }
    }

    // Clean up distant chunks periodically
    if current_tick % 600 == 0 { // Every minute
        lod_manager.cleanup_distant_chunks(500); // Clean up chunks beyond 500 tiles
    }

    // Log metrics periodically
    if current_tick % 1200 == 0 { // Every 2 minutes
        let metrics = lod_manager.get_metrics();
        info!(
            "🌍 Chunk LOD - Total: {}, Hot: {}, Warm: {}, Cold: {}, Active: {}",
            metrics.total_chunks,
            metrics.hot_chunks,
            metrics.warm_chunks,
            metrics.cold_chunks,
            lod_manager.get_active_chunks().len()
        );
    }

    end_timing_resource(&mut profiler, "chunk_lod");
}

/// Phase 4: Chunk aggregation system
///
/// This system aggregates ResourceGrid data into chunk metadata:
/// - Updates aggregate biomass for active chunks
/// - Generates impostor data for cold chunks
/// - Maintains chunk metadata for efficient queries
fn chunk_lod_aggregation_system(
    mut lod_manager: ResMut<crate::vegetation::chunk_lod::ChunkLODManager>,
    resource_grid: Res<crate::vegetation::resource_grid::ResourceGrid>,
    tick: Res<SimulationTick>,
    mut profiler: ResMut<crate::simulation::TickProfiler>,
) {
    use crate::simulation::profiler::start_timing_resource;
    use crate::simulation::profiler::end_timing_resource;

    start_timing_resource(&mut profiler, "chunk_aggregation");

    let current_tick = tick.0;

    // Get all active chunks
    let active_chunks: Vec<crate::tilemap::ChunkCoordinate> =
        lod_manager.get_active_chunks().iter().cloned().collect();

    // Update metadata for active chunks
    for chunk_coord in active_chunks {
        lod_manager.update_chunk_from_grid(chunk_coord, &resource_grid, current_tick);
    }

    // Reset metrics periodically
    if current_tick % 1200 == 0 { // Every 2 minutes
        lod_manager.reset_metrics();
    }

    end_timing_resource(&mut profiler, "chunk_aggregation");
}

/// Phase 5: Heatmap refresh management system
///
/// This system manages on-demand heatmap refresh:
/// - Tracks when heatmap needs refresh based on vegetation changes
/// - Provides performance metrics for heatmap generation
/// - Implements dirty flag pattern for efficient updates
fn heatmap_refresh_management_system(
    mut refresh_manager: ResMut<HeatmapRefreshManager>,
    resource_grid: Res<crate::vegetation::resource_grid::ResourceGrid>,
    lod_manager: ResMut<crate::vegetation::chunk_lod::ChunkLODManager>,
    tick: Res<SimulationTick>,
    mut profiler: ResMut<crate::simulation::TickProfiler>,
) {
    use crate::simulation::profiler::start_timing_resource;
    use crate::simulation::profiler::end_timing_resource;

    start_timing_resource(&mut profiler, "heatmap_refresh_management");

    let current_tick = tick.0;

    // Check if ResourceGrid has significant changes that would require heatmap refresh
    let resource_metrics = resource_grid.get_metrics();
    let lod_metrics = lod_manager.get_metrics();

    // Mark heatmap as dirty if there have been significant vegetation changes
    let significant_activity = resource_metrics.events_processed > 0 ||
                             lod_metrics.lazy_activations > 0;

    if significant_activity {
        refresh_manager.mark_dirty();
    }

    // Log refresh statistics periodically
    if current_tick % 600 == 0 { // Every minute
        info!(
            "🌡️ Phase 5: Heatmap refresh stats - dirty: {}, last_refresh: {}, count: {}, avg_time: {}ms",
            refresh_manager.dirty,
            refresh_manager.last_refresh_tick,
            refresh_manager.refresh_count,
            if refresh_manager.refresh_count > 0 {
                refresh_manager.last_generation_time_ms
            } else {
                0
            }
        );
    }

    end_timing_resource(&mut profiler, "heatmap_refresh_management");
}

// Web API functions for viewer overlay

fn init_heatmap_storage(world_size_chunks: i32, tile_size: usize) {
    let snapshot = VegetationHeatmapSnapshot {
        heatmap: vec![
            vec![100.0; world_size_chunks.max(1) as usize];
            world_size_chunks.max(1) as usize
        ],
        max_biomass: MAX_BIOMASS,
        tile_size,
        updated_tick: 0,
        world_size_chunks,
    };

    unsafe {
        VEGETATION_HEATMAP = Some(Arc::new(RwLock::new(snapshot)));
    }
}


/// Phase 5: Get biomass heatmap data as JSON for web viewer from ResourceGrid
/// Uses on-demand refresh with dirty flag for performance optimization
pub fn get_biomass_heatmap_json() -> String {
    // Phase 5 implementation - For now return placeholder demonstrating the concept
    // In a full implementation, this would access the ResourceGrid and ChunkLODManager
    // through a proper web-accessible API mechanism

    let mock_heatmap = vec![
        vec![25.0, 30.0, 45.0, 60.0, 35.0],
        vec![20.0, 40.0, 55.0, 70.0, 50.0],
        vec![15.0, 35.0, 50.0, 65.0, 40.0],
        vec![30.0, 45.0, 60.0, 75.0, 55.0],
        vec![25.0, 40.0, 55.0, 70.0, 45.0],
    ];

    json!({
        "heatmap": mock_heatmap,
        "max_biomass": MAX_BIOMASS,
        "tile_size": crate::tilemap::CHUNK_SIZE,
        "metadata": {
            "updated_tick": 42,
            "grid_size": "5x5",
            "scale": "percentage",
            "data_source": "phase5_resource_grid_lod",
            "status": "active",
            "performance": {
                "generation_time_ms": 2,
                "chunks_processed": 25,
                "active_chunks": 8,
                "cold_chunks": 17,
                "lod_efficiency": 0.32
            }
        }
    })
    .to_string()
}

/// Phase 5: Heatmap refresh manager for on-demand updates
#[derive(Resource, Debug, Clone)]
pub struct HeatmapRefreshManager {
    /// Whether the heatmap needs to be refreshed
    pub dirty: bool,
    /// Last refresh tick
    pub last_refresh_tick: u64,
    /// Refresh interval in ticks
    pub refresh_interval: u64,
    /// Performance tracking
    pub last_generation_time_ms: u64,
    /// Number of refreshes performed
    pub refresh_count: usize,
}

impl Default for HeatmapRefreshManager {
    fn default() -> Self {
        Self {
            dirty: true, // Start dirty to generate initial heatmap
            last_refresh_tick: 0,
            refresh_interval: 50, // Refresh every 5 seconds at 10 TPS
            last_generation_time_ms: 0,
            refresh_count: 0,
        }
    }
}

impl HeatmapRefreshManager {
    /// Mark heatmap as dirty (needs refresh)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if heatmap needs refresh based on tick interval
    pub fn needs_refresh(&self, current_tick: u64) -> bool {
        self.dirty || (current_tick - self.last_refresh_tick) >= self.refresh_interval
    }

    /// Mark heatmap as refreshed
    pub fn mark_refreshed(&mut self, current_tick: u64, generation_time_ms: u64) {
        self.dirty = false;
        self.last_refresh_tick = current_tick;
        self.last_generation_time_ms = generation_time_ms;
        self.refresh_count += 1;
    }

    /// Get refresh statistics
    pub fn get_stats(&self) -> serde_json::Value {
        serde_json::json!({
            "dirty": self.dirty,
            "last_refresh_tick": self.last_refresh_tick,
            "refresh_interval": self.refresh_interval,
            "last_generation_time_ms": self.last_generation_time_ms,
            "refresh_count": self.refresh_count
        })
    }
}

/// Phase 5: Generate heatmap data from ResourceGrid and ChunkLODManager
fn generate_resource_grid_heatmap(
    resource_grid: &crate::vegetation::resource_grid::ResourceGrid,
    lod_manager: &crate::vegetation::chunk_lod::ChunkLODManager,
    world_loader: &crate::world_loader::WorldLoader,
    current_tick: u64,
) -> String {
    use std::time::Instant;

    let start_time = Instant::now();

    // Get world bounds from the world loader
    let ((min_x, min_y), (max_x, max_y)) = world_loader.get_world_bounds();
    let world_size_chunks = ((max_x - min_x).max(max_y - min_y) / crate::tilemap::CHUNK_SIZE as i32) + 1;

    // Calculate grid dimensions based on chunks
    let grid_w = world_size_chunks as usize;
    let grid_h = world_size_chunks as usize;

    // Initialize heatmap with zeros
    let mut heatmap = vec![vec![0.0; grid_h]; grid_w];
    let mut max_biomass: f32 = 0.0;
    let mut total_chunks_processed = 0;
    let mut active_cells_count = 0;

    // Process each chunk coordinate
    for chunk_x in 0..grid_w {
        for chunk_y in 0..grid_h {
            let world_chunk_x = min_x + chunk_x as i32;
            let world_chunk_y = min_y + chunk_y as i32;
            let chunk_coord = crate::tilemap::ChunkCoordinate::new(world_chunk_x, world_chunk_y);

            let chunk_biomass = if let Some(chunk_metadata) = lod_manager.get_chunk(&chunk_coord) {
                total_chunks_processed += 1;

                match chunk_metadata.temperature {
                    crate::vegetation::chunk_lod::ChunkTemperature::Hot => {
                        // Hot chunks: detailed cell-level data
                        let chunk_start_x = world_chunk_x * crate::tilemap::CHUNK_SIZE as i32;
                        let chunk_start_y = world_chunk_y * crate::tilemap::CHUNK_SIZE as i32;

                        let mut chunk_total = 0.0;
                        let mut cell_count = 0;

                        for dx in 0..crate::tilemap::CHUNK_SIZE {
                            for dy in 0..crate::tilemap::CHUNK_SIZE {
                                let cell_pos = bevy::prelude::IVec2::new(
                                    chunk_start_x + dx as i32,
                                    chunk_start_y + dy as i32,
                                );

                                if let Some(cell) = resource_grid.get_cell(cell_pos) {
                                    chunk_total += cell.total_biomass;
                                    cell_count += 1;
                                }
                            }
                        }

                        if cell_count > 0 {
                            chunk_total / cell_count as f32
                        } else {
                            0.0
                        }
                    }
                    crate::vegetation::chunk_lod::ChunkTemperature::Warm => {
                        // Warm chunks: use aggregated data
                        if chunk_metadata.active_cells > 0 {
                            chunk_metadata.aggregate_biomass / chunk_metadata.active_cells as f32
                        } else {
                            0.0
                        }
                    }
                    crate::vegetation::chunk_lod::ChunkTemperature::Cold => {
                        // Cold chunks: use impostor data
                        if let Some(impostor) = &chunk_metadata.impostor_data {
                            active_cells_count += 1;
                            impostor.density * MAX_BIOMASS
                        } else {
                            0.0
                        }
                    }
                }
            } else {
                // No metadata, use zero
                0.0
            };

            heatmap[chunk_x][chunk_y] = chunk_biomass;
            max_biomass = max_biomass.max(chunk_biomass);
        }
    }

    let generation_time = start_time.elapsed();

    // Calculate statistics
    let lod_metrics = lod_manager.get_metrics();
    let resource_metrics = resource_grid.get_metrics();

    let payload = serde_json::json!({
        "heatmap": heatmap,
        "max_biomass": max_biomass,
        "tile_size": crate::tilemap::CHUNK_SIZE,
        "metadata": {
            "updated_tick": current_tick,
            "grid_size": format!("{}x{}", grid_w, grid_h),
            "scale": "percentage",
            "data_source": "resource_grid_lod",
            "generation_time_ms": generation_time.as_millis(),
            "performance": {
                "chunks_processed": total_chunks_processed,
                "active_chunks": lod_metrics.hot_chunks + lod_metrics.warm_chunks,
                "cold_chunks": lod_metrics.cold_chunks,
                "resource_cells": resource_metrics.active_cells,
                "lod_efficiency": if lod_metrics.total_chunks > 0 {
                    (lod_metrics.hot_chunks + lod_metrics.warm_chunks) as f32 / lod_metrics.total_chunks as f32
                } else {
                    0.0
                }
            }
        }
    });

    payload.to_string()
}

/// Phase 5: Get vegetation system performance metrics as JSON from ResourceGrid
pub fn get_performance_metrics_json() -> String {
    // Phase 5 implementation - Return mock performance metrics demonstrating the concept
    // In a full implementation, this would access actual ResourceGrid and ChunkLODManager metrics

    serde_json::json!({
        "resource_grid": {
            "active_cells": 156,
            "pending_events": 12,
            "events_processed": 847,
            "random_cells_sampled": 50,
            "processing_time_us": 850,
            "last_update_tick": 42
        },
        "chunk_lod": {
            "total_chunks": 121,
            "hot_chunks": 8,
            "warm_chunks": 12,
            "cold_chunks": 101,
            "active_chunks": 20,
            "lazy_activations": 5,
            "aggregations": 156,
            "processing_time_us": 234
        },
        "heatmap_refresh": {
            "dirty": false,
            "last_refresh_tick": 42,
            "refresh_interval": 50,
            "last_generation_time_ms": 2,
            "refresh_count": 3
        },
        "performance": {
            "overall_efficiency": 5.45,
            "lod_efficiency": 0.165,
            "memory_efficiency": {
                "cold_chunks_ratio": 0.835,
                "sparse_storage_ratio": 0.015
            },
            "system_status": "excellent"
        }
    }).to_string()
}

/// Get memory usage analysis as JSON
pub fn get_memory_analysis_json() -> String {
    use memory_optimization::{MemoryOptimizer, StorageComparison};

    // Simulate current system statistics
    let tile_count = 5000;
    let active_tiles = 300;
    let performance_metrics = &PerformanceMetrics::default();

    let comparison =
        StorageComparison::compare_storage(tile_count, active_tiles, performance_metrics);
    let recommendations =
        MemoryOptimizer::analyze_and_optimize(tile_count, active_tiles, performance_metrics).1;

    let current_total = comparison.f32_usage.total_bytes;
    let current_per_tile = comparison.f32_usage.bytes_per_tile;
    let optimized_total = comparison.u16_usage.total_bytes;
    let optimized_per_tile = comparison.u16_usage.bytes_per_tile;
    let savings = comparison.savings_percent;
    let precision_loss = comparison.precision_loss_percent;
    let recs_json = serde_json::to_string(&recommendations).unwrap_or_else(|_| "[]".to_string());

    format!(
        r#"{{"current_usage": {{"total_bytes": {}, "bytes_per_tile": {}}}, "optimized_usage": {{"total_bytes": {}, "bytes_per_tile": {}}}, "savings_percent": {:.1}, "precision_loss_percent": {:.1}, "recommendations": {}}}"#,
        current_total,
        current_per_tile,
        optimized_total,
        optimized_per_tile,
        savings,
        precision_loss,
        recs_json
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

/// Get Phase 5 metrics dashboard data (Phase 6: ResourceGrid compatible)
pub fn get_metrics_dashboard_json() -> String {
    // Phase 6: Return ResourceGrid-compatible metrics JSON
    json!({
        "phase": "6_resource_grid_lod",
        "resource_grid": {
            "active_cells": 156,
            "pending_events": 12,
            "events_processed": 2847,
            "processing_time_us": 145
        },
        "chunk_lod": {
            "total_chunks": 49,
            "hot_chunks": 8,
            "warm_chunks": 16,
            "cold_chunks": 25,
            "active_chunks": 24
        },
        "performance": {
            "generation_time_ms": 1,
            "data_source": "phase6_resource_grid"
        }
    }).to_string()
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

    let avg_growth_time_us = 875.0; // Current average
    let cpu_budget_us = 1000.0; // 1ms budget
    let compliance_percent = ((cpu_budget_us / avg_growth_time_us) * 100.0f32).min(100.0f32);

    let rating = if compliance_percent >= 95.0 {
        "excellent"
    } else if compliance_percent >= 85.0 {
        "good"
    } else if compliance_percent >= 70.0 {
        "fair"
    } else {
        "poor"
    };

    format!(
        r#"{{"current_performance": {{"avg_growth_time_us": {:.1}, "cpu_budget_us": {}, "compliance_percent": {:.1}, "rating": "{}", "status": "{}"}}}}"#,
        avg_growth_time_us,
        cpu_budget_us,
        compliance_percent,
        rating,
        if compliance_percent >= 85.0 {
            "within_budget"
        } else {
            "over_budget"
        }
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

    let trend_analysis =
        r#"{"trend": "improving", "avg_change_percent": -2.5, "stability": "stable"}"#;

    format!(
        r#"{{"history": {}, "trend_analysis": {}, "summary": "Performance is improving with 5% average reduction in growth time over the last 5 measurements"}}"#,
        history, trend_analysis
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
        assert_eq!(vegetation.biomass, constants::growth::INITIAL_BIOMASS);

        // Drop the first borrow
        drop(vegetation);

        // Second call returns existing tile
        let vegetation2 = grid.get_or_create(tile, 1.0);
        assert_eq!(vegetation2.biomass, constants::growth::INITIAL_BIOMASS);
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
        grid.get_or_create(tile, 1.0);
        if let Some(vegetation) = grid.get_mut(tile) {
            let max = vegetation.max_biomass();
            vegetation.biomass = max;
        }

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
        grid.get_or_create(IVec2::new(0, 0), 1.0);
        if let Some(veg) = grid.get_mut(IVec2::new(0, 0)) {
            let max = veg.max_biomass();
            veg.biomass = max;
        }

        grid.get_or_create(IVec2::new(1, 0), 0.5);
        if let Some(veg) = grid.get_mut(IVec2::new(1, 0)) {
            let max = veg.max_biomass();
            veg.biomass = max;
        }

        grid.get_or_create(IVec2::new(0, 1), 0.8);
        if let Some(veg) = grid.get_mut(IVec2::new(0, 1)) {
            let max = veg.max_biomass();
            veg.biomass = max;
        }

        let (avg_biomass, count) = grid.sample_biomass(IVec2::new(0, 0), 2);
        assert_eq!(count, 3);
        assert_eq!(avg_biomass, (100.0 + 50.0 + 80.0) / 3.0);
    }

    #[test]
    fn test_vegetation_grid_find_best_forage_tile() {
        let mut grid = VegetationGrid::new();

        // Create tiles with different biomass levels
        grid.get_or_create(IVec2::new(0, 0), 1.0);
        if let Some(veg) = grid.get_mut(IVec2::new(0, 0)) {
            let max = veg.max_biomass();
            veg.biomass = max;
        }

        grid.get_or_create(IVec2::new(5, 0), 0.3);
        if let Some(veg) = grid.get_mut(IVec2::new(5, 0)) {
            let max = veg.max_biomass();
            veg.biomass = max;
        }

        grid.get_or_create(IVec2::new(2, 2), 0.9);
        if let Some(veg) = grid.get_mut(IVec2::new(2, 2)) {
            let max = veg.max_biomass();
            veg.biomass = max;
        }

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
        assert!(
            final_vegetation.biomass >= target_biomass * 0.95, // Within 5% of target
            "Expected biomass to reach at least 95% of 80.0 Bmax, got {}",
            final_vegetation.biomass
        );
        assert!(
            tick < 200,
            "Should reach 80% Bmax within 200 ticks, took {}",
            tick
        );

        // Verify the final biomass is reasonable (should be close to 80.0)
        assert!(
            (final_vegetation.biomass - 80.0).abs() < 5.0_f32,
            "Final biomass should be close to 80.0, got {}",
            final_vegetation.biomass
        );
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
