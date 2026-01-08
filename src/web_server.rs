use bevy::log::{info, error};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::net::TcpListener;
const DEFAULT_WEB_PORT: u16 = 54321;

fn resolve_web_server_port() -> u16 {
    std::env::var("LIFE_SIM_WEB_PORT")
        .or_else(|_| std::env::var("LIFE_SIM_PORT"))
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|port| *port != 0)
        .unwrap_or(DEFAULT_WEB_PORT)
}
use std::thread;

use crate::tilemap::{Chunk, CHUNK_SIZE};

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

pub fn start_simple_web_server() -> u16 {
    let port = resolve_web_server_port();
    let bind_address = format!("0.0.0.0:{}", port);
    println!("🌐 WEB_SERVER: Starting web server on {}", bind_address);
    thread::spawn(move || {
        let listener = TcpListener::bind(&bind_address).unwrap_or_else(|e| {
            eprintln!("❌ WEB_SERVER: Failed to bind to {}: {}", bind_address, e);
            std::process::exit(1);
        });
        println!("✅ WEB_SERVER: Server listening on http://{}", bind_address);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(move || {
                        handle_http_connection(stream);
                    });
                }
                Err(e) => {
                    error!("WEB_SERVER: Connection failed: {}", e);
                }
            }
        }
    });
    info!("WEB_SERVER: Web server started in background thread");
    port
}

fn handle_http_connection(mut stream: std::net::TcpStream) {
    use std::io::{BufRead, BufReader, Write};

    let mut reader = BufReader::new(stream.try_clone().unwrap());

    // Read HTTP request
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_ok() {
        if request_line.starts_with("GET ") {
            // Extract path
            let path = request_line.split_whitespace().nth(1).unwrap_or("/");

            // Read remaining headers
            while reader.read_line(&mut String::new()).is_ok() {
                // Skip headers
            }

            match path {
                "/viewer.html" | "/" => {
                    // Serve the HTML viewer
                    if let Ok(html) = std::fs::read_to_string("web-viewer/viewer.html") {
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                            html.len(),
                            html
                        );
                        let _ = stream.write_all(response.as_bytes());
                    }
                }
                "/api/world_info" => {
                    // Return world information
                    let response_data = json!({
                        "center_chunk": {"x": 0, "y": 0},
                        "world_size": {"width": 20, "height": 20}
                    });

                    let json = serde_json::to_string(&response_data).unwrap();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                        json.len(),
                        json
                    );
                    let _ = stream.write_all(response.as_bytes());
                }
                path if path.starts_with("/api/chunks") => {
                    // Parse chunk coordinates from query parameters
                    let coords = parse_chunk_coords(&path);
                    let chunk_data = generate_chunk_data(&coords);

                    let response_data = json!({
                        "chunk_data": chunk_data
                    });

                    let json = serde_json::to_string(&response_data).unwrap();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                        json.len(),
                        json
                    );
                    let _ = stream.write_all(response.as_bytes());
                }
                _ => {
                    // 404 Not Found
                    let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes());
                }
            }
        }
    }
}

fn parse_chunk_coords(path: &str) -> Vec<(i32, i32)> {
    // Extract coordinates from path like /api/chunks?coords=0,0&coords=1,0
    if let Some(query_part) = path.split('?').nth(1) {
        let mut coords = Vec::new();
        for param in query_part.split('&') {
            if let Some(coord_part) = param.strip_prefix("coords=") {
                if let Some((x_str, y_str)) = coord_part.split_once(',') {
                    if let (Ok(x), Ok(y)) = (x_str.parse::<i32>(), y_str.parse::<i32>()) {
                        coords.push((x, y));
                    }
                }
            }
        }
        return coords;
    }
    // Default to center chunk (0, 0)
    vec![(0, 0)]
}

fn generate_chunk_data(coords: &[(i32, i32)]) -> HashMap<String, Vec<Vec<String>>> {
    let mut chunk_data = HashMap::new();

    for &(chunk_x, chunk_y) in coords {
        let chunk_key = format!("{},{}", chunk_x, chunk_y);
        let terrain_data = generate_procedural_chunk(chunk_x, chunk_y);
        chunk_data.insert(chunk_key, terrain_data);
    }

    chunk_data
}

fn generate_procedural_chunk(chunk_x: i32, chunk_y: i32) -> Vec<Vec<String>> {
    let mut chunk = Vec::with_capacity(CHUNK_SIZE);
    let seed = (chunk_x as u64).wrapping_mul(1000).wrapping_add(chunk_y as u64);
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    for y in 0..CHUNK_SIZE {
        let mut row = Vec::with_capacity(CHUNK_SIZE);
        for x in 0..CHUNK_SIZE {
            let world_x = chunk_x * CHUNK_SIZE as i32 + x as i32;
            let world_y = chunk_y * CHUNK_SIZE as i32 + y as i32;

            // Generate terrain based on patterns and noise
            let terrain_type = generate_terrain_type(world_x, world_y, &mut rng);
            row.push(terrain_type);
        }
        chunk.push(row);
    }

    chunk
}

fn generate_terrain_type(world_x: i32, world_y: i32, rng: &mut rand::rngs::StdRng) -> String {
    // Create some interesting patterns
    let distance_from_origin = ((world_x * world_x + world_y * world_y) as f64).sqrt();
    let pattern1 = ((world_x as f64 * 0.1).sin() + (world_y as f64 * 0.1).cos()) * 0.5 + 0.5;
    let pattern2 = ((world_x as f64 * 0.05).sin() + (world_y as f64 * 0.05).cos()) * 0.5 + 0.5;
    let random_factor = rng.gen::<f64>();

    if distance_from_origin < 20.0 {
        "DeepWater".to_string()
    } else if distance_from_origin < 40.0 {
        "Water".to_string()
    } else if pattern1 < 0.2 {
        "Forest".to_string()
    } else if pattern1 < 0.3 {
        "Mountain".to_string()
    } else if pattern1 < 0.4 {
        "Stone".to_string()
    } else if pattern2 < 0.3 {
        "Sand".to_string()
    } else if pattern2 < 0.5 && random_factor < 0.3 {
        "Swamp".to_string()
    } else if distance_from_origin > 80.0 {
        "Snow".to_string()
    } else if distance_from_origin > 60.0 && random_factor < 0.5 {
        "Desert".to_string()
    } else {
        "Grass".to_string()
    }
}

// Integration with existing terrain system
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
