use super::{ChunkCoordinate, WorldGenerator};
use bevy::log::info;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const VIEW_DISTANCE: i32 = 3;
pub const UNLOAD_DISTANCE: i32 = 5;
pub const MAX_LOADED_CHUNKS: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkLoadRequest {
    pub coordinate: ChunkCoordinate,
    pub priority: LoadPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadPriority {
    High,
    Medium,
    Low,
}

#[derive(Resource)]
pub struct ChunkManager {
    loaded_chunks: HashMap<ChunkCoordinate, Entity>,
    chunk_entities: HashMap<Entity, ChunkCoordinate>,
    view_distance: i32,
    unload_distance: i32,
    max_loaded_chunks: usize,
    load_requests: Vec<ChunkLoadRequest>,
    statistics: ChunkManagerStatistics,
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self {
            loaded_chunks: HashMap::new(),
            chunk_entities: HashMap::new(),
            view_distance: VIEW_DISTANCE,
            unload_distance: UNLOAD_DISTANCE,
            max_loaded_chunks: MAX_LOADED_CHUNKS,
            load_requests: Vec::new(),
            statistics: ChunkManagerStatistics::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkManagerStatistics {
    pub total_chunks_loaded: u64,
    pub total_chunks_unloaded: u64,
    pub peak_loaded_chunks: usize,
    pub current_loaded_chunks: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl Default for ChunkManagerStatistics {
    fn default() -> Self {
        Self {
            total_chunks_loaded: 0,
            total_chunks_unloaded: 0,
            peak_loaded_chunks: 0,
            current_loaded_chunks: 0,
            cache_hits: 0,
            cache_misses: 0,
        }
    }
}

impl ChunkManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(view_distance: i32, unload_distance: i32, max_loaded_chunks: usize) -> Self {
        Self {
            view_distance,
            unload_distance,
            max_loaded_chunks,
            ..Default::default()
        }
    }

    pub fn is_chunk_loaded(&self, coord: &ChunkCoordinate) -> bool {
        self.loaded_chunks.contains_key(coord)
    }

    pub fn get_chunk_entity(&self, coord: &ChunkCoordinate) -> Option<Entity> {
        self.loaded_chunks.get(coord).copied()
    }

    pub fn get_chunk_coordinate(&self, entity: Entity) -> Option<ChunkCoordinate> {
        self.chunk_entities.get(&entity).copied()
    }

    pub fn register_chunk(&mut self, coord: ChunkCoordinate, entity: Entity) {
        self.loaded_chunks.insert(coord, entity);
        self.chunk_entities.insert(entity, coord);
        self.statistics.total_chunks_loaded += 1;
        self.statistics.current_loaded_chunks = self.loaded_chunks.len();

        if self.loaded_chunks.len() > self.statistics.peak_loaded_chunks {
            self.statistics.peak_loaded_chunks = self.loaded_chunks.len();
        }
    }

    pub fn unregister_chunk(&mut self, coord: &ChunkCoordinate) -> Option<Entity> {
        if let Some(entity) = self.loaded_chunks.remove(coord) {
            self.chunk_entities.remove(&entity);
            self.statistics.total_chunks_unloaded += 1;
            self.statistics.current_loaded_chunks = self.loaded_chunks.len();
            Some(entity)
        } else {
            None
        }
    }

    pub fn request_chunk_load(&mut self, coordinate: ChunkCoordinate, priority: LoadPriority) {
        if !self.is_chunk_loaded(&coordinate) {
            let request = ChunkLoadRequest {
                coordinate,
                priority,
            };

            // Insert with priority ordering (high priority first)
            match priority {
                LoadPriority::High => {
                    let index = self
                        .load_requests
                        .iter()
                        .position(|r| r.priority == LoadPriority::High)
                        .unwrap_or(self.load_requests.len());
                    self.load_requests.insert(index, request);
                }
                LoadPriority::Medium => {
                    let index = self
                        .load_requests
                        .iter()
                        .position(|r| r.priority == LoadPriority::Low)
                        .unwrap_or(self.load_requests.len());
                    self.load_requests.insert(index, request);
                }
                LoadPriority::Low => {
                    self.load_requests.push(request);
                }
            }
        }
    }

    pub fn get_next_load_request(&mut self) -> Option<ChunkLoadRequest> {
        if self.loaded_chunks.len() >= self.max_loaded_chunks {
            return None;
        }

        self.load_requests.pop()
    }

    pub fn get_chunks_to_load(&self, center: ChunkCoordinate) -> Vec<ChunkCoordinate> {
        let mut chunks_to_load = Vec::new();

        for dx in -self.view_distance..=self.view_distance {
            for dy in -self.view_distance..=self.view_distance {
                let coord = ChunkCoordinate {
                    x: center.x + dx,
                    y: center.y + dy,
                };

                if !self.is_chunk_loaded(&coord) {
                    chunks_to_load.push(coord);
                }
            }
        }

        // Sort by distance from center (closest first)
        chunks_to_load.sort_by(|a, b| {
            let dist_a = a.distance_to(&center);
            let dist_b = b.distance_to(&center);
            dist_a.cmp(&dist_b)
        });

        chunks_to_load
    }

    pub fn get_chunks_to_unload(&self, center: ChunkCoordinate) -> Vec<ChunkCoordinate> {
        let mut chunks_to_unload = Vec::new();

        for coord in self.loaded_chunks.keys() {
            let dx = (coord.x - center.x).abs();
            let dy = (coord.y - center.y).abs();

            if dx > self.unload_distance || dy > self.unload_distance {
                chunks_to_unload.push(*coord);
            }
        }

        chunks_to_unload
    }

    pub fn get_loaded_chunk_coordinates(&self) -> Vec<ChunkCoordinate> {
        self.loaded_chunks.keys().copied().collect()
    }

    pub fn get_chunks_in_radius(
        &self,
        center: ChunkCoordinate,
        radius: i32,
    ) -> Vec<ChunkCoordinate> {
        self.loaded_chunks
            .keys()
            .filter(|coord| coord.distance_to(&center) <= radius)
            .copied()
            .collect()
    }

    pub fn get_statistics(&self) -> &ChunkManagerStatistics {
        &self.statistics
    }

    pub fn reset_statistics(&mut self) {
        self.statistics = ChunkManagerStatistics::default();
    }

    pub fn get_cache_hit_rate(&self) -> f32 {
        let total_requests = self.statistics.cache_hits + self.statistics.cache_misses;
        if total_requests == 0 {
            0.0
        } else {
            self.statistics.cache_hits as f32 / total_requests as f32
        }
    }
}

pub fn chunk_loading_system(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    world_generator: Res<WorldGenerator>,
    query_positions: Query<&super::terrain_query::PositionComponent>,
) {
    if query_positions.is_empty() {
        return;
    }

    // Calculate center of all entities
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut count = 0;

    for pos in query_positions.iter() {
        sum_x += pos.x;
        sum_y += pos.y;
        count += 1;
    }

    if count == 0 {
        return;
    }

    let center_x = sum_x / count as f32;
    let center_y = sum_y / count as f32;

    // Convert to chunk coordinates
    let center_chunk = ChunkCoordinate::from_world_position(center_x, center_y, 10.0);

    // Load new chunks
    let chunks_to_load = chunk_manager.get_chunks_to_load(center_chunk);

    for coord in chunks_to_load {
        if chunk_manager.loaded_chunks.len() >= chunk_manager.max_loaded_chunks {
            break;
        }

        let chunk = world_generator.generate_chunk(coord);
        let entity = commands.spawn(chunk).id();
        chunk_manager.register_chunk(coord, entity);

        info!(
            "Loaded chunk ({}, {}) - Total loaded: {}",
            coord.x,
            coord.y,
            chunk_manager.loaded_chunks.len()
        );
    }
}

pub fn chunk_unloading_system(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    query_positions: Query<&super::terrain_query::PositionComponent>,
) {
    if query_positions.is_empty() {
        return;
    }

    // Calculate center of all entities
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut count = 0;

    for pos in query_positions.iter() {
        sum_x += pos.x;
        sum_y += pos.y;
        count += 1;
    }

    if count == 0 {
        return;
    }

    let center_x = sum_x / count as f32;
    let center_y = sum_y / count as f32;

    // Convert to chunk coordinates
    let center_chunk = ChunkCoordinate::from_world_position(center_x, center_y, 10.0);

    // Unload distant chunks
    let chunks_to_unload = chunk_manager.get_chunks_to_unload(center_chunk);

    for coord in chunks_to_unload {
        if let Some(entity) = chunk_manager.unregister_chunk(&coord) {
            commands.entity(entity).despawn();

            info!(
                "Unloaded chunk ({}, {}) - Total loaded: {}",
                coord.x,
                coord.y,
                chunk_manager.loaded_chunks.len()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_manager_creation() {
        let manager = ChunkManager::new();
        assert_eq!(manager.view_distance, VIEW_DISTANCE);
        assert_eq!(manager.unload_distance, UNLOAD_DISTANCE);
        assert_eq!(manager.max_loaded_chunks, MAX_LOADED_CHUNKS);
    }

    #[test]
    fn test_chunk_registration() {
        let mut manager = ChunkManager::new();
        let coord = ChunkCoordinate::new(0, 0);
        let entity = Entity::from_raw(1);

        manager.register_chunk(coord, entity);
        assert!(manager.is_chunk_loaded(&coord));
        assert_eq!(manager.get_chunk_entity(&coord), Some(entity));
        assert_eq!(manager.get_chunk_coordinate(entity), Some(coord));
    }

    #[test]
    fn test_chunk_unregistration() {
        let mut manager = ChunkManager::new();
        let coord = ChunkCoordinate::new(0, 0);
        let entity = Entity::from_raw(1);

        manager.register_chunk(coord, entity);
        let unregistered_entity = manager.unregister_chunk(&coord);

        assert_eq!(unregistered_entity, Some(entity));
        assert!(!manager.is_chunk_loaded(&coord));
        assert_eq!(manager.get_chunk_entity(&coord), None);
    }

    #[test]
    fn test_load_requests() {
        let mut manager = ChunkManager::new();
        let coord1 = ChunkCoordinate::new(0, 0);
        let coord2 = ChunkCoordinate::new(1, 1);

        manager.request_chunk_load(coord2, LoadPriority::Low);
        manager.request_chunk_load(coord1, LoadPriority::High);

        // High priority should be first
        assert_eq!(manager.get_next_load_request().unwrap().coordinate, coord1);
        assert_eq!(manager.get_next_load_request().unwrap().coordinate, coord2);
    }

    #[test]
    fn test_chunks_in_radius() {
        let mut manager = ChunkManager::new();
        let center = ChunkCoordinate::new(0, 0);
        let coord1 = ChunkCoordinate::new(1, 1);
        let coord2 = ChunkCoordinate::new(3, 3);

        manager.register_chunk(coord1, Entity::from_raw(1));
        manager.register_chunk(coord2, Entity::from_raw(2));

        let chunks_in_radius_2 = manager.get_chunks_in_radius(center, 2);
        assert_eq!(chunks_in_radius_2.len(), 1);
        assert!(chunks_in_radius_2.contains(&coord1));

        let chunks_in_radius_5 = manager.get_chunks_in_radius(center, 5);
        assert_eq!(chunks_in_radius_5.len(), 2);
    }
}
