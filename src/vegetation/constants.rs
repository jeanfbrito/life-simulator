#![allow(dead_code)]
/// Vegetation system constants and parameters
///
/// This module consolidates all vegetation-related parameters for the plant system,
/// including growth rates, consumption limits, and foraging behavior constants.

/// Plant growth parameters
pub mod growth {
    /// Logistic growth rate coefficient (r)
    /// Controls how fast vegetation biomass regenerates
    /// Higher values = faster regrowth, but can lead to instability
    pub const GROWTH_RATE: f32 = 0.08; // 8% per tick at optimal conditions

    /// Initial biomass assigned to newly generated vegetation tiles
    /// Represents sparse ground cover that must regrow before becoming forageable
    pub const INITIAL_BIOMASS: f32 = 5.0;

    /// Maximum sustainable biomass per tile (Bmax)
    /// Represents the carrying capacity of vegetation on a single tile
    /// Measured in biomass units (arbitrary but consistent across the system)
    pub const MAX_BIOMASS: f32 = 100.0;

    /// Growth system update frequency
    /// Plant growth runs every N ticks (1 tick = 100ms at 10 TPS)
    /// Every 10 ticks = 1 second for growth updates
    pub const GROWTH_INTERVAL_TICKS: u64 = 10;

    /// Minimum biomass threshold for considering a tile "depleted"
    /// Tiles below this threshold provide negligible nutrition
    pub const DEPLETED_THRESHOLD: f32 = 5.0;

    /// Biomass recovery threshold for active tile tracking
    /// Tiles below this threshold are marked as "active" for faster updates
    pub const ACTIVE_TILE_THRESHOLD: f32 = 0.95; // 95% of Bmax
}

/// Herbivore consumption parameters
pub mod consumption {
    use super::growth::MAX_BIOMASS;

    /// Maximum biomass an herbivore can consume in a single meal
    /// This is species-specific but capped at 30% of available biomass
    pub const MAX_MEAL_FRACTION: f32 = 0.3; // 30% rule from plan

    /// Maximum absolute biomass consumable per meal
    /// Prevents unrealistic consumption from very high biomass tiles
    pub const MAX_MEAL_ABSOLUTE: f32 = MAX_BIOMASS * MAX_MEAL_FRACTION; // 30 units

    /// Minimum biomass required for a tile to be considered "forageable"
    /// Tiles below this level are skipped during foraging searches
    pub const FORAGE_MIN_BIOMASS: f32 = 10.0;

    /// Biomass level at which herbivores give up on a patch
    /// When biomass falls below this, animals seek new grazing areas
    pub const GIVING_UP_THRESHOLD: f32 = 20.0;

    /// Giving-up threshold as percentage of optimal biomass
    /// Animals will leave patches below this percentage of maximum biomass
    pub const GIVING_UP_THRESHOLD_RATIO: f32 = 0.25; // 25% of optimal biomass

    /// Cooldown ticks before re-evaluating the same depleted tile
    /// Prevents animals from repeatedly checking exhausted patches
    pub const DEPLETED_TILE_COOLDOWN: u64 = 50; // 5 seconds at 10 TPS
}

/// Species-specific consumption profiles
/// These values integrate with the existing SpeciesNeeds system
pub mod species {
    /// Rabbit-specific vegetation consumption
    pub mod rabbit {
        

        /// Meal size as fraction of rabbit's daily needs
        /// Rabbits eat small, frequent meals
        pub const MEAL_SIZE_FRACTION: f32 = 0.15; // 15% of daily intake per meal

        /// Daily biomass intake requirement
        /// Based on rabbit metabolic needs and vegetation nutritional value
        pub const DAILY_BIOMASS_NEED: f32 = 25.0; // biomass units per day

        /// Preferred biomass range for rabbit foraging
        /// Rabbits avoid both depleted and overly dense patches
        pub const PREFERRED_BIOMASS_MIN: f32 = 30.0;
        pub const PREFERRED_BIOMASS_MAX: f32 = 80.0;

        /// Rabbit foraging search parameters
        pub const SEARCH_RADIUS: i32 = 15; // tiles
        pub const SAMPLE_SIZE: usize = 8; // candidate tiles to evaluate
    }

    /// Deer-specific vegetation consumption (future)
    pub mod deer {
        /// Deer eat larger meals less frequently
        pub const MEAL_SIZE_FRACTION: f32 = 0.25; // 25% of daily intake per meal

        /// Daily biomass intake requirement (larger than rabbit)
        pub const DAILY_BIOMASS_NEED: f32 = 80.0; // biomass units per day

        /// Preferred biomass range for deer foraging
        pub const PREFERRED_BIOMASS_MIN: f32 = 40.0;
        pub const PREFERRED_BIOMASS_MAX: f32 = 90.0;

        /// Deer foraging search parameters (wider range than rabbits)
        pub const SEARCH_RADIUS: i32 = 25; // tiles
        pub const SAMPLE_SIZE: usize = 12; // candidate tiles to evaluate
    }
}

/// Predator fear and behavioral modifiers
pub mod predator_effects {
    /// Feeding duration reduction when predators are nearby
    /// Represents the trade-off between feeding and vigilance
    pub const FEAR_FEEDING_REDUCTION: f32 = 0.3; // 30% shorter feeding

    /// Radius at which predator presence affects herbivore behavior
    /// Distance in tiles at which herbivores become cautious
    pub const FEAR_RADIUS: i32 = 40; // tiles

    /// Biomass threshold increase under fear (less selective when scared)
    /// Desperate herbivores accept lower quality food when threatened
    pub const FEAR_BIOMASS_TOLERANCE: f32 = 0.2; // 20% lower threshold

    /// Movement speed increase when fleeing perceived danger
    /// Herbivores move faster when leaving areas due to predator presence
    pub const FEAR_SPEED_BOOST: f32 = 1.5; // 1.5x normal speed
}

/// Terrain-specific vegetation modifiers
pub mod terrain_modifiers {
    

    /// Maximum biomass multiplier by terrain type
    /// Some terrains support more vegetation than others
    pub fn max_biomass_multiplier(terrain: &str) -> f32 {
        match terrain {
            "Grass" => 1.0,              // Baseline
            "Forest" => 1.2,             // Understory vegetation
            "Dirt" => 0.7,               // Poor soil
            "Swamp" => 0.8,              // Water-logged but productive
            "Sand" | "Desert" => 0.2,    // Sparse vegetation
            "Stone" | "Mountain" => 0.1, // Lichens, mosses only
            "Snow" => 0.3,               // Limited alpine vegetation
            _ => 0.0,                    // No vegetation on water, deep water
        }
    }

    /// Growth rate modifier by terrain type
    /// Some terrains promote faster or slower vegetation growth
    pub fn growth_rate_modifier(terrain: &str) -> f32 {
        match terrain {
            "Grass" => 1.0,              // Baseline growth rate
            "Forest" => 1.1,             // Protected environment, slightly faster
            "Dirt" => 0.8,               // Poorer nutrients, slower growth
            "Swamp" => 1.2,              // High moisture, faster growth
            "Sand" | "Desert" => 0.4,    // Water-limited, slow growth
            "Stone" | "Mountain" => 0.3, // Harsh conditions, very slow
            "Snow" => 0.5,               // Cold-limited, slow growth
            _ => 0.0,                    // No growth on water
        }
    }
}

/// Performance and optimization parameters
pub mod performance {
    /// Maximum number of active tiles to process per growth cycle
    /// Limits CPU usage for vegetation updates on large maps
    pub const MAX_ACTIVE_TILES_PER_UPDATE: usize = 2000;

    /// Default number of vegetation chunks processed per update pass
    pub const DEFAULT_CHUNKS_PER_PASS: usize = 8;

    /// Minimum number of chunks processed when throttling for performance
    pub const MIN_CHUNKS_PER_PASS: usize = 1;

    /// Maximum number of chunks processed when ramping up throughput
    pub const MAX_CHUNKS_PER_PASS: usize = 128;

    /// Random sample size of inactive tiles to update per cycle
    /// Ensures inactive tiles still get occasional updates
    pub const INACTIVE_SAMPLE_SIZE: usize = 100;

    /// Grid chunk size for spatial organization
    /// Aligns with map chunk system for cache efficiency
    pub const CHUNK_SIZE: usize = 16; // Same as map chunks

    /// Memory optimization threshold
    /// Switch to sparse storage when vegetation density falls below this
    pub const SPARSE_STORAGE_THRESHOLD: f32 = 0.1; // 10% of tiles have vegetation

    /// Phase 4 performance budget targets
    /// CPU time budget per growth cycle (in microseconds)
    /// Vegetation updates should stay within this budget at 1 Hz
    pub const CPU_BUDGET_US: u64 = 1000; // 1ms per growth cycle

    /// Time budget for chunk processing (spread over main loop)
    pub const CHUNK_PROCESS_BUDGET_US: u64 = 2000; // 2ms per frame

    /// Base growth intervals per chunk tier (ticks)
    pub const CHUNK_INTERVAL_HOT_TICKS: u64 = 50; // active grazing areas (~5s at 10 TPS)
    pub const CHUNK_INTERVAL_WARM_TICKS: u64 = 150; // moderately active (~15s)
    pub const CHUNK_INTERVAL_COLD_TICKS: u64 = 300; // idle regions (~30s)

    /// Adaptive rate adjustments for chunk budget (scale up/down)
    pub const CHUNK_RATE_ADJUST_UP: f32 = 1.25;
    pub const CHUNK_RATE_ADJUST_DOWN: f32 = 0.7;

    /// Maximum tiles to process in a single batch
    /// Prevents large spikes in CPU usage by breaking updates into chunks
    pub const BATCH_SIZE: usize = 64; // Process up to 64 tiles per chunk pass

    /// Maximum time per batch before yielding (in microseconds)
    /// Ensures the system doesn't exceed time budget per batch
    pub const BATCH_TIME_BUDGET_US: u64 = 250; // 0.25ms per batch

    /// Performance profiling intervals
    /// How often to collect detailed performance metrics
    pub const PROFILING_INTERVAL_TICKS: u64 = 300; // Every 30 seconds at 10 TPS

    /// Target average biomass for performance scaling
    /// Used to adjust update frequency based on system load
    pub const TARGET_AVG_BIOMASS: f32 = 50.0; // 50% of maximum

    /// Adaptive scaling thresholds
    /// When average biomass is above this, reduce update frequency
    pub const HIGH_BIOMASS_THRESHOLD: f32 = 80.0; // 80% of maximum

    /// When average biomass is below this, increase update frequency
    pub const LOW_BIOMASS_THRESHOLD: f32 = 20.0; // 20% of maximum

    /// How often to refresh the vegetation heatmap snapshot for the web viewer
    pub const HEATMAP_UPDATE_INTERVAL_TICKS: u64 = 120; // every 12s at 10 TPS
}

/// Memory optimization parameters for Phase 4
pub mod memory {
    /// Memory usage thresholds for optimization triggers
    /// High memory usage threshold (in bytes)
    pub const HIGH_MEMORY_THRESHOLD: usize = 50 * 1024 * 1024; // 50MB

    /// Medium memory usage threshold (in bytes)
    pub const MEDIUM_MEMORY_THRESHOLD: usize = 20 * 1024 * 1024; // 20MB

    /// Low memory usage threshold (in bytes)
    pub const LOW_MEMORY_THRESHOLD: usize = 5 * 1024 * 1024; // 5MB

    /// Per-tile memory overhead threshold
    /// Above this, we should optimize storage
    pub const PER_TILE_OVERHEAD_THRESHOLD: usize = 45; // bytes

    /// Regional grid configuration for cache efficiency
    /// Size of each region in tiles (power of 2 for cache efficiency)
    pub const REGION_SIZE: usize = 64; // 8x8 tiles per region

    /// Maximum tiles per region before splitting
    pub const MAX_TILES_PER_REGION: usize = REGION_SIZE * REGION_SIZE; // 4096 tiles

    /// Memory optimization intervals
    /// How often to analyze memory usage (in ticks)
    pub const MEMORY_ANALYSIS_INTERVAL_TICKS: u64 = 600; // Every minute at 10 TPS

    /// Storage optimization recommendations
    /// When to recommend u16 storage vs f32
    pub const U16_STORAGE_RECOMMENDATION_THRESHOLD: f32 = 20.0; // 20% savings

    /// When to recommend regional grid organization
    pub const REGIONAL_GRID_RECOMMENDATION_TILES: usize = 10000; // 10k tiles

    /// Cache efficiency thresholds
    /// Minimum acceptable cache hit rate
    pub const MIN_CACHE_HIT_RATE: f32 = 0.7; // 70%

    /// Memory compression thresholds
    /// When to consider data compression
    pub const COMPRESSION_RECOMMENDATION_BYTES: usize = 100 * 1024 * 1024; // 100MB

    /// Sparse storage optimization
    /// When to use sparse storage vs dense storage
    pub const SPARSE_STORAGE_DENSITY_THRESHOLD: f32 = 0.05; // 5% tile coverage
}

/// Debug and monitoring parameters
pub mod debug {
    /// Enable detailed vegetation logging
    pub const VERBOSE_LOGGING: bool = false;

    /// Biomass reporting interval (in ticks)
    /// How often to report vegetation statistics
    pub const REPORTING_INTERVAL: u64 = 600; // Every 60 seconds at 10 TPS

    /// Sample size for biomass quality checks
    /// Number of tiles to sample for quality metrics
    pub const QUALITY_SAMPLE_SIZE: usize = 50;

    /// Enable biomass overlay in web viewer
    pub const ENABLE_OVERLAY: bool = true;

    /// Heatmap normalization range for viewer
    pub const HEATMAP_MIN: f32 = 0.0;
    pub const HEATMAP_MAX: f32 = 1.0;
}
