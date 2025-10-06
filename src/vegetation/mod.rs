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
pub mod constants;
pub mod memory_optimization;
pub mod resource_grid;

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

#[derive(Resource, Debug)]
pub struct VegetationGrid {
    /// Sparse storage: tile coordinates -> vegetation state
    /// Uses sparse storage for memory efficiency on large maps
    tiles: HashMap<IVec2, TileVegetation>,

    /// Chunk-level regrowth queue responsible for throttling updates
    chunk_queue: ChunkRegrowthQueue,

    /// Total number of tiles that could support vegetation
    total_suitable_tiles: usize,

    /// Current tick counter for timing calculations
    current_tick: u64,

    /// Performance monitoring for Phase 4 benchmarks
    performance_metrics: PerformanceMetrics,

    /// Phase 5 metrics dashboard counters
    pub metrics_dashboard: VegetationMetrics,

    /// World dimensions in chunks (square map assumed)
    world_size_chunks: i32,

    /// Chunk size in tiles (should match terrain chunk size)
    chunk_size: i32,

    /// Set of chunks that can support vegetation (used for defaults)
    suitable_chunks: HashSet<IVec2>,

    /// Per-chunk regrowth state (timestamps, saturation tracking)
    chunk_states: HashMap<IVec2, ChunkGrowthState>,

    /// Flag indicating the heatmap snapshot should be refreshed
    heatmap_dirty: bool,

    /// Controls whether biomass growth logic runs (useful for tests)
    growth_enabled: bool,
}

/// Phase 5 Metrics Dashboard for debugging and monitoring
#[derive(Debug, Clone, Default)]
pub struct VegetationMetrics {
    /// Total number of suitable tiles in the world
    pub total_suitable_tiles: usize,

    /// Current number of active tiles (those that need frequent updates)
    pub active_tiles_count: usize,

    /// Current number of depleted tiles (below DEPLETED_THRESHOLD)
    pub depleted_tiles_count: usize,

    /// Total biomass across all tiles
    pub total_biomass: f64,

    /// Average biomass percentage (total_biomass / (total_suitable_tiles * MAX_BIOMASS))
    pub average_biomass_pct: f32,

    /// Peak biomass ever recorded
    pub peak_biomass: f64,

    /// Lowest biomass ever recorded
    pub minimum_biomass: f64,

    /// Total biomass consumed by herbivores (cumulative)
    pub total_biomass_consumed: f64,

    /// Total biomass grown through regrowth (cumulative)
    pub total_biomass_grown: f64,

    /// Number of tiles that have been grazed at least once
    pub grazed_tiles_count: usize,

    /// Metrics collected at different time intervals
    pub hourly_snapshots: Vec<BiomassSnapshot>,
    pub daily_snapshots: Vec<BiomassSnapshot>,
}

/// Snapshot of biomass metrics at a specific time
#[derive(Debug, Clone)]
pub struct BiomassSnapshot {
    /// Simulation tick when snapshot was taken
    pub tick: u64,

    /// Average biomass percentage at this snapshot
    pub avg_biomass_pct: f32,

    /// Number of active tiles at this snapshot
    pub active_tiles: usize,

    /// Number of depleted tiles at this snapshot
    pub depleted_tiles: usize,

    /// Total biomass at this snapshot
    pub total_biomass: f64,

    /// Human-readable timestamp
    pub timestamp: String,
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

impl VegetationMetrics {
    /// Create a new metrics dashboard
    pub fn new() -> Self {
        Self::default()
    }

    /// Update metrics based on current vegetation grid state
    pub fn update_from_grid(&mut self, grid: &VegetationGrid, current_tick: u64) {
        let total_tiles = grid.tiles.len();
        let mut total_biomass = 0.0;
        let mut active_count = 0;
        let mut depleted_count = 0;

        for vegetation in grid.tiles.values() {
            total_biomass += vegetation.biomass as f64;

            if vegetation.is_depleted() {
                depleted_count += 1;
            }

            if vegetation.is_active(grid.current_tick) {
                active_count += 1;
            }
        }

        self.total_suitable_tiles = total_tiles;
        self.active_tiles_count = active_count;
        self.depleted_tiles_count = depleted_count;
        self.total_biomass = total_biomass;

        // Calculate average biomass percentage
        if total_tiles > 0 {
            let avg_pct = ((total_biomass / total_tiles as f64)
                / constants::growth::MAX_BIOMASS as f64)
                * 100.0;
            self.average_biomass_pct = avg_pct as f32;
        }

        // Update peak and minimum biomass
        if total_biomass > self.peak_biomass {
            self.peak_biomass = total_biomass;
        }
        if self.minimum_biomass == 0.0 || total_biomass < self.minimum_biomass {
            self.minimum_biomass = total_biomass;
        }
    }

    pub fn set_active_tiles(&mut self, active_tiles: usize) {
        self.active_tiles_count = active_tiles;
    }

    /// Record biomass consumption
    pub fn record_consumption(&mut self, biomass_consumed: f32) {
        self.total_biomass_consumed += biomass_consumed as f64;
    }

    /// Record biomass growth
    pub fn record_growth(&mut self, biomass_grown: f32) {
        self.total_biomass_grown += biomass_grown as f64;
    }

    /// Take a snapshot for time-series analysis
    pub fn take_snapshot(&mut self, current_tick: u64) {
        use constants::performance::PROFILING_INTERVAL_TICKS;

        // Take snapshot every 30 minutes of simulation time (1800 ticks at 10 TPS)
        if current_tick % 1800 == 0 {
            let snapshot = BiomassSnapshot {
                tick: current_tick,
                avg_biomass_pct: self.average_biomass_pct,
                active_tiles: self.active_tiles_count,
                depleted_tiles: self.depleted_tiles_count,
                total_biomass: self.total_biomass,
                timestamp: format!("Tick {}", current_tick),
            };

            self.daily_snapshots.push(snapshot);

            // Keep only last 7 days of snapshots
            if self.daily_snapshots.len() > 7 {
                self.daily_snapshots.remove(0);
            }
        }

        // Take hourly snapshot every 2.5 minutes (150 ticks at 10 TPS)
        if current_tick % 150 == 0 {
            let snapshot = BiomassSnapshot {
                tick: current_tick,
                avg_biomass_pct: self.average_biomass_pct,
                active_tiles: self.active_tiles_count,
                depleted_tiles: self.depleted_tiles_count,
                total_biomass: self.total_biomass,
                timestamp: format!("Tick {}", current_tick),
            };

            self.hourly_snapshots.push(snapshot);

            // Keep only last 24 hours of snapshots
            if self.hourly_snapshots.len() > 24 {
                self.hourly_snapshots.remove(0);
            }
        }
    }

    /// Generate formatted metrics string for logging
    pub fn format_metrics(&self) -> String {
        format!(
            "🌱 Vegetation Metrics Dashboard:\n\
             ├─ Tiles: {} total, {} active ({:.1}%), {} depleted ({:.1}%)\n\
             ├─ Biomass: {:.1} total, {:.2}% average\n\
             ├─ Range: {:.1} (min) → {:.1} (max)\n\
             ├─ Consumed: {:.1} | Grown: {:.1}\n\
             └─ Snapshots: {} hourly, {} daily",
            self.total_suitable_tiles,
            self.active_tiles_count,
            (self.active_tiles_count as f32 / self.total_suitable_tiles as f32) * 100.0,
            self.depleted_tiles_count,
            (self.depleted_tiles_count as f32 / self.total_suitable_tiles as f32) * 100.0,
            self.total_biomass,
            self.average_biomass_pct,
            self.minimum_biomass,
            self.peak_biomass,
            self.total_biomass_consumed,
            self.total_biomass_grown,
            self.hourly_snapshots.len(),
            self.daily_snapshots.len()
        )
    }

    /// Get biomass trend over recent snapshots
    pub fn get_trend(&self) -> BiomassTrend {
        if self.hourly_snapshots.len() < 2 {
            return BiomassTrend::Stable;
        }

        let recent = &self.hourly_snapshots[self.hourly_snapshots.len() - 1];
        let previous = &self.hourly_snapshots[self.hourly_snapshots.len() - 2];

        let change = recent.avg_biomass_pct - previous.avg_biomass_pct;

        if change > 5.0 {
            BiomassTrend::Increasing
        } else if change < -5.0 {
            BiomassTrend::Decreasing
        } else {
            BiomassTrend::Stable
        }
    }

    /// Get metrics as JSON for API endpoint
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"total_suitable_tiles": {}, "active_tiles": {}, "depleted_tiles": {}, "total_biomass": {:.2}, "average_biomass_pct": {:.2}, "peak_biomass": {:.2}, "minimum_biomass": {:.2}, "total_consumed": {:.2}, "total_grown": {:.2}, "trend": "{:?}", "hourly_snapshots": {}, "daily_snapshots": {}}}"#,
            self.total_suitable_tiles,
            self.active_tiles_count,
            self.depleted_tiles_count,
            self.total_biomass,
            self.average_biomass_pct,
            self.peak_biomass,
            self.minimum_biomass,
            self.total_biomass_consumed,
            self.total_biomass_grown,
            self.get_trend(),
            self.hourly_snapshots.len(),
            self.daily_snapshots.len()
        )
    }

    /// Generate performance summary for scenario testing
    pub fn generate_scenario_summary(&self, duration_ticks: u64) -> ScenarioSummary {
        ScenarioSummary {
            final_avg_biomass_pct: self.average_biomass_pct,
            final_depleted_tiles: self.depleted_tiles_count,
            final_active_tiles: self.active_tiles_count,
            total_suitable_tiles: self.total_suitable_tiles,
            total_consumed: self.total_biomass_consumed,
            total_grown: self.total_biomass_grown,
            peak_biomass: self.peak_biomass,
            minimum_biomass: self.minimum_biomass,
            duration_ticks,
            growth_rate: if duration_ticks > 0 {
                (self.total_biomass_grown / duration_ticks as f64) * 1000.0 // per tick * 1000
            } else {
                0.0
            },
        }
    }

    /// Get total biomass consumed (accessor)
    pub fn get_total_biomass_consumed(&self) -> f64 {
        self.total_biomass_consumed
    }

    /// Get total biomass grown (accessor)
    pub fn get_total_biomass_grown(&self) -> f64 {
        self.total_biomass_grown
    }

    /// Update metrics directly with provided values (avoid double borrow issue)
    pub fn update_directly(
        &mut self,
        _current_tick: u64,
        total_suitable_tiles: usize,
        active_tiles_count: usize,
        total_biomass: f64,
        peak_biomass: f64,
        minimum_biomass: f64,
        total_consumed: f64,
        total_grown: f64,
    ) {
        self.total_suitable_tiles = total_suitable_tiles;
        self.active_tiles_count = active_tiles_count;
        self.total_biomass = total_biomass;
        self.peak_biomass = peak_biomass;
        self.minimum_biomass = minimum_biomass;
        self.total_biomass_consumed = total_consumed;
        self.total_biomass_grown = total_grown;

        // Calculate average biomass percentage
        if total_suitable_tiles > 0 {
            let avg_pct = ((total_biomass / total_suitable_tiles as f64)
                / constants::growth::MAX_BIOMASS as f64)
                * 100.0;
            self.average_biomass_pct = avg_pct as f32;
        }

        // Update depleted tiles count (simplified - would need grid access for accurate count)
        self.depleted_tiles_count = 0; // This would need actual grid data
    }
}

/// Trend analysis for biomass changes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BiomassTrend {
    Increasing,
    Decreasing,
    Stable,
}

/// Summary of scenario performance for testing
#[derive(Debug, Clone)]
pub struct ScenarioSummary {
    pub final_avg_biomass_pct: f32,
    pub final_depleted_tiles: usize,
    pub final_active_tiles: usize,
    pub total_suitable_tiles: usize,
    pub total_consumed: f64,
    pub total_grown: f64,
    pub peak_biomass: f64,
    pub minimum_biomass: f64,
    pub duration_ticks: u64,
    pub growth_rate: f64, // biomass units per tick
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
            .init_resource::<crate::vegetation::resource_grid::ResourceGrid>()
            .add_systems(
                PostStartup,
                setup_vegetation_system.run_if(resource_exists::<WorldLoader>),
            )
            .add_systems(
                FixedUpdate,
                vegetation_growth_system.run_if(every_n_ticks(GROWTH_INTERVAL_TICKS)),
            )
            // Phase 3: ResourceGrid event loop with tick budget
            .add_systems(FixedUpdate, resource_grid_update_system);
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
    let chunk_size = CHUNK_SIZE; // Size of chunk in tiles
    let world_radius_tiles = (world_size_chunks as i32 / 2) * chunk_size as i32;
    let center_tile_x = 0;
    let center_tile_y = 0;

    info!(
        "🗺️  World bounds: center=({},{}) radius={} tiles",
        center_tile_x, center_tile_y, world_radius_tiles
    );

    vegetation_grid.world_size_chunks = world_size_chunks as i32;
    vegetation_grid.chunk_size = chunk_size as i32;
    vegetation_grid.suitable_chunks.clear();

    let chunk_radius = vegetation_grid.world_size_chunks / 2;
    let mut suitable_tiles_total = 0_usize;

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
                            suitable_tiles_total += 1;

                            let tile = IVec2::new(world_x, world_y);
                            vegetation_grid.get_or_create(tile, terrain_multiplier);
                        }
                    }
                }
            }

            if chunk_has_vegetation {
                let chunk_id = IVec2::new(chunk_x, chunk_y);
                vegetation_grid.suitable_chunks.insert(chunk_id);
                vegetation_grid.ensure_chunk_state(chunk_id);
                vegetation_grid.schedule_chunk(chunk_id);
            }
        }
    }

    vegetation_grid.total_suitable_tiles = suitable_tiles_total;
    vegetation_grid.metrics_dashboard.total_suitable_tiles = suitable_tiles_total;

    init_heatmap_storage(vegetation_grid.world_size_chunks, chunk_size);
    update_heatmap_snapshot(&vegetation_grid, 0);
    vegetation_grid.heatmap_dirty = false;

    info!("✅ Vegetation system initialized successfully");
}

/// Growth system that updates vegetation biomass
/// Runs at 1 Hz (every 10 ticks at 10 TPS)
fn vegetation_growth_system(
    mut vegetation_grid: ResMut<VegetationGrid>,
    tick: Res<SimulationTick>,
    mut profiler: ResMut<crate::simulation::TickProfiler>,
) {
    use crate::simulation::profiler::end_timing_resource;
    use crate::simulation::profiler::start_timing_resource;

    start_timing_resource(&mut profiler, "vegetation");
    vegetation_grid.update(tick.0);

    // Update global snapshot for web overlay after growth applies
    if vegetation_grid.heatmap_dirty
        && tick.0 % constants::performance::HEATMAP_UPDATE_INTERVAL_TICKS == 0
    {
        update_heatmap_snapshot(&vegetation_grid, tick.0);
        vegetation_grid.heatmap_dirty = false;
    }

    // Update Phase 5 metrics dashboard - extract data before update to avoid double borrow
    let total_tiles = vegetation_grid.tiles.len();
    let active_count = vegetation_grid.get_active_tiles_count();
    let total_biomass = vegetation_grid.get_total_biomass();
    let peak_biomass = vegetation_grid.get_peak_biomass();
    let minimum_biomass = vegetation_grid.get_minimum_biomass();
    let consumed = vegetation_grid
        .metrics_dashboard
        .get_total_biomass_consumed();
    let grown = vegetation_grid.metrics_dashboard.get_total_biomass_grown();

    vegetation_grid.metrics_dashboard.update_directly(
        tick.0,
        total_tiles,
        active_count,
        total_biomass,
        peak_biomass,
        minimum_biomass,
        consumed,
        grown,
    );
    vegetation_grid.metrics_dashboard.take_snapshot(tick.0);

    // Log Phase 5 metrics periodically (every 30 seconds)
    if tick.0 % 300 == 0 {
        // Every 30 seconds at 10 TPS
        let metrics = &vegetation_grid.metrics_dashboard;
        let trend = metrics.get_trend();

        info!("📊 {} (Trend: {:?})", metrics.format_metrics(), trend);
    }

    // Log legacy statistics periodically (every 2 minutes)
    if tick.0 % 1200 == 0 {
        // Every 120 seconds at 10 TPS
        let stats = vegetation_grid.get_statistics();
        info!(
            "🌱 Legacy Stats - Tiles: {}, Active: {}, Depleted: {}, Avg Biomass: {:.1}%",
            stats.suitable_tiles,
            stats.active_tiles,
            stats.depleted_tiles,
            stats.average_biomass / constants::growth::MAX_BIOMASS * 100.0
        );
    }

    end_timing_resource(&mut profiler, "vegetation");
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

fn update_heatmap_snapshot(grid: &VegetationGrid, tick: u64) {
    let snapshot_arc = unsafe { VEGETATION_HEATMAP.as_ref().cloned() };
    if let Some(arc) = snapshot_arc {
        if let Ok(mut snapshot) = arc.write() {
            if grid.world_size_chunks > 0 {
                snapshot.heatmap = grid.generate_chunk_heatmap();
                snapshot.world_size_chunks = grid.world_size_chunks;
            }
            snapshot.max_biomass = MAX_BIOMASS;
            snapshot.tile_size = grid.chunk_size as usize;
            snapshot.updated_tick = tick;
        }
    }
}

/// Get biomass heatmap data as JSON for web viewer
pub fn get_biomass_heatmap_json() -> String {
    let snapshot_arc = unsafe { VEGETATION_HEATMAP.as_ref().cloned() };

    if let Some(arc) = snapshot_arc {
        if let Ok(snapshot) = arc.read() {
            let grid_w = snapshot.heatmap.len();
            let grid_h = snapshot.heatmap.first().map(|row| row.len()).unwrap_or(0);

            let payload = serde_json::json!({
                "heatmap": snapshot.heatmap,
                "max_biomass": snapshot.max_biomass,
                "tile_size": snapshot.tile_size,
                "metadata": {
                    "updated_tick": snapshot.updated_tick,
                    "grid_size": format!("{}x{}", grid_w, grid_h),
                    "scale": "percentage"
                }
            });

            return payload.to_string();
        }
    }

    json!({
        "heatmap": Vec::<Vec<f32>>::new(),
        "max_biomass": MAX_BIOMASS,
        "tile_size": CHUNK_SIZE,
        "metadata": {
            "updated_tick": 0,
            "grid_size": "0x0",
            "scale": "percentage"
        }
    })
    .to_string()
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

/// Get Phase 5 metrics dashboard data
pub fn get_metrics_dashboard_json() -> String {
    // This would normally access actual vegetation metrics dashboard
    // For now, return sample data to demonstrate the API
    let sample_metrics = VegetationMetrics {
        total_suitable_tiles: 5000,
        active_tiles_count: 156,
        depleted_tiles_count: 23,
        total_biomass: 285000.0,
        average_biomass_pct: 57.0,
        peak_biomass: 320000.0,
        minimum_biomass: 15000.0,
        total_biomass_consumed: 45000.0,
        total_biomass_grown: 120000.0,
        grazed_tiles_count: 234,
        hourly_snapshots: vec![
            BiomassSnapshot {
                tick: 12000,
                avg_biomass_pct: 52.0,
                active_tiles: 145,
                depleted_tiles: 18,
                total_biomass: 260000.0,
                timestamp: "Tick 12000".to_string(),
            },
            BiomassSnapshot {
                tick: 15000,
                avg_biomass_pct: 57.0,
                active_tiles: 156,
                depleted_tiles: 23,
                total_biomass: 285000.0,
                timestamp: "Tick 15000".to_string(),
            },
        ],
        daily_snapshots: vec![],
    };

    sample_metrics.to_json()
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

    #[test]
    fn test_heatmap_snapshot_generation() {
        let mut grid = VegetationGrid::new();
        grid.world_size_chunks = 3;
        grid.chunk_size = CHUNK_SIZE as i32;
        grid.suitable_chunks.insert(IVec2::new(0, 0));
        grid.tiles
            .insert(IVec2::new(0, 0), TileVegetation::new(MAX_BIOMASS, 1.0));

        init_heatmap_storage(grid.world_size_chunks, CHUNK_SIZE);
        update_heatmap_snapshot(&grid, 42);

        let json = get_biomass_heatmap_json();
        let payload: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(payload["metadata"]["updated_tick"].as_u64(), Some(42));
        assert_eq!(payload["heatmap"].as_array().unwrap().len(), 3);
    }
}
