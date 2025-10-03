pub mod biome;
pub mod chunk;
pub mod chunk_manager;
pub mod terrain;
pub mod terrain_query;
pub mod world_generator;

pub use biome::{BiomeGenerator, BiomeType};
pub use chunk::{Chunk, ChunkCoordinate, CHUNK_SIZE};
pub use chunk_manager::{ChunkLoadRequest, ChunkManager, ChunkManagerStatistics, LoadPriority};
pub use terrain::{TerrainProperties, TerrainType};
pub use terrain_query::{AreaAnalysis, PositionComponent, TerrainQuery, TerrainQuerySystem};
pub use world_generator::{WorldConfig, WorldGenerator, WorldMetadata, WorldStatistics};

use crate::tilemap::chunk_manager::{MAX_LOADED_CHUNKS, UNLOAD_DISTANCE, VIEW_DISTANCE};
use bevy::log::{info, warn};
use bevy::prelude::*;

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkManager>()
            .insert_resource(WorldGenerator::new(WorldConfig::default()))
            .add_systems(Startup, tilemap_init_system)
            .add_systems(
                Update,
                (
                    chunk_manager::chunk_loading_system,
                    chunk_manager::chunk_unloading_system,
                    terrain_query::terrain_query_api_system,
                ),
            );
    }
}

fn tilemap_init_system(_commands: Commands, world_generator: Res<WorldGenerator>) {
    info!("TILEMAP: Initializing headless tilemap system");

    // Generate initial world statistics
    let world_stats = world_generator.generate_world_statistics();
    info!(
        "TILEMAP: World generated - {} chunks, {} tiles, {:.1}% walkable",
        world_stats.total_chunks,
        world_stats.total_tiles,
        world_stats.get_walkable_percentage() * 100.0
    );

    // Find spawn point for entities
    if let Some((spawn_x, spawn_y)) = world_generator.find_spawn_point() {
        info!("TILEMAP: Spawn point found at ({}, {})", spawn_x, spawn_y);
    } else {
        warn!("TILEMAP: Could not find suitable spawn point");
    }

    // Log world metadata
    let world_metadata = WorldMetadata::new(WorldConfig::default(), world_stats);
    info!(
        "TILEMAP: World metadata generated at {:?}",
        world_metadata.generated_at
    );
}

#[derive(Debug, Clone, Resource)]
pub struct TilemapConfig {
    pub chunk_size: usize,
    pub tile_size: f32,
    pub view_distance: i32,
    pub unload_distance: i32,
    pub max_loaded_chunks: usize,
}

impl Default for TilemapConfig {
    fn default() -> Self {
        Self {
            chunk_size: CHUNK_SIZE,
            tile_size: 10.0,
            view_distance: VIEW_DISTANCE,
            unload_distance: UNLOAD_DISTANCE,
            max_loaded_chunks: MAX_LOADED_CHUNKS,
        }
    }
}

// Event systems for external communication
#[derive(Debug, Clone, Event)]
pub struct TerrainQueryEvent {
    pub position: (i32, i32),
    pub radius: Option<i32>,
}

#[derive(Debug, Clone, Event)]
pub struct TerrainQueryResponse {
    pub terrain: Vec<TerrainQuery>,
    pub analysis: Option<AreaAnalysis>,
}

#[derive(Debug, Clone, Event)]
pub struct PathfindingRequestEvent {
    pub start: (i32, i32),
    pub goal: (i32, i32),
}

#[derive(Debug, Clone, Event)]
pub struct PathfindingResponseEvent {
    pub path: Option<Vec<(i32, i32)>>,
    pub cost: Option<f32>,
}

#[derive(Debug, Clone, Event)]
pub struct WorldStatisticsRequestEvent;

#[derive(Debug, Clone, Event)]
pub struct WorldStatisticsResponseEvent {
    pub chunk_manager_stats: ChunkManagerStatistics,
    pub world_stats: Option<WorldStatistics>,
}

// Event handlers for WebSocket/integration communication
pub fn handle_terrain_query_events(
    mut query_events: EventReader<TerrainQueryEvent>,
    mut response_events: EventWriter<TerrainQueryResponse>,
    chunk_manager: Res<ChunkManager>,
    chunks_query: Query<&Chunk>,
    _world_generator: Res<WorldGenerator>,
) {
    for event in query_events.read() {
        match event.radius {
            Some(radius) => {
                let terrain = TerrainQuerySystem::get_terrain_in_radius(
                    &chunk_manager,
                    event.position.0,
                    event.position.1,
                    radius,
                    &chunks_query,
                );

                let analysis = TerrainQuerySystem::analyze_area(
                    &chunk_manager,
                    event.position.0,
                    event.position.1,
                    radius,
                    &chunks_query,
                );

                response_events.write(TerrainQueryResponse {
                    terrain,
                    analysis: Some(analysis),
                });
            }
            None => {
                if let Some(terrain) = TerrainQuerySystem::get_terrain_at_position(
                    &chunk_manager,
                    event.position.0,
                    event.position.1,
                    &chunks_query,
                ) {
                    response_events.write(TerrainQueryResponse {
                        terrain: vec![terrain],
                        analysis: None,
                    });
                }
            }
        }
    }
}

pub fn handle_pathfinding_events(
    mut path_events: EventReader<PathfindingRequestEvent>,
    mut response_events: EventWriter<PathfindingResponseEvent>,
    chunk_manager: Res<ChunkManager>,
    chunks_query: Query<&Chunk>,
) {
    for event in path_events.read() {
        let path =
            TerrainQuerySystem::find_path(&chunk_manager, event.start, event.goal, &chunks_query);

        let cost = path.as_ref().map(|p| {
            if p.len() < 2 {
                0.0
            } else {
                let mut total_cost = 0.0;
                for i in 1..p.len() {
                    let dx = p[i].0 - p[i - 1].0;
                    let dy = p[i].1 - p[i - 1].1;
                    total_cost += ((dx * dx + dy * dy) as f32).sqrt();
                }
                total_cost
            }
        });

        response_events.write(PathfindingResponseEvent { path, cost });
    }
}

pub fn handle_world_statistics_events(
    mut stats_events: EventReader<WorldStatisticsRequestEvent>,
    mut response_events: EventWriter<WorldStatisticsResponseEvent>,
    chunk_manager: Res<ChunkManager>,
    _world_generator: Res<WorldGenerator>,
) {
    for _event in stats_events.read() {
        let world_stats = Some(_world_generator.generate_world_statistics());

        response_events.write(WorldStatisticsResponseEvent {
            chunk_manager_stats: chunk_manager.get_statistics().clone(),
            world_stats,
        });
    }
}

// Utility functions for external systems
pub fn get_tilemap_info(
    chunk_manager: &ChunkManager,
    world_generator: &WorldGenerator,
) -> serde_json::Value {
    serde_json::json!({
        "loaded_chunks": chunk_manager.get_loaded_chunk_coordinates(),
        "chunk_count": chunk_manager.get_statistics().current_loaded_chunks,
        "cache_hit_rate": chunk_manager.get_cache_hit_rate(),
        "world_bounds": world_generator.get_world_bounds()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tilemap_config_default() {
        let config = TilemapConfig::default();
        assert_eq!(config.chunk_size, CHUNK_SIZE);
        assert_eq!(config.tile_size, 10.0);
        assert_eq!(config.view_distance, VIEW_DISTANCE);
        assert_eq!(config.unload_distance, UNLOAD_DISTANCE);
        assert_eq!(config.max_loaded_chunks, MAX_LOADED_CHUNKS);
    }

    #[test]
    fn test_events_creation() {
        let query_event = TerrainQueryEvent {
            position: (0, 0),
            radius: Some(5),
        };

        let path_event = PathfindingRequestEvent {
            start: (0, 0),
            goal: (10, 10),
        };

        assert_eq!(query_event.position, (0, 0));
        assert_eq!(query_event.radius, Some(5));
        assert_eq!(path_event.start, (0, 0));
        assert_eq!(path_event.goal, (10, 10));
    }
}
