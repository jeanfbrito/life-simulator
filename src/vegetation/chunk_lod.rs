//! Phase 4: Chunk Level-of-Detail (LOD) System
//!
//! This module implements Level-of-Detail chunk management for the vegetation system:
//! - Tracks active chunks based on proximity to agents (hot/warm/cold)
//! - Stores chunk metadata with aggregate biomass
//! - Implements lazy activation between detail levels
//! - Provides far-range impostor data for web overlay

use crate::tilemap::{ChunkCoordinate, CHUNK_SIZE};
use crate::vegetation::resource_grid::{GrazingCell, ResourceGrid};
use bevy::log::warn;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// Extension trait for IVec2 to get max element
trait IVec2Ext {
    fn max_element(&self) -> i32;
}

impl IVec2Ext for IVec2 {
    fn max_element(&self) -> i32 {
        self.x.abs().max(self.y.abs())
    }
}

/// Chunk temperature levels for LOD management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkTemperature {
    /// Hot: Full per-cell detail (within 100 tiles of agents)
    Hot,
    /// Warm: Aggregate detail (100-200 tiles from agents)
    Warm,
    /// Cold: Impostor only (beyond 200 tiles from agents)
    Cold,
}

impl ChunkTemperature {
    /// Get the distance range for this temperature level
    pub fn distance_range(&self) -> (i32, i32) {
        match self {
            ChunkTemperature::Hot => (0, 100),
            ChunkTemperature::Warm => (100, 200),
            ChunkTemperature::Cold => (200, i32::MAX),
        }
    }

    /// Check if a distance falls within this temperature level
    pub fn contains_distance(&self, distance: i32) -> bool {
        let (min, max) = self.distance_range();
        distance >= min && distance < max
    }
}

/// Chunk metadata with aggregated biomass data
#[derive(Debug, Clone)]
pub struct ChunkMetadata {
    /// Chunk coordinate
    pub coordinate: ChunkCoordinate,

    /// Current temperature level
    pub temperature: ChunkTemperature,

    /// Aggregate biomass for all cells in this chunk
    pub aggregate_biomass: f32,

    /// Number of active cells in this chunk
    pub active_cells: usize,

    /// Maximum possible biomass for this chunk
    pub max_biomass: f32,

    /// Average growth rate modifier for this chunk
    pub avg_growth_rate: f32,

    /// Last update tick
    pub last_update_tick: u64,

    /// Impostor data for cold chunks (color/density)
    pub impostor_data: Option<ChunkImpostor>,
}

/// Impostor data for far-range chunks
#[derive(Debug, Clone)]
pub struct ChunkImpostor {
    /// Average biomass density (0.0 to 1.0)
    pub density: f32,

    /// Dominant vegetation color (RGB)
    pub color: [u8; 3],

    /// Visual representation quality level
    pub quality: ImpostorQuality,
}

/// Impostor quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpostorQuality {
    /// Low quality: basic color/density
    Low,
    /// Medium quality: slightly more detail
    Medium,
    /// High quality: near-visual fidelity
    High,
}

impl ChunkMetadata {
    /// Create new metadata for a chunk
    pub fn new(coordinate: ChunkCoordinate, temperature: ChunkTemperature) -> Self {
        Self {
            coordinate,
            temperature,
            aggregate_biomass: 0.0,
            active_cells: 0,
            max_biomass: 0.0,
            avg_growth_rate: 1.0,
            last_update_tick: 0,
            impostor_data: None,
        }
    }

    /// Update aggregate data from ResourceGrid cells
    pub fn update_from_cells(&mut self, cells: &[(IVec2, &GrazingCell)], current_tick: u64) {
        self.aggregate_biomass = cells.iter().map(|(_, cell)| cell.total_biomass).sum();
        self.active_cells = cells.len();
        self.max_biomass = cells.iter().map(|(_, cell)| cell.max_biomass).sum();
        self.avg_growth_rate = if cells.is_empty() {
            1.0
        } else {
            cells
                .iter()
                .map(|(_, cell)| cell.growth_rate_modifier)
                .sum::<f32>()
                / cells.len() as f32
        };
        self.last_update_tick = current_tick;

        // Generate impostor data if this is a cold chunk
        if self.temperature == ChunkTemperature::Cold {
            self.impostor_data = Some(self.generate_impostor());
        }
    }

    /// Generate impostor data from aggregate information
    pub fn generate_impostor(&self) -> ChunkImpostor {
        let density = if self.max_biomass > 0.0 {
            (self.aggregate_biomass / self.max_biomass).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Generate color based on biomass density
        let color = if density > 0.7 {
            [34, 139, 34] // Dark green
        } else if density > 0.4 {
            [50, 205, 50] // Medium green
        } else if density > 0.2 {
            [144, 238, 144] // Light green
        } else {
            [189, 183, 107] // Dry grass
        };

        let quality = match self.temperature {
            ChunkTemperature::Hot => ImpostorQuality::High,
            ChunkTemperature::Warm => ImpostorQuality::Medium,
            ChunkTemperature::Cold => ImpostorQuality::Low,
        };

        ChunkImpostor {
            density,
            color,
            quality,
        }
    }

    /// Get biomass density as percentage (0.0 to 1.0)
    pub fn biomass_density(&self) -> f32 {
        if self.max_biomass > 0.0 {
            (self.aggregate_biomass / self.max_biomass).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Check if this chunk needs updating based on age
    pub fn needs_update(&self, current_tick: u64, update_interval: u64) -> bool {
        match self.temperature {
            ChunkTemperature::Hot => current_tick - self.last_update_tick >= update_interval / 2, // Hot chunks update more frequently
            ChunkTemperature::Warm => current_tick - self.last_update_tick >= update_interval,
            ChunkTemperature::Cold => false, // Cold chunks only update on activation
        }
    }
}

/// Chunk LOD manager for handling level-of-detail transitions
#[derive(Resource, Debug)]
pub struct ChunkLODManager {
    /// Metadata for all tracked chunks
    pub chunks: HashMap<ChunkCoordinate, ChunkMetadata>,

    /// Set of currently active chunks (hot or warm)
    active_chunks: HashSet<ChunkCoordinate>,

    /// Agent positions for proximity tracking
    agent_positions: Vec<IVec2>,

    /// Configuration settings
    config: ChunkLODConfig,

    /// Performance metrics
    metrics: ChunkLODMetrics,
}

/// Configuration for chunk LOD management
#[derive(Debug, Clone)]
pub struct ChunkLODConfig {
    /// Hot chunk radius (tiles)
    pub hot_radius: i32,

    /// Warm chunk radius (tiles)
    pub warm_radius: i32,

    /// Update interval for warm chunks (ticks)
    pub warm_update_interval: u64,

    /// Update interval for hot chunks (ticks)
    pub hot_update_interval: u64,

    /// Maximum number of active chunks to track
    pub max_active_chunks: usize,
}

impl Default for ChunkLODConfig {
    fn default() -> Self {
        Self {
            hot_radius: 100,
            warm_radius: 200,
            warm_update_interval: 100, // 10 seconds at 10 TPS
            hot_update_interval: 50,   // 5 seconds at 10 TPS
            max_active_chunks: 1000,
        }
    }
}

/// Performance metrics for chunk LOD system
#[derive(Debug, Clone, Default)]
pub struct ChunkLODMetrics {
    /// Total number of tracked chunks
    pub total_chunks: usize,

    /// Number of hot chunks
    pub hot_chunks: usize,

    /// Number of warm chunks
    pub warm_chunks: usize,

    /// Number of cold chunks
    pub cold_chunks: usize,

    /// Number of lazy activations performed
    pub lazy_activations: usize,

    /// Number of aggregations performed
    pub aggregations: usize,

    /// CPU time spent processing (microseconds)
    pub processing_time_us: u64,
}

impl ChunkLODManager {
    /// Create new chunk LOD manager
    pub fn new(config: ChunkLODConfig) -> Self {
        Self {
            chunks: HashMap::new(),
            active_chunks: HashSet::new(),
            agent_positions: Vec::new(),
            config,
            metrics: ChunkLODMetrics::default(),
        }
    }

    /// Update agent positions and recalculate chunk temperatures
    pub fn update_agent_positions(&mut self, agent_positions: Vec<IVec2>) {
        self.agent_positions = agent_positions;
        self.recalculate_chunk_temperatures();
    }

    /// Recalculate temperature for all tracked chunks based on agent proximity
    fn recalculate_chunk_temperatures(&mut self) {
        let start_time = std::time::Instant::now();

        // Clear active chunks
        self.active_chunks.clear();

        // Collect temperature updates first to avoid borrow checker issues
        let mut temperature_updates = Vec::new();
        for (coord, _metadata) in self.chunks.iter() {
            let new_temperature = self.calculate_chunk_temperature(*coord);
            temperature_updates.push((*coord, new_temperature));
        }

        // Apply temperature updates
        for (coord, new_temperature) in temperature_updates {
            if let Some(metadata) = self.chunks.get_mut(&coord) {
                let old_temperature = metadata.temperature;
                metadata.temperature = new_temperature;

                // Track active chunks (hot or warm)
                if matches!(
                    new_temperature,
                    ChunkTemperature::Hot | ChunkTemperature::Warm
                ) {
                    self.active_chunks.insert(coord);
                }

                // Update impostor data for temperature changes
                if old_temperature != new_temperature {
                    if new_temperature == ChunkTemperature::Cold {
                        metadata.impostor_data = Some(metadata.generate_impostor());
                    } else {
                        metadata.impostor_data = None;
                    }
                }
            }
        }

        // Update metrics
        let elapsed = start_time.elapsed().as_micros() as u64;
        self.metrics.processing_time_us += elapsed;
        self.update_temperature_metrics();
    }

    /// Calculate temperature for a chunk based on distance to nearest agent
    fn calculate_chunk_temperature(&self, chunk_coord: ChunkCoordinate) -> ChunkTemperature {
        // Convert chunk coordinate to world position (center of chunk)
        let chunk_center_x = chunk_coord.x * CHUNK_SIZE as i32 + CHUNK_SIZE as i32 / 2;
        let chunk_center_y = chunk_coord.y * CHUNK_SIZE as i32 + CHUNK_SIZE as i32 / 2;
        let chunk_pos = IVec2::new(chunk_center_x, chunk_center_y);

        // Find minimum distance to any agent
        let min_distance = self
            .agent_positions
            .iter()
            .map(|&agent_pos| (agent_pos - chunk_pos).abs().max_element())
            .min()
            .unwrap_or(i32::MAX);

        // Determine temperature based on distance
        if ChunkTemperature::Hot.contains_distance(min_distance) {
            ChunkTemperature::Hot
        } else if ChunkTemperature::Warm.contains_distance(min_distance) {
            ChunkTemperature::Warm
        } else {
            ChunkTemperature::Cold
        }
    }

    /// Update temperature metrics
    fn update_temperature_metrics(&mut self) {
        self.metrics.total_chunks = self.chunks.len();
        self.metrics.hot_chunks = 0;
        self.metrics.warm_chunks = 0;
        self.metrics.cold_chunks = 0;

        for metadata in self.chunks.values() {
            match metadata.temperature {
                ChunkTemperature::Hot => self.metrics.hot_chunks += 1,
                ChunkTemperature::Warm => self.metrics.warm_chunks += 1,
                ChunkTemperature::Cold => self.metrics.cold_chunks += 1,
            }
        }
    }

    /// Get or create metadata for a chunk
    pub fn get_or_create_chunk(&mut self, coordinate: ChunkCoordinate) -> &mut ChunkMetadata {
        if !self.chunks.contains_key(&coordinate) {
            let temperature = self.calculate_chunk_temperature(coordinate);
            let metadata = ChunkMetadata::new(coordinate, temperature);
            self.chunks.insert(coordinate, metadata);

            if matches!(temperature, ChunkTemperature::Hot | ChunkTemperature::Warm) {
                self.active_chunks.insert(coordinate);
            }
        }

        self.chunks.get_mut(&coordinate).unwrap()
    }

    /// Update chunk metadata from ResourceGrid data
    pub fn update_chunk_from_grid(
        &mut self,
        coordinate: ChunkCoordinate,
        grid: &ResourceGrid,
        current_tick: u64,
    ) {
        if let Some(metadata) = self.chunks.get_mut(&coordinate) {
            // Collect cells in this chunk
            let chunk_start = IVec2::new(
                coordinate.x * CHUNK_SIZE as i32,
                coordinate.y * CHUNK_SIZE as i32,
            );

            let mut cells = Vec::new();
            for dx in 0..CHUNK_SIZE as i32 {
                for dy in 0..CHUNK_SIZE as i32 {
                    let cell_pos = chunk_start + IVec2::new(dx, dy);
                    if let Some(cell) = grid.get_cell(cell_pos) {
                        cells.push((cell_pos, cell));
                    }
                }
            }

            metadata.update_from_cells(&cells, current_tick);
            self.metrics.aggregations += 1;
        }
    }

    /// Perform lazy activation for a chunk (convert from aggregate to fine detail)
    pub fn lazy_activate_chunk(
        &mut self,
        coordinate: ChunkCoordinate,
        grid: &mut ResourceGrid,
        _current_tick: u64,
    ) {
        if let Some(metadata) = self.chunks.get(&coordinate) {
            if metadata.temperature == ChunkTemperature::Warm
                || metadata.temperature == ChunkTemperature::Hot
            {
                // Ensure all cells in this chunk exist in the ResourceGrid
                let chunk_start = IVec2::new(
                    coordinate.x * CHUNK_SIZE as i32,
                    coordinate.y * CHUNK_SIZE as i32,
                );

                for dx in 0..CHUNK_SIZE as i32 {
                    for dy in 0..CHUNK_SIZE as i32 {
                        let cell_pos = chunk_start + IVec2::new(dx, dy);

                        // Only create cells if there should be biomass here
                        // This is where terrain data would be consulted
                        if grid.get_cell(cell_pos).is_none() {
                            // Create cell with default values based on some criteria
                            // For now, create a small percentage of cells
                            if (dx + dy) % 4 == 0 {
                                if let Err(e) = grid.get_or_create_cell(cell_pos, 50.0, 1.0) {
                                    warn!("Failed to create vegetation cell at {:?}: {}", cell_pos, e);
                                }
                            }
                        }
                    }
                }

                self.metrics.lazy_activations += 1;
            }
        }
    }

    /// Get metadata for a chunk
    pub fn get_chunk(&self, coordinate: &ChunkCoordinate) -> Option<&ChunkMetadata> {
        self.chunks.get(coordinate)
    }

    /// Check if a chunk is active (hot or warm)
    pub fn is_chunk_active(&self, coordinate: &ChunkCoordinate) -> bool {
        self.active_chunks.contains(coordinate)
    }

    /// Get all active chunks
    pub fn get_active_chunks(&self) -> &HashSet<ChunkCoordinate> {
        &self.active_chunks
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> &ChunkLODMetrics {
        &self.metrics
    }

    /// Reset performance metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = ChunkLODMetrics::default();
        self.metrics.total_chunks = self.chunks.len();
    }

    /// Clean up chunks that are too far from any agent
    pub fn cleanup_distant_chunks(&mut self, max_distance: i32) {
        let chunks_to_remove: Vec<ChunkCoordinate> = self
            .chunks
            .iter()
            .filter(|(_, metadata)| {
                // Find distance to nearest agent
                let chunk_center = IVec2::new(
                    metadata.coordinate.x * CHUNK_SIZE as i32 + CHUNK_SIZE as i32 / 2,
                    metadata.coordinate.y * CHUNK_SIZE as i32 + CHUNK_SIZE as i32 / 2,
                );

                let min_distance = self
                    .agent_positions
                    .iter()
                    .map(|&agent_pos| (agent_pos - chunk_center).abs().max_element())
                    .min()
                    .unwrap_or(i32::MAX);

                min_distance > max_distance
            })
            .map(|(coord, _)| *coord)
            .collect();

        for coord in chunks_to_remove {
            self.chunks.remove(&coord);
            self.active_chunks.remove(&coord);
        }
    }
}

impl Default for ChunkLODManager {
    fn default() -> Self {
        Self::new(ChunkLODConfig::default())
    }
}
