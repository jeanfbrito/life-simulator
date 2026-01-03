mod height_map;
mod settings;
mod simplex_noise;

pub use height_map::HeightMap;
pub use settings::{Algorithm, MapGen2Config, OpenRct2Settings, SpotNoiseConfig};
pub use simplex_noise::{generate_simplex_noise, smooth_height_map, SimplexNoise};
