use bevy::math::IVec2;
use serde::{Deserialize, Serialize};

/// Port of OpenRCT2's `Algorithm` enum (MapGen.h)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    Blank,
    SimplexNoise,
    HeightmapImage,
}

impl Default for Algorithm {
    fn default() -> Self {
        Algorithm::Blank
    }
}

/// Settings struct mirroring `OpenRCT2::World::MapGenerator::Settings`.
#[derive(Debug, Clone)]
pub struct OpenRct2Settings {
    pub algorithm: Algorithm,
    pub map_size: IVec2,
    pub water_level: i32,
    pub land_texture: i32,
    pub edge_texture: i32,
    pub heightmap_low: i32,
    pub heightmap_high: i32,
    pub smooth_tile_edges: bool,
    pub trees: bool,
    pub tree_to_land_ratio: i32,
    pub min_tree_altitude: i32,
    pub max_tree_altitude: i32,
    pub beaches: bool,
    pub simplex_base_freq: i32,
    pub simplex_octaves: i32,
    pub smooth_height_map: bool,
    pub smooth_strength: u32,
    pub normalize_height: bool,
}

impl Default for OpenRct2Settings {
    fn default() -> Self {
        Self {
            algorithm: Algorithm::SimplexNoise,
            map_size: IVec2::new(150, 150),
            water_level: 6,
            land_texture: 0,
            edge_texture: 0,
            heightmap_low: 1,     // Produces ~0-5 base height (water range)
            heightmap_high: 40,   // Produces ~20 max height, smoothing raises to 30-60
            smooth_tile_edges: true,
            trees: true,
            tree_to_land_ratio: 25,
            min_tree_altitude: 10,
            max_tree_altitude: 50,
            beaches: true,
            simplex_base_freq: 175,
            simplex_octaves: 6,
            smooth_height_map: true,
            smooth_strength: 1,
            normalize_height: true,
        }
    }
}

/// Map Generator 2.0 configuration for enhanced terrain generation
/// Controls boundary enforcement, water placement, and terrain distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapGen2Config {
    // Perimeter boundary widths (in tiles)
    pub perimeter_deep_water_width: u32,     // Outermost layer = DeepWater (default: 1)
    pub perimeter_shallow_water_width: u32,  // Middle layer = ShallowWater (default: 2)
    pub perimeter_sand_min_width: u32,       // Inner layer = Sand/Beach (default: 1)

    // Internal water body transitions
    pub internal_water_transition_width: u32, // ShallowWater buffer between DeepWater and land (default: 1)

    // Terrain distribution targets
    pub land_coverage_target: f32,           // Minimum land percentage (0.0-1.0) (default: 0.60 = 60%)
    pub grass_forest_ratio: f32,             // Grass+Forest as percentage of land (0.0-1.0) (default: 0.50 = 50%)

    // Water spot placement
    pub water_spot_count: u32,               // Target number of internal water bodies (default: 8)
    pub water_spot_radius_min: f32,          // Minimum water spot radius in tiles (default: 3.0)
    pub water_spot_radius_max: f32,          // Maximum water spot radius in tiles (default: 8.0)
}

impl Default for MapGen2Config {
    fn default() -> Self {
        Self {
            perimeter_deep_water_width: 1,
            perimeter_shallow_water_width: 2,
            perimeter_sand_min_width: 1,
            internal_water_transition_width: 1,
            land_coverage_target: 0.60,
            grass_forest_ratio: 0.50,
            water_spot_count: 8,
            water_spot_radius_min: 3.0,
            water_spot_radius_max: 8.0,
        }
    }
}

/// Spot noise configuration for Factorio-style water body placement
/// Controls noise-based distribution of internal water bodies for natural clustering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotNoiseConfig {
    pub frequency: f64,          // Base frequency of spot noise (default: 0.02)
    pub spot_threshold: f64,     // Noise value threshold for spot placement (default: 0.6)
    pub spot_radius_scale: f32,  // Multiplier for water spot size (default: 1.0)
    pub jitter_amount: f32,      // Random position offset 0.0-1.0 (default: 0.3)
}

impl Default for SpotNoiseConfig {
    fn default() -> Self {
        Self {
            frequency: 0.02,
            spot_threshold: 0.6,
            spot_radius_scale: 1.0,
            jitter_amount: 0.3,
        }
    }
}
