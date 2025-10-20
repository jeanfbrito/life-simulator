use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::tilemap::{ChunkCoordinate, CHUNK_SIZE};

/// Cached world data that stores loaded chunks with multi-layer support
#[derive(Debug, Clone, Resource)]
pub struct CachedWorld {
    pub name: String,
    pub seed: u64,
    pub chunks: HashMap<(i32, i32), HashMap<String, Vec<Vec<String>>>>,
    pub is_loaded: bool,
}

// Global static for web server access
static mut CACHED_WORLD: Option<CachedWorld> = None;
static CACHED_WORLD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

impl CachedWorld {
    /// Get global cached world (for web server access)
    pub fn global_get() -> Option<CachedWorld> {
        if let Ok(_lock) = CACHED_WORLD_LOCK.lock() {
            unsafe { CACHED_WORLD.clone() }
        } else {
            error!("Failed to acquire cached world lock for reading");
            None
        }
    }

    /// Set global cached world (for web server access)
    pub fn global_set(world: CachedWorld) {
        if let Ok(_lock) = CACHED_WORLD_LOCK.lock() {
            unsafe {
                CACHED_WORLD = Some(world);
            }
        } else {
            error!("Failed to acquire cached world lock for writing");
        }
    }

    /// Check if global cached world is loaded
    pub fn global_is_loaded() -> bool {
        if let Ok(_lock) = CACHED_WORLD_LOCK.lock() {
            unsafe { CACHED_WORLD.as_ref().map(|w| w.is_loaded).unwrap_or(false) }
        } else {
            error!("Failed to acquire cached world lock for checking loaded status");
            false
        }
    }

    /// Get terrain layer from global cached world (for backward compatibility)
    pub fn global_get_chunk(chunk_x: i32, chunk_y: i32) -> Option<Vec<Vec<String>>> {
        if let Ok(_lock) = CACHED_WORLD_LOCK.lock() {
            unsafe {
                CACHED_WORLD
                    .as_ref()
                    .and_then(|w| w.chunks.get(&(chunk_x, chunk_y)))
                    .and_then(|layers| layers.get("terrain").cloned())
            }
        } else {
            error!("Failed to acquire cached world lock for terrain access");
            None
        }
    }

    /// Get all layers from global cached world
    pub fn global_get_chunk_layers(
        chunk_x: i32,
        chunk_y: i32,
    ) -> Option<HashMap<String, Vec<Vec<String>>>> {
        if let Ok(_lock) = CACHED_WORLD_LOCK.lock() {
            unsafe {
                CACHED_WORLD
                    .as_ref()
                    .and_then(|w| w.chunks.get(&(chunk_x, chunk_y)).cloned())
            }
        } else {
            error!("Failed to acquire cached world lock for chunk layers access");
            None
        }
    }

    /// Get specific layer from global cached world
    pub fn global_get_chunk_layer(
        chunk_x: i32,
        chunk_y: i32,
        layer_name: &str,
    ) -> Option<Vec<Vec<String>>> {
        let _lock = CACHED_WORLD_LOCK.lock().unwrap();
        unsafe {
            CACHED_WORLD
                .as_ref()
                .and_then(|w| w.chunks.get(&(chunk_x, chunk_y)))
                .and_then(|layers| layers.get(layer_name).cloned())
        }
    }
}

impl Default for CachedWorld {
    fn default() -> Self {
        Self {
            name: "Generated World".to_string(),
            seed: 12345,
            chunks: HashMap::new(),
            is_loaded: false,
        }
    }
}

impl CachedWorld {
    /// Create a new empty cached world
    pub fn new(name: String, seed: u64) -> Self {
        Self {
            name,
            seed,
            chunks: HashMap::new(),
            is_loaded: false,
        }
    }

    /// Load world from serialized data
    pub fn from_serialized(serialized_world: crate::serialization::SerializedWorld) -> Self {
        let chunks = crate::serialization::WorldSerializer::chunks_to_multi_layer_hashmap(
            serialized_world.chunks,
        );
        Self {
            name: serialized_world.name,
            seed: serialized_world.seed,
            chunks,
            is_loaded: true,
        }
    }

    /// Get terrain data for a specific chunk (for backward compatibility)
    pub fn get_chunk(&self, chunk_x: i32, chunk_y: i32) -> Option<Vec<Vec<String>>> {
        self.chunks
            .get(&(chunk_x, chunk_y))
            .and_then(|layers| layers.get("terrain").cloned())
    }

    /// Get all layers for a specific chunk
    pub fn get_chunk_layers(
        &self,
        chunk_x: i32,
        chunk_y: i32,
    ) -> Option<HashMap<String, Vec<Vec<String>>>> {
        self.chunks.get(&(chunk_x, chunk_y)).cloned()
    }

    /// Get specific layer for a specific chunk
    pub fn get_chunk_layer(
        &self,
        chunk_x: i32,
        chunk_y: i32,
        layer_name: &str,
    ) -> Option<Vec<Vec<String>>> {
        self.chunks
            .get(&(chunk_x, chunk_y))
            .and_then(|layers| layers.get(layer_name).cloned())
    }

    /// Set all layers for a specific chunk
    pub fn set_chunk_layers(
        &mut self,
        chunk_x: i32,
        chunk_y: i32,
        layers: HashMap<String, Vec<Vec<String>>>,
    ) {
        self.chunks.insert((chunk_x, chunk_y), layers);
    }

    /// Set specific layer for a specific chunk
    pub fn set_chunk_layer(
        &mut self,
        chunk_x: i32,
        chunk_y: i32,
        layer_name: String,
        data: Vec<Vec<String>>,
    ) {
        let chunk_layers = self
            .chunks
            .entry((chunk_x, chunk_y))
            .or_insert_with(HashMap::new);
        chunk_layers.insert(layer_name, data);
    }

    /// Clear all cached chunks
    pub fn clear(&mut self) {
        self.chunks.clear();
        self.is_loaded = false;
    }

    /// Load chunks from serialized world
    pub fn load_from_serialized(
        &mut self,
        serialized_world: crate::serialization::SerializedWorld,
    ) {
        self.name = serialized_world.name;
        self.seed = serialized_world.seed;
        self.chunks = crate::serialization::WorldSerializer::chunks_to_multi_layer_hashmap(
            serialized_world.chunks,
        );
        self.is_loaded = true;
    }

    /// Get world information as JSON
    pub fn get_world_info_json(&self) -> String {
        format!(
            r#"{{"center_chunk": {{"x": 0, "y": 0}}, "world_size": {{"width": 20, "height": 20}}, "seed": {}, "name": "{}", "is_loaded": {}}}"#,
            self.seed, self.name, self.is_loaded
        )
    }

    /// Generate chunks JSON for API response (terrain only for backward compatibility)
    pub fn generate_chunks_json(&self, path: &str) -> String {
        // Parse coordinates from path like /api/chunks?coords=0,0&coords=1,0
        let coords = self.parse_chunk_coords(path);
        let mut chunk_data = HashMap::new();

        for &(chunk_x, chunk_y) in &coords {
            let chunk_key = format!("{},{}", chunk_x, chunk_y);

            if let Some(terrain_data) = self.get_chunk(chunk_x, chunk_y) {
                chunk_data.insert(chunk_key, terrain_data);
            }
        }

        // Convert to JSON string
        let mut json_parts = Vec::new();
        for (key, data) in chunk_data {
            let data_str = data
                .iter()
                .map(|row| {
                    format!(
                        "[{}]",
                        row.iter()
                            .map(|tile| format!("\"{}\"", tile))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            json_parts.push(format!("\"{}\": [{}]", key, data_str));
        }

        format!("{{\"chunk_data\": {{{}}}}}", json_parts.join(", "))
    }

    /// Generate multi-layer chunks JSON for API response
    pub fn generate_multi_layer_chunks_json(&self, path: &str) -> String {
        // Parse coordinates from path like /api/chunks?coords=0,0&coords=1,0
        let coords = self.parse_chunk_coords(path);
        let mut chunk_data = HashMap::new();

        for &(chunk_x, chunk_y) in &coords {
            let chunk_key = format!("{},{}", chunk_x, chunk_y);

            if let Some(layers) = self.get_chunk_layers(chunk_x, chunk_y) {
                // Convert each layer to JSON
                let mut layer_json_parts = Vec::new();
                for (layer_name, layer_data) in layers {
                    let data_str = layer_data
                        .iter()
                        .map(|row| {
                            format!(
                                "[{}]",
                                row.iter()
                                    .map(|tile| format!("\"{}\"", tile))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    layer_json_parts.push(format!("\"{}\": [{}]", layer_name, data_str));
                }
                chunk_data.insert(chunk_key, format!("{{{}}}", layer_json_parts.join(", ")));
            }
        }

        // Convert to JSON string
        let mut json_parts = Vec::new();
        for (key, layers_json) in chunk_data {
            json_parts.push(format!("\"{}\": {}", key, layers_json));
        }

        format!("{{\"chunk_data\": {{{}}}}}", json_parts.join(", "))
    }

    fn parse_chunk_coords(&self, path: &str) -> Vec<(i32, i32)> {
        // Extract coordinates from path like /api/chunks?coords=0,0&coords=1,0
        // or /api/chunks?center_x=0&center_y=0&radius=1
        if let Some(query_part) = path.split('?').nth(1) {
            let mut coords = Vec::new();

            // Check for center_x/center_y/radius format first
            let mut center_x = 0i32;
            let mut center_y = 0i32;
            let mut radius = 1i32;
            let mut has_center_format = false;

            for param in query_part.split('&') {
                if let Some(x_str) = param.strip_prefix("center_x=") {
                    if let Ok(x) = x_str.parse::<i32>() {
                        center_x = x;
                        has_center_format = true;
                    }
                } else if let Some(y_str) = param.strip_prefix("center_y=") {
                    if let Ok(y) = y_str.parse::<i32>() {
                        center_y = y;
                        has_center_format = true;
                    }
                } else if let Some(r_str) = param.strip_prefix("radius=") {
                    if let Ok(r) = r_str.parse::<i32>() {
                        radius = r;
                        has_center_format = true;
                    }
                } else if let Some(coord_part) = param.strip_prefix("coords=") {
                    // Handle explicit coords format
                    if let Some((x_str, y_str)) = coord_part.split_once(',') {
                        if let (Ok(x), Ok(y)) = (x_str.parse::<i32>(), y_str.parse::<i32>()) {
                            coords.push((x, y));
                        }
                    }
                }
            }

            // If center/radius format was used, generate coords in a square
            if has_center_format {
                for x in (center_x - radius)..=(center_x + radius) {
                    for y in (center_y - radius)..=(center_y + radius) {
                        coords.push((x, y));
                    }
                }
                return coords;
            } else if !coords.is_empty() {
                // Return explicit coords if they were provided
                return coords;
            }
        }
        // Default to center chunk (0, 0)
        vec![(0, 0)]
    }
}

/// Component to mark entities that should update the cached world
#[derive(Component)]
pub struct UpdateCachedWorld {
    pub serialized_world: crate::serialization::SerializedWorld,
}

/// System to handle cached world updates
pub fn handle_cached_world_updates(
    mut commands: Commands,
    update_requests: Query<(Entity, &UpdateCachedWorld)>,
    mut cached_world: ResMut<CachedWorld>,
) {
    for (entity, request) in update_requests.iter() {
        info!(
            "Updating cached world to: {}",
            request.serialized_world.name
        );

        cached_world.load_from_serialized(request.serialized_world.clone());

        // Also update global instance for web server access
        CachedWorld::global_set(cached_world.clone());

        commands.entity(entity).remove::<UpdateCachedWorld>();
    }
}

/// Plugin to add cached world systems
pub struct CachedWorldPlugin;

impl Plugin for CachedWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CachedWorld>()
            .add_systems(Update, handle_cached_world_updates);
    }
}
