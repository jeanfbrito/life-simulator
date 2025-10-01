use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, RwLock};
use crate::tilemap::{WorldGenerator, WorldConfig};
use crate::cached_world::CachedWorld;

pub fn start_simple_web_server() {
    println!("🌐 WEB_SERVER: Starting web server on port 54321");

    // Create world generator for terrain generation
    let world_generator = Arc::new(RwLock::new(WorldGenerator::new(WorldConfig::default())));

    thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:54321").unwrap_or_else(|e| {
            eprintln!("❌ WEB_SERVER: Failed to bind to port 54321: {}", e);
            std::process::exit(1);
        });
        println!("✅ WEB_SERVER: Server listening on http://127.0.0.1:54321");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let world_generator = Arc::clone(&world_generator);
                    thread::spawn(move || {
                        handle_connection(stream, world_generator);
                    });
                }
                Err(e) => {
                    eprintln!("❌ WEB_SERVER: Connection failed: {}", e);
                }
            }
        }
    });
    println!("✅ LIFE_SIMULATOR: Web server started at http://127.0.0.1:54321");
}

fn handle_connection(mut stream: TcpStream, world_generator: Arc<RwLock<WorldGenerator>>) {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).unwrap();
    eprintln!("🌐 WEB_SERVER: Received {} bytes", bytes_read);

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let lines: Vec<&str> = request.lines().collect();

    if lines.is_empty() {
        eprintln!("❌ WEB_SERVER: Empty request");
        return;
    }

    eprintln!("📨 WEB_SERVER: Request: {}", lines[0]);

    let request_line = lines[0];
    let parts: Vec<&str> = request_line.split_whitespace().collect();

    if parts.len() < 2 {
        send_response(&mut stream, "400 Bad Request", "text/plain", "Bad Request");
        return;
    }

    let method = parts[0];
    let path = parts[1];

    // Handle CORS preflight requests
    if method == "OPTIONS" {
        send_response(&mut stream, "200 OK", "text/plain", "");
        return;
    }

    // Handle POST requests for seed updates
    if method == "POST" && path == "/api/seed" {
        handle_seed_update(&mut stream, &world_generator, &request);
        return;
    }

    // Handle POST requests for saving maps
    if method == "POST" && path == "/api/save" {
        handle_map_save(&mut stream, &world_generator, &request);
        return;
    }

    // Handle POST requests for loading maps
    if method == "POST" && path == "/api/load" {
        handle_map_load(&mut stream, &world_generator, &request);
        return;
    }

    if method != "GET" {
        send_response(&mut stream, "405 Method Not Allowed", "text/plain", "Method Not Allowed");
        return;
    }

    match path {
        "/viewer.html" | "/" => {
            if let Ok(html) = std::fs::read_to_string("web-viewer/viewer.html") {
                send_response(&mut stream, "200 OK", "text/html", &html);
            } else {
                send_response(&mut stream, "404 Not Found", "text/plain", "HTML file not found");
            }
        }
        "/api/world_info" => {
            let seed = world_generator.read().unwrap().get_seed();
            let json = format!(r#"{{"center_chunk": {{"x": 0, "y": 0}}, "world_size": {{"width": 20, "height": 20}}, "seed": {}}}"#, seed);
            send_response(&mut stream, "200 OK", "application/json", &json);
        }
        "/api/seed" => {
            let seed = world_generator.read().unwrap().get_seed();
            let json = format!(r#"{{"seed": {}}}"#, seed);
            send_response(&mut stream, "200 OK", "application/json", &json);
        }
        "/api/worlds" => {
            // List all saved worlds
            match list_saved_worlds() {
                Ok(worlds) => {
                    let json = format!(r#"{{"worlds": [{}]}}"#, worlds.join(","));
                    send_response(&mut stream, "200 OK", "application/json", &json);
                }
                Err(e) => {
                    let json = format!(r#"{{"error": "{}"}}"#, e);
                    send_response(&mut stream, "500 Internal Server Error", "application/json", &json);
                }
            }
        }
        path if path.starts_with("/api/chunks") => {
            // Only use cached world data - no fallback to generator
            if crate::cached_world::CachedWorld::global_is_loaded() {
                let coords = parse_chunk_coords_from_path(path);
                let mut chunk_data = std::collections::HashMap::new();

                for &(chunk_x, chunk_y) in &coords {
                    let chunk_key = format!("{},{}", chunk_x, chunk_y);

                    if let Some(terrain_data) = crate::cached_world::CachedWorld::global_get_chunk(chunk_x, chunk_y) {
                        let data_str = terrain_data.iter()
                            .map(|row| format!("[{}]", row.iter().map(|tile| format!("\"{}\"", tile)).collect::<Vec<_>>().join(", ")))
                            .collect::<Vec<_>>()
                            .join(", ");
                        chunk_data.insert(chunk_key, data_str);
                    }
                }

                let json_parts: Vec<String> = chunk_data
                    .into_iter()
                    .map(|(key, data_str)| format!("\"{}\": [{}]", key, data_str))
                    .collect();

                let json = format!("{{\"chunk_data\": {{{}}}}}", json_parts.join(", "));
                send_response(&mut stream, "200 OK", "application/json", &json);
            } else {
                // No cached world loaded - return error
                send_response(&mut stream, "404 Not Found", "application/json", r#"{"error": "No world loaded. Please load a world first."}"#);
            }
        }
        _ => {
            send_response(&mut stream, "404 Not Found", "text/plain", "Not Found");
        }
    }
}

fn send_response(stream: &mut TcpStream, status: &str, content_type: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n{}",
        status,
        content_type,
        body.len(),
        body
    );

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn handle_seed_update(mut stream: &mut TcpStream, world_generator: &Arc<RwLock<WorldGenerator>>, request: &str) {
    // Extract JSON body from request
    let lines: Vec<&str> = request.lines().collect();

    if let Some(body_start) = lines.iter().position(|line| line.is_empty()) {
        let body = lines[body_start + 1..].join("\n");

        // Parse JSON to extract new seed
        if let Ok(new_seed) = parse_seed_from_json(&body) {
            // Update the world generator seed
            {
                let mut gen = world_generator.write().unwrap();
                gen.set_seed(new_seed);
            }

            let response_json = format!(r#"{{"success": true, "seed": {}}}"#, new_seed);
            send_response(&mut stream, "200 OK", "application/json", &response_json);
        } else {
            send_response(&mut stream, "400 Bad Request", "application/json", r#"{"success": false, "error": "Invalid seed format"}"#);
        }
    } else {
        send_response(&mut stream, "400 Bad Request", "application/json", r#"{"success": false, "error": "No request body"}"#);
    }
}

fn parse_seed_from_json(json_str: &str) -> Result<u64, Box<dyn std::error::Error>> {
    // Simple JSON parsing to extract seed value
    // Expected format: {"seed": 12345}

    if let Some(seed_start) = json_str.find("\"seed\"") {
        // Find the colon that comes after "seed"
        let after_seed_key = &json_str[seed_start + 6..]; // +6 to skip "seed"

        if let Some(colon_pos) = after_seed_key.find(':') {
            let after_colon = &after_seed_key[colon_pos + 1..];

            // The seed value should be immediately after the colon
            let seed_str = after_colon.trim().trim_end_matches('}');

            seed_str.parse::<u64>().map_err(|e| e.into())
        } else {
            Err("Invalid JSON format - no colon after seed".into())
        }
    } else {
        Err("No seed field found".into())
    }
}

fn handle_map_save(mut stream: &mut TcpStream, world_generator: &Arc<RwLock<WorldGenerator>>, request: &str) {
    // Extract JSON body from request
    let lines: Vec<&str> = request.lines().collect();

    if let Some(body_start) = lines.iter().position(|line| line.is_empty()) {
        let body = lines[body_start + 1..].join("\n");

        // Parse JSON to extract file name and map name
        if let Ok((file_path, map_name)) = parse_save_request_from_json(&body) {
            // Create a saves directory if it doesn't exist
            let full_path = format!("saves/{}.ron", file_path);

            // Generate chunks around center for saving
            let mut chunks = std::collections::HashMap::new();
            let center_x = 0;
            let center_y = 0;
            let radius = 3; // Save 7x7 chunk area around center

            for chunk_x in (center_x - radius)..=(center_x + radius) {
                for chunk_y in (center_y - radius)..=(center_y + radius) {
                    let chunk_tiles = world_generator.read().unwrap().generate_procedural_chunk(chunk_x, chunk_y);
                    chunks.insert((chunk_x, chunk_y), chunk_tiles);
                }
            }

            let serialized_world = crate::serialization::WorldSerializer::create_serialized_world(
                map_name.clone(),
                world_generator.read().unwrap().get_seed(),
                crate::tilemap::WorldConfig::default(),
                chunks,
            );

            match crate::serialization::WorldSerializer::save_world(&serialized_world, &full_path) {
                Ok(()) => {
                    let response_json = format!(r#"{{"success": true, "message": "World saved as {}", "path": "{}"}}"#, map_name, full_path);
                    send_response(&mut stream, "200 OK", "application/json", &response_json);
                }
                Err(e) => {
                    let response_json = format!(r#"{{"success": false, "error": "Failed to save world: {}"}}"#, e);
                    send_response(&mut stream, "500 Internal Server Error", "application/json", &response_json);
                }
            }
        } else {
            send_response(&mut stream, "400 Bad Request", "application/json", r#"{"success": false, "error": "Invalid save request format"}"#);
        }
    } else {
        send_response(&mut stream, "400 Bad Request", "application/json", r#"{"success": false, "error": "No request body"}"#);
    }
}

fn handle_map_load(mut stream: &mut TcpStream, world_generator: &Arc<RwLock<WorldGenerator>>, request: &str) {
    // Extract JSON body from request
    let lines: Vec<&str> = request.lines().collect();

    if let Some(body_start) = lines.iter().position(|line| line.is_empty()) {
        let body = lines[body_start + 1..].join("\n");

        // Parse JSON to extract file path
        if let Ok(file_path) = parse_load_request_from_json(&body) {
            let full_path = format!("saves/{}.ron", file_path);

            match crate::serialization::WorldSerializer::load_world(&full_path) {
                Ok(serialized_world) => {
                    // Update the world generator with the loaded seed
                    {
                        let mut gen = world_generator.write().unwrap();
                        gen.set_seed(serialized_world.seed);
                    }

                    // Populate the global cached world with the loaded data
                    let cached_world = crate::cached_world::CachedWorld::from_serialized(serialized_world.clone());
                    crate::cached_world::CachedWorld::global_set(cached_world);

                    let response_json = format!(r#"{{"success": true, "message": "World loaded successfully", "name": "{}", "seed": {}}}"#, serialized_world.name, serialized_world.seed);
                    send_response(&mut stream, "200 OK", "application/json", &response_json);
                }
                Err(e) => {
                    let response_json = format!(r#"{{"success": false, "error": "Failed to load world: {}"}}"#, e);
                    send_response(&mut stream, "404 Not Found", "application/json", &response_json);
                }
            }
        } else {
            send_response(&mut stream, "400 Bad Request", "application/json", r#"{"success": false, "error": "Invalid load request format"}"#);
        }
    } else {
        send_response(&mut stream, "400 Bad Request", "application/json", r#"{"success": false, "error": "No request body"}"#);
    }
}

fn parse_save_request_from_json(json_str: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Expected format: {"file_name": "my_world", "map_name": "My World"}
    let mut file_name = None;
    let mut map_name = None;

    // Extract file_name
    if let Some(file_start) = json_str.find("\"file_name\"") {
        let after_file_key = &json_str[file_start + 11..]; // +11 to skip "file_name"
        if let Some(colon_pos) = after_file_key.find(':') {
            let after_colon = &after_file_key[colon_pos + 1..];
            let file_str = after_colon.trim().trim_end_matches('}').trim_matches('"');
            file_name = Some(file_str.to_string());
        }
    }

    // Extract map_name
    if let Some(name_start) = json_str.find("\"map_name\"") {
        let after_name_key = &json_str[name_start + 10..]; // +10 to skip "map_name"
        if let Some(colon_pos) = after_name_key.find(':') {
            let after_colon = &after_name_key[colon_pos + 1..];
            let name_str = after_colon.trim().trim_end_matches('}').trim_matches('"');
            map_name = Some(name_str.to_string());
        }
    }

    match (file_name, map_name) {
        (Some(file), Some(name)) => Ok((file, name)),
        _ => Err("Missing required fields".into()),
    }
}

fn parse_load_request_from_json(json_str: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Expected format: {"file_name": "my_world"}

    if let Some(file_start) = json_str.find("\"file_name\"") {
        let after_file_key = &json_str[file_start + 11..]; // +11 to skip "file_name"
        if let Some(colon_pos) = after_file_key.find(':') {
            let after_colon = &after_file_key[colon_pos + 1..];
            let file_str = after_colon.trim().trim_end_matches('}').trim_matches('"');
            return Ok(file_str.to_string());
        }
    }

    Err("No file_name field found".into())
}

fn parse_chunk_coords_from_path(path: &str) -> Vec<(i32, i32)> {
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

fn list_saved_worlds() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let saves_dir = std::path::Path::new("saves");

    // Create saves directory if it doesn't exist
    if !saves_dir.exists() {
        std::fs::create_dir_all(saves_dir)?;
        return Ok(vec![]);
    }

    let mut worlds = Vec::new();

    for entry in std::fs::read_dir(saves_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("ron") {
            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                worlds.push(format!("\"{}\"", file_stem));
            }
        }
    }

    worlds.sort();
    Ok(worlds)
}

