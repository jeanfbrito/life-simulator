use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerrainType {
    Grass,
    Stone,
    Sand,
    Water,
    Dirt,
    Snow,
    Forest,
    Mountain,
    DeepWater,
    ShallowWater,
    Swamp,
    Desert,
}

impl TerrainType {
    /// Parse terrain type from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Grass" => Some(TerrainType::Grass),
            "Stone" => Some(TerrainType::Stone),
            "Sand" => Some(TerrainType::Sand),
            "Water" => Some(TerrainType::Water),
            "Dirt" => Some(TerrainType::Dirt),
            "Snow" => Some(TerrainType::Snow),
            "Forest" => Some(TerrainType::Forest),
            "Mountain" => Some(TerrainType::Mountain),
            "DeepWater" => Some(TerrainType::DeepWater),
            "ShallowWater" => Some(TerrainType::ShallowWater),
            "Swamp" => Some(TerrainType::Swamp),
            "Desert" => Some(TerrainType::Desert),
            _ => None,
        }
    }

    pub fn is_walkable(&self) -> bool {
        !matches!(
            self,
            TerrainType::Water | TerrainType::DeepWater | TerrainType::Mountain
        )
    }

    pub fn movement_cost(&self) -> f32 {
        match self {
            TerrainType::Grass | TerrainType::Dirt | TerrainType::Sand => 1.0,
            TerrainType::Stone => 1.2,
            TerrainType::Snow => 1.5,
            TerrainType::Forest => 1.8,
            TerrainType::ShallowWater | TerrainType::Swamp => 2.0,
            TerrainType::Desert => 1.3,
            TerrainType::Water | TerrainType::DeepWater | TerrainType::Mountain => f32::INFINITY,
        }
    }

    pub fn fertility(&self) -> f32 {
        match self {
            TerrainType::Grass => 0.8,
            TerrainType::Dirt => 0.7,
            TerrainType::Forest => 0.9,
            TerrainType::Swamp => 0.6,
            TerrainType::Sand | TerrainType::Desert => 0.1,
            TerrainType::Stone | TerrainType::Mountain => 0.0,
            TerrainType::Snow => 0.2,
            TerrainType::Water | TerrainType::DeepWater | TerrainType::ShallowWater => 0.0,
        }
    }

    pub fn resource_potential(&self) -> Vec<&'static str> {
        match self {
            TerrainType::Forest => vec!["wood", "berries", "herbs"],
            TerrainType::Mountain => vec!["stone", "ore", "minerals"],
            TerrainType::Stone => vec!["stone", "minerals"],
            TerrainType::Grass => vec!["grain", "herbs"],
            TerrainType::Desert => vec!["sand", "rare_minerals"],
            TerrainType::Swamp => vec!["herbs", "clay"],
            TerrainType::Water | TerrainType::DeepWater | TerrainType::ShallowWater => vec!["fish"],
            _ => vec![],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TerrainProperties {
    pub terrain_type: TerrainType,
    pub elevation: f32,
    pub moisture: f32,
    pub temperature: f32,
}

impl TerrainProperties {
    pub fn new(terrain_type: TerrainType) -> Self {
        let (elevation, moisture, temperature) = match terrain_type {
            TerrainType::Mountain => (0.9, 0.3, 0.2),
            TerrainType::Forest => (0.4, 0.7, 0.5),
            TerrainType::Grass => (0.3, 0.5, 0.6),
            TerrainType::Desert => (0.3, 0.1, 0.9),
            TerrainType::Snow => (0.5, 0.6, 0.1),
            TerrainType::Swamp => (0.1, 0.9, 0.7),
            TerrainType::Water => (0.0, 1.0, 0.5),
            TerrainType::DeepWater => (-0.2, 1.0, 0.4),
            TerrainType::ShallowWater => (0.05, 1.0, 0.5),
            TerrainType::Stone => (0.6, 0.2, 0.5),
            TerrainType::Sand => (0.2, 0.2, 0.7),
            TerrainType::Dirt => (0.3, 0.4, 0.6),
        };

        Self {
            terrain_type,
            elevation,
            moisture,
            temperature,
        }
    }

    pub fn determine_terrain(elevation: f32, moisture: f32, temperature: f32) -> TerrainType {
        if elevation < 0.0 {
            TerrainType::DeepWater
        } else if elevation < 0.1 {
            TerrainType::ShallowWater
        } else if elevation < 0.2 {
            if moisture > 0.8 {
                TerrainType::Swamp
            } else {
                TerrainType::Sand
            }
        } else if elevation > 0.8 {
            TerrainType::Mountain
        } else if elevation > 0.6 {
            if temperature < 0.3 {
                TerrainType::Snow
            } else {
                TerrainType::Stone
            }
        } else {
            // Mid elevations
            if temperature > 0.7 && moisture < 0.3 {
                TerrainType::Desert
            } else if moisture > 0.6 {
                TerrainType::Forest
            } else if moisture > 0.3 {
                TerrainType::Grass
            } else {
                TerrainType::Dirt
            }
        }
    }
}