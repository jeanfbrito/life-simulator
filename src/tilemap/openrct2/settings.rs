use bevy::math::IVec2;

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
