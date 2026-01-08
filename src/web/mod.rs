use bevy::prelude::*;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use crate::tilemap::{Chunk, WorldGenerator, CHUNK_SIZE};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub data: Option<serde_json::Value>,
}

impl WebSocketMessage {
    pub fn new(message_type: &str, data: Option<serde_json::Value>) -> Self {
        Self {
            message_type: message_type.to_string(),
            data,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Resource)]
pub struct WebSocketServer {
    pub clients: Arc<RwLock<HashMap<String, WebSocketStream<TcpStream>>>>,
    pub world_data: Arc<RwLock<HashMap<String, Vec<Vec<String>>>>>, // chunk_key -> 2D array of terrain strings
}

impl Default for WebSocketServer {
    fn default() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            world_data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

pub struct WebSocketPlugin;

impl Plugin for WebSocketPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WebSocketServer>()
            .add_systems(Startup, start_websocket_server)
            .add_systems(Update, handle_websocket_requests);
    }
}

fn start_websocket_server() {
    info!("WEB: Starting WebSocket server on port 8080");

    // This would need to run in a separate tokio runtime
    // For now, we'll integrate it differently since Bevy manages its own runtime
    tokio::spawn(async move {
        let listener = TcpListener::bind("0.0.0.0:8080")
            .await
            .expect("Failed to bind to port 8080");

        info!("WEB: WebSocket server listening on ws://0.0.0.0:8080");

        while let Ok((stream, addr)) = listener.accept().await {
            info!("WEB: New connection from {}", addr);

            let ws_stream = tokio_tungstenite::accept_async(stream)
                .await
                .expect("Error during WebSocket handshake");

            // Handle connection in a separate task
            tokio::spawn(handle_connection(ws_stream, addr.to_string()));
        }
    });
}

async fn handle_connection(mut ws_stream: WebSocketStream<TcpStream>, client_id: String) {
    info!("WEB: Handling connection from {}", client_id);

    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                    handle_client_message(&mut ws_stream, parsed).await;
                } else {
                    error!("WEB: Failed to parse message from {}: {}", client_id, text);
                }
            }
            Ok(Message::Close(_)) => {
                info!("WEB: Client {} disconnected", client_id);
                break;
            }
            Err(e) => {
                error!("WEB: WebSocket error from {}: {}", client_id, e);
                break;
            }
            _ => {}
        }
    }
}

async fn handle_client_message(
    ws_stream: &mut WebSocketStream<TcpStream>,
    message: serde_json::Value,
) {
    if let Some(msg_type) = message.get("type").and_then(|v| v.as_str()) {
        match msg_type {
            "get_world_info" => {
                // Send world information
                let response = WebSocketMessage::new(
                    "world_info",
                    Some(serde_json::json!({
                        "center_chunk": {"x": 0, "y": 0},
                        "world_size": {"width": 20, "height": 20}
                    })),
                );

                if let Ok(json) = response.to_json() {
                    let _ = ws_stream.send(Message::Text(json)).await;
                }
            }
            "get_chunks" => {
                // Handle chunk requests
                if let Some(coords) = message.get("chunk_coordinates").and_then(|v| v.as_array()) {
                    handle_chunk_request(ws_stream, coords).await;
                }
            }
            "regenerate_world" => {
                info!("WEB: Client requested world regeneration");
                // Send acknowledgment
                let response = WebSocketMessage::new(
                    "world_regenerated",
                    Some(serde_json::json!({
                        "success": true
                    })),
                );

                if let Ok(json) = response.to_json() {
                    let _ = ws_stream.send(Message::Text(json)).await;
                }
            }
            _ => {
                warn!("WEB: Unknown message type: {}", msg_type);
            }
        }
    }
}

async fn handle_chunk_request(
    ws_stream: &mut WebSocketStream<TcpStream>,
    chunk_coords: &[serde_json::Value],
) {
    // Generate sample chunk data for now
    let mut chunk_data = HashMap::new();

    for coord in chunk_coords {
        if let (Some(x), Some(y)) = (
            coord.get("x").and_then(|v| v.as_i64()),
            coord.get("y").and_then(|v| v.as_i64()),
        ) {
            let chunk_key = format!("{},{}", x, y);
            let terrain_data = generate_sample_chunk(x, y);
            chunk_data.insert(chunk_key, terrain_data);
        }
    }

    let response = WebSocketMessage::new(
        "chunk_data",
        Some(serde_json::json!({
            "chunk_data": chunk_data
        })),
    );

    if let Ok(json) = response.to_json() {
        let _ = ws_stream.send(Message::Text(json)).await;
    }
}

fn generate_sample_chunk(chunk_x: i64, chunk_y: i64) -> Vec<Vec<String>> {
    let mut chunk = Vec::with_capacity(CHUNK_SIZE);

    for y in 0..CHUNK_SIZE {
        let mut row = Vec::with_capacity(CHUNK_SIZE);
        for x in 0..CHUNK_SIZE {
            // Simple pattern generation based on position
            let world_x = chunk_x * CHUNK_SIZE as i64 + x as i64;
            let world_y = chunk_y * CHUNK_SIZE as i64 + y as i64;

            let terrain_type = if (world_x + world_y) % 20 == 0 {
                "Water".to_string()
            } else if (world_x * world_y) % 30 == 0 {
                "Forest".to_string()
            } else if (world_x - world_y) % 25 == 0 {
                "Stone".to_string()
            } else if world_x % 15 == 0 || world_y % 15 == 0 {
                "Sand".to_string()
            } else {
                "Grass".to_string()
            };

            row.push(terrain_type);
        }
        chunk.push(row);
    }

    chunk
}

fn handle_websocket_requests(
    _world_generator: Res<WorldGenerator>,
    _websocket_server: Res<WebSocketServer>,
    _chunk_manager: Res<crate::tilemap::ChunkManager>,
    _chunks_query: Query<&Chunk>,
) {
    // This system could handle requests from the game that need to be sent to WebSocket clients
    // For now, we'll keep it simple
}

// Helper functions for sending messages to all connected clients
pub async fn broadcast_message(server: &WebSocketServer, _message: WebSocketMessage) {
    let clients = server.clients.read().await;

    for (_, _ws_stream) in clients.iter() {
        // Note: This would need mutable access to the streams
        // In a real implementation, you'd need to handle this differently
        // Perhaps using a message queue system
    }
}

// Terrain conversion helpers
pub fn terrain_type_to_string(terrain: &crate::tilemap::TerrainType) -> String {
    match terrain {
        crate::tilemap::TerrainType::Grass => "Grass".to_string(),
        crate::tilemap::TerrainType::Stone => "Stone".to_string(),
        crate::tilemap::TerrainType::Sand => "Sand".to_string(),
        crate::tilemap::TerrainType::Water => "Water".to_string(),
        crate::tilemap::TerrainType::Dirt => "Dirt".to_string(),
        crate::tilemap::TerrainType::Snow => "Snow".to_string(),
        crate::tilemap::TerrainType::Forest => "Forest".to_string(),
        crate::tilemap::TerrainType::Mountain => "Mountain".to_string(),
        crate::tilemap::TerrainType::DeepWater => "DeepWater".to_string(),
        crate::tilemap::TerrainType::ShallowWater => "ShallowWater".to_string(),
        crate::tilemap::TerrainType::Swamp => "Swamp".to_string(),
        crate::tilemap::TerrainType::Desert => "Desert".to_string(),
    }
}

pub fn serialize_chunk(chunk: &Chunk) -> Vec<Vec<String>> {
    let mut serialized = Vec::with_capacity(CHUNK_SIZE);

    for y in 0..CHUNK_SIZE {
        let mut row = Vec::with_capacity(CHUNK_SIZE);
        for x in 0..CHUNK_SIZE {
            if let Some(terrain) = chunk.get_tile(x, y) {
                row.push(terrain_type_to_string(&terrain));
            } else {
                row.push("Grass".to_string()); // Default fallback
            }
        }
        serialized.push(row);
    }

    serialized
}
