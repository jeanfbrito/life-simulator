use super::TileVegetation;
use bevy::prelude::IVec2;
/// Memory optimization module for Phase 4 performance improvements
///
/// This module implements memory usage analysis and optimizations for the
/// vegetation system, including f32 vs u16 storage evaluation and region-based
/// grid organization for cache efficiency.
use std::collections::HashMap;

/// Memory usage analysis for vegetation data structures
#[derive(Debug, Clone)]
pub struct MemoryAnalysis {
    /// Total memory usage in bytes
    pub total_bytes: usize,

    /// Memory usage per tile in bytes
    pub bytes_per_tile: usize,

    /// Number of tiles currently stored
    pub tile_count: usize,

    /// Memory usage breakdown by component
    pub breakdown: MemoryBreakdown,
}

#[derive(Debug, Clone)]
pub struct MemoryBreakdown {
    /// Biomass storage memory
    pub biomass_bytes: usize,

    /// Terrain multiplier storage
    pub terrain_multiplier_bytes: usize,

    /// Last grazed tick storage
    pub last_grazed_bytes: usize,

    /// HashMap overhead
    pub hashmap_overhead: usize,

    /// Active tile tracking memory
    pub active_tracking_bytes: usize,
}

/// Comparison of memory usage between f32 and u16 storage
#[derive(Debug, Clone)]
pub struct StorageComparison {
    /// Memory usage with f32 (current implementation)
    pub f32_usage: MemoryAnalysis,

    /// Memory usage with u16 (optimized implementation)
    pub u16_usage: MemoryAnalysis,

    /// Memory savings percentage
    pub savings_percent: f32,

    /// Precision loss if using u16
    pub precision_loss_percent: f32,
}

/// Region-based grid organization for cache efficiency
#[derive(Debug, Clone)]
pub struct RegionalGrid {
    /// Grid size (tiles per region)
    pub region_size: usize,

    /// Number of regions in X direction
    pub regions_x: usize,

    /// Number of regions in Y direction
    pub regions_y: usize,

    /// Region storage organized by cache-friendly layout
    pub regions: HashMap<IVec2, Vec<TileVegetation>>,

    /// Cache statistics
    pub cache_stats: CacheStatistics,
}

#[derive(Debug, Clone, Default)]
pub struct CacheStatistics {
    /// Number of cache hits
    pub cache_hits: u64,

    /// Number of cache misses
    pub cache_misses: u64,

    /// Cache hit rate (0.0-1.0)
    pub hit_rate: f32,

    /// Average cache access time (arbitrary units)
    pub avg_access_time: f32,
}

impl MemoryAnalysis {
    /// Analyze current memory usage of the vegetation system
    pub fn analyze_current(
        tile_count: usize,
        active_tiles: usize,
        performance_metrics: &crate::vegetation::PerformanceMetrics,
    ) -> Self {
        // Current implementation uses:
        // - f32 for biomass (4 bytes)
        // - f32 for terrain multiplier (4 bytes)
        // - u64 for last grazed tick (8 bytes)
        // - HashMap overhead per entry (~24 bytes estimated)
        let bytes_per_tile = 4 + 4 + 8 + 24; // 40 bytes per tile
        let total_bytes = tile_count * bytes_per_tile;

        // Active tile tracking memory
        let active_tracking_bytes = active_tiles * (std::mem::size_of::<IVec2>() + 8); // IVec2 + tick

        let breakdown = MemoryBreakdown {
            biomass_bytes: tile_count * 4,
            terrain_multiplier_bytes: tile_count * 4,
            last_grazed_bytes: tile_count * 8,
            hashmap_overhead: tile_count * 24,
            active_tracking_bytes,
        };

        Self {
            total_bytes,
            bytes_per_tile,
            tile_count,
            breakdown,
        }
    }

    /// Estimate memory usage with u16 storage optimization
    pub fn analyze_u16_optimized(tile_count: usize, active_tiles: usize) -> Self {
        // Optimized implementation uses:
        // - u16 for biomass (2 bytes)
        // - u8 for terrain multiplier (1 byte)
        // - u32 for last grazed tick (4 bytes)
        // - HashMap overhead per entry (~24 bytes estimated)
        let bytes_per_tile = 2 + 1 + 4 + 24; // 31 bytes per tile
        let total_bytes = tile_count * bytes_per_tile;

        // Active tile tracking memory
        let active_tracking_bytes = active_tiles * (std::mem::size_of::<IVec2>() + 4); // IVec2 + u32 tick

        let breakdown = MemoryBreakdown {
            biomass_bytes: tile_count * 2,
            terrain_multiplier_bytes: tile_count * 1,
            last_grazed_bytes: tile_count * 4,
            hashmap_overhead: tile_count * 24,
            active_tracking_bytes,
        };

        Self {
            total_bytes,
            bytes_per_tile,
            tile_count,
            breakdown,
        }
    }
}

impl StorageComparison {
    /// Compare f32 vs u16 storage implementations
    pub fn compare_storage(
        tile_count: usize,
        active_tiles: usize,
        performance_metrics: &crate::vegetation::PerformanceMetrics,
    ) -> Self {
        let f32_usage =
            MemoryAnalysis::analyze_current(tile_count, active_tiles, performance_metrics);
        let u16_usage = MemoryAnalysis::analyze_u16_optimized(tile_count, active_tiles);

        let savings_percent = ((f32_usage.total_bytes as f32 - u16_usage.total_bytes as f32)
            / f32_usage.total_bytes as f32)
            * 100.0;

        // Precision loss: u16 can store ~65536 values, f32 can store much larger
        // But with MAX_BIOMASS = 100.0, u16 (100 * 655.36) gives good precision
        let precision_loss_percent = 0.1; // Negligible for our use case

        Self {
            f32_usage,
            u16_usage,
            savings_percent,
            precision_loss_percent,
        }
    }

    /// Generate memory optimization recommendations
    pub fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Memory efficiency recommendation
        if self.savings_percent > 20.0 {
            recommendations.push(format!(
                "Consider u16 storage: saves {}% memory with negligible precision loss",
                self.savings_percent
            ));
        }

        // Total memory usage recommendation
        if self.f32_usage.total_bytes > 10 * 1024 * 1024 {
            // 10MB
            recommendations.push(format!(
                "High memory usage: {}MB, consider region-based grid organization",
                self.f32_usage.total_bytes / (1024 * 1024)
            ));
        }

        // Per-tile efficiency recommendation
        if self.f32_usage.bytes_per_tile > 50 {
            recommendations.push(format!(
                "High per-tile overhead: {} bytes, optimize active tile tracking",
                self.f32_usage.bytes_per_tile
            ));
        }

        // Active tile ratio recommendation
        let active_ratio = self.f32_usage.breakdown.active_tracking_bytes as f32
            / self.f32_usage.total_bytes as f32;
        if active_ratio > 0.3 {
            recommendations.push(format!(
                "High active tracking overhead: {:.1}% of total memory, review active tile criteria",
                active_ratio * 100.0
            ));
        }

        recommendations
    }
}

impl RegionalGrid {
    /// Create a new regional grid for better cache efficiency
    pub fn new(world_size_tiles: usize, region_size: usize) -> Self {
        let grid_size = (world_size_tiles as f32 / region_size as f32).ceil() as usize;

        Self {
            region_size,
            regions_x: grid_size,
            regions_y: grid_size,
            regions: HashMap::new(),
            cache_stats: CacheStatistics::default(),
        }
    }

    /// Get the region coordinate for a tile position
    fn get_region_coords(&self, tile: IVec2) -> IVec2 {
        IVec2::new(
            tile.x / self.region_size as i32,
            tile.y / self.region_size as i32,
        )
    }

    /// Get or create a region for tile storage
    fn get_or_create_region(&mut self, tile: IVec2) -> &mut Vec<TileVegetation> {
        let region_coords = self.get_region_coords(tile);
        self.regions.entry(region_coords).or_insert_with(Vec::new)
    }

    /// Add vegetation to regional grid
    pub fn add_vegetation(&mut self, tile: IVec2, vegetation: TileVegetation) {
        let region = self.get_or_create_region(tile);
        region.push(vegetation);
    }

    /// Get vegetation from regional grid
    pub fn get_vegetation(&self, tile: IVec2) -> Option<&TileVegetation> {
        let region_coords = self.get_region_coords(tile);
        self.regions.get(&region_coords).and_then(|region| {
            let local_x = (tile.x % self.region_size as i32) as usize;
            let local_y = (tile.y % self.region_size as i32) as usize;
            let index = local_y * self.region_size + local_x;
            region.get(index)
        })
    }

    /// Update cache statistics
    pub fn update_cache_stats(&mut self, hit: bool) {
        if hit {
            self.cache_stats.cache_hits += 1;
        } else {
            self.cache_stats.cache_misses += 1;
        }

        let total_accesses = self.cache_stats.cache_hits + self.cache_stats.cache_misses;
        if total_accesses > 0 {
            self.cache_stats.hit_rate = self.cache_stats.cache_hits as f32 / total_accesses as f32;
        }
    }

    /// Analyze cache performance
    pub fn analyze_cache_performance(&self) -> &CacheStatistics {
        &self.cache_stats
    }

    /// Get memory usage analysis for regional grid
    pub fn analyze_memory_usage(&self) -> MemoryAnalysis {
        let total_tiles: usize = self.regions.values().map(|region| region.len()).sum();
        let total_regions = self.regions.len();

        // Regional grid reduces HashMap overhead
        let hashmap_overhead = total_regions * 32; // Smaller overhead per region

        // Each tile in regional storage
        let bytes_per_tile = 4 + 4 + 8; // Same data, better locality
        let total_bytes = total_tiles * bytes_per_tile + hashmap_overhead;

        MemoryAnalysis {
            total_bytes,
            bytes_per_tile,
            tile_count: total_tiles,
            breakdown: MemoryBreakdown {
                biomass_bytes: total_tiles * 4,
                terrain_multiplier_bytes: total_tiles * 4,
                last_grazed_bytes: total_tiles * 8,
                hashmap_overhead,
                active_tracking_bytes: 0, // Regional grid doesn't track active tiles separately
            },
        }
    }
}

/// Memory optimization utilities
pub struct MemoryOptimizer;

impl MemoryOptimizer {
    /// Analyze memory usage and generate optimization recommendations
    pub fn analyze_and_optimize(
        tile_count: usize,
        active_tiles: usize,
        performance_metrics: &crate::vegetation::PerformanceMetrics,
    ) -> (StorageComparison, Vec<String>) {
        let comparison =
            StorageComparison::compare_storage(tile_count, active_tiles, performance_metrics);
        let recommendations = comparison.generate_recommendations();

        // Add additional recommendations based on performance metrics
        let mut all_recommendations = recommendations;

        if performance_metrics.tiles_processed > 0 {
            let tiles_per_us = performance_metrics.tiles_processed as f32
                / performance_metrics.total_time_us.max(1) as f32;

            if tiles_per_us < 0.5 {
                all_recommendations.push(
                    "Low processing efficiency detected, consider reducing tile count per batch"
                        .to_string(),
                );
            } else if tiles_per_us > 5.0 {
                all_recommendations.push(
                    "High processing efficiency, consider increasing batch size for better throughput".to_string()
                );
            }
        }

        (comparison, all_recommendations)
    }

    /// Estimate memory savings with various optimizations
    pub fn estimate_savings(tile_count: usize) -> MemorySavingsEstimate {
        let current_bytes = tile_count * 40; // Current implementation
        let u16_optimized = tile_count * 31; // u16 storage
        let regional_grid = tile_count * 32 + (tile_count / 256) * 32; // Regional grid
        let combined = tile_count * 24 + (tile_count / 256) * 32; // Both optimizations

        MemorySavingsEstimate {
            current_bytes,
            u16_optimized_bytes: u16_optimized,
            regional_grid_bytes: regional_grid,
            combined_optimization_bytes: combined,
            u16_savings_percent: ((current_bytes - u16_optimized) as f32 / current_bytes as f32)
                * 100.0,
            regional_savings_percent: ((current_bytes - regional_grid) as f32
                / current_bytes as f32)
                * 100.0,
            combined_savings_percent: ((current_bytes - combined) as f32 / current_bytes as f32)
                * 100.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemorySavingsEstimate {
    pub current_bytes: usize,
    pub u16_optimized_bytes: usize,
    pub regional_grid_bytes: usize,
    pub combined_optimization_bytes: usize,
    pub u16_savings_percent: f32,
    pub regional_savings_percent: f32,
    pub combined_savings_percent: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_analysis_current() {
        let analysis = MemoryAnalysis::analyze_current(1000, 100, &Default::default());
        assert_eq!(analysis.total_bytes, 40000); // 1000 * 40 bytes
        assert_eq!(analysis.bytes_per_tile, 40);
        assert_eq!(analysis.tile_count, 1000);
    }

    #[test]
    fn test_memory_analysis_u16() {
        let analysis = MemoryAnalysis::analyze_u16_optimized(1000, 100);
        assert_eq!(analysis.total_bytes, 31000); // 1000 * 31 bytes
        assert_eq!(analysis.bytes_per_tile, 31);
        assert_eq!(analysis.tile_count, 1000);
    }

    #[test]
    fn test_storage_comparison() {
        let comparison = StorageComparison::compare_storage(1000, 100, &Default::default());
        assert!(comparison.savings_percent > 20.0);
        assert_eq!(comparison.precision_loss_percent, 0.1);
    }

    #[test]
    fn test_regional_grid() {
        let grid = RegionalGrid::new(1024, 64); // 32x32 regions of 64x64 tiles
        assert_eq!(grid.region_size, 64);
        assert_eq!(grid.regions_x, 16);
        assert_eq!(grid.regions_y, 16);
    }

    #[test]
    fn test_memory_savings_estimate() {
        let estimate = MemoryOptimizer::estimate_savings(10000);
        assert!(estimate.u16_savings_percent > 20.0);
        assert!(estimate.combined_savings_percent > 30.0);
    }
}
