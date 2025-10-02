use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, RwLock};
use crate::world_loader::{WorldLoader, list_available_worlds};
use crate::cached_world::CachedWorld;

fn parse_chunk_key(chunk_key: &str) -> Result<(i32, i32), Box<dyn std::error::Error>> {
    // Parse chunk key format "x,y" into tuple (x, y)
    let parts: Vec<&str> = chunk_key.split(',').collect();
    if parts.len() != 2 {
        return Err("Invalid chunk key format".into());
    }

    let x = parts[0].parse::<i32>()?;
    let y = parts[1].parse::<i32>()?;
    Ok((x, y))
}

pub fn start_simple_web_server() {
    println!("🌐 WEB_SERVER: Starting web server on port 54321");

    // Load the default world for the web server
    let world_loader = match WorldLoader::load_default() {
        Ok(loader) => {
            println!("✅ WEB_SERVER: World loaded: {} (seed: {})", loader.get_name(), loader.get_seed());

            // Initialize CachedWorld with the loaded world data
            let mut cached_chunks = std::collections::HashMap::new();
            for (chunk_key, chunk) in &loader.get_world_info().chunks {
                if let Ok((chunk_x, chunk_y)) = parse_chunk_key(chunk_key) {
                    cached_chunks.insert((chunk_x, chunk_y), chunk.layers.clone());
                }
            }

            let cached_world = CachedWorld {
                name: loader.get_name().to_string(),
                seed: loader.get_seed(),
                chunks: cached_chunks,
                is_loaded: true,
            };

            // Set the global cached world
            CachedWorld::global_set(cached_world);
            println!("✅ WEB_SERVER: CachedWorld initialized with {} chunks", loader.get_chunk_count());

            Arc::new(RwLock::new(loader))
        }
        Err(e) => {
            eprintln!("❌ WEB_SERVER: Failed to load world: {}", e);
            eprintln!("💡 WEB_SERVER: Please generate a world first using: cargo run --bin map_generator");
            // Create a placeholder world loader for error handling
            let placeholder = WorldLoader::load_from_file("maps/test_world.ron").unwrap_or_else(|_| {
                eprintln!("❌ WEB_SERVER: No test world available, creating minimal placeholder");
                // This would need to be handled better in production
                panic!("No world files available for web server");
            });
            Arc::new(RwLock::new(placeholder))
        }
    };

    let _world_loader_clone = Arc::clone(&world_loader);
    thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:54321").unwrap_or_else(|e| {
            eprintln!("❌ WEB_SERVER: Failed to bind to port 54321: {}", e);
            std::process::exit(1);
        });
        println!("✅ WEB_SERVER: Server listening on http://127.0.0.1:54321");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let world_loader = Arc::clone(&world_loader);
                    thread::spawn(move || {
                        handle_connection(stream, world_loader);
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

fn handle_connection(mut stream: TcpStream, world_loader: Arc<RwLock<WorldLoader>>) {
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

    // Handle POST requests for world selection
    if method == "POST" && path == "/api/world/select" {
        handle_world_selection(&mut stream, &world_loader, &request);
        return;
    }

    if method != "GET" && method != "POST" {
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
            let world = world_loader.read().unwrap();
            let seed = world.get_seed();
            let name = world.get_name();
            let chunk_count = world.get_chunk_count();
            let (bounds_min, bounds_max) = world.get_world_bounds();
            let json = format!(r#"{{"name": "{}", "seed": {}, "chunk_count": {}, "bounds": {{"min": {{"x": {}, "y": {}}}, "max": {{"x": {}, "y": {}}}}}}}"#,
                name, seed, chunk_count, bounds_min.0, bounds_min.1, bounds_max.0, bounds_max.1);
            send_response(&mut stream, "200 OK", "application/json", &json);
        }
        "/api/world/current" => {
            let world = world_loader.read().unwrap();
            let json = format!(r#"{{"name": "{}", "seed": {}, "chunk_count": {}, "file_path": "{}"}}"#,
                world.get_name(), world.get_seed(), world.get_chunk_count(), "maps/current");
            send_response(&mut stream, "200 OK", "application/json", &json);
        }
        "/api/worlds" => {
            // List all available worlds
            match list_available_worlds() {
                Ok(worlds) => {
                    let world_jsons: Vec<String> = worlds.iter().map(|w| {
                        format!(r#"{{"name": "{}", "file_path": "{}", "seed": {}, "chunk_count": {}, "version": "{}", "file_size": {}}}"#,
                            w.name, w.file_path, w.seed, w.chunk_count, w.version, w.file_size)
                    }).collect();
                    let json = format!(r#"{{"worlds": [{}]}}"#, world_jsons.join(","));
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
                // Check if multi-layer format is requested
                let use_multi_layer = path.contains("&layers=true") || path.contains("?layers=true");

                let json = if use_multi_layer {
                    // Use the new multi-layer format
                    if let Some(cached_world) = crate::cached_world::CachedWorld::global_get() {
                        cached_world.generate_multi_layer_chunks_json(path)
                    } else {
                        r#"{"error": "Failed to access cached world."}"#.to_string()
                    }
                } else {
                    // Use legacy terrain-only format for backward compatibility
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

                    format!("{{\"chunk_data\": {{{}}}}}", json_parts.join(", "))
                };

                send_response(&mut stream, "200 OK", "application/json", &json);
            } else {
                // No cached world loaded - return error
                send_response(&mut stream, "404 Not Found", "application/json", r#"{"error": "No world loaded. Please load a world first."}"#);
            }
        }
        path if path.starts_with("/js/") => {
            // Serve JavaScript files from the js directory
            let file_path = path.trim_start_matches('/');
            if let Ok(content) = std::fs::read_to_string(file_path) {
                send_response(&mut stream, "200 OK", "application/javascript", &content);
            } else {
                send_response(&mut stream, "404 Not Found", "text/plain", "JavaScript file not found");
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

fn handle_world_selection(mut stream: &mut TcpStream, world_loader: &Arc<RwLock<WorldLoader>>, request: &str) {
    // Extract JSON body from request
    let lines: Vec<&str> = request.lines().collect();

    if let Some(body_start) = lines.iter().position(|line| line.is_empty()) {
        let body = lines[body_start + 1..].join("\n");

        // Parse JSON to extract world name
        if let Ok(world_name) = parse_world_name_from_json(&body) {
            // Try to load the selected world
            match WorldLoader::load_by_name(&world_name) {
                Ok(new_world) => {
                    // Update the world loader
                    let world_seed = new_world.get_seed();
                    {
                        let mut loader = world_loader.write().unwrap();
                        *loader = new_world;
                    }

                    // Update the CachedWorld for the chunk API
                    let loader = world_loader.read().unwrap();
                    let mut cached_chunks = std::collections::HashMap::new();

                    // Load all chunks from the world into the CachedWorld
                    for (chunk_key, chunk) in &loader.get_world_info().chunks {
                        if let Ok((chunk_x, chunk_y)) = parse_chunk_key(chunk_key) {
                            cached_chunks.insert((chunk_x, chunk_y), chunk.layers.clone());
                        }
                    }

                    let cached_world = CachedWorld {
                        name: world_name.clone(),
                        seed: world_seed,
                        chunks: cached_chunks,
                        is_loaded: true,
                    };

                    // Set the global cached world
                    CachedWorld::global_set(cached_world);

                    let response_json = format!(r#"{{"success": true, "world_name": "{}", "seed": {}}}"#,
                        world_name, world_seed);
                    send_response(&mut stream, "200 OK", "application/json", &response_json);
                }
                Err(e) => {
                    let response_json = format!(r#"{{"success": false, "error": "Failed to load world '{}': {}"}}"#, world_name, e);
                    send_response(&mut stream, "404 Not Found", "application/json", &response_json);
                }
            }
        } else {
            send_response(&mut stream, "400 Bad Request", "application/json", r#"{"success": false, "error": "Invalid world name format"}"#);
        }
    } else {
        send_response(&mut stream, "400 Bad Request", "application/json", r#"{"success": false, "error": "No request body"}"#);
    }
}

fn parse_world_name_from_json(json_str: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Simple JSON parsing to extract world name
    // Expected format: {"world_name": "my_world"}

    if let Some(name_start) = json_str.find("\"world_name\"") {
        // Find the colon that comes after "world_name"
        let after_name_key = &json_str[name_start + 12..]; // +12 to skip "world_name"

        if let Some(colon_pos) = after_name_key.find(':') {
            let after_colon = &after_name_key[colon_pos + 1..];

            // The world name should be in quotes after the colon
            let name_with_quotes = after_colon.trim().trim_end_matches('}');

            if name_with_quotes.starts_with('"') && name_with_quotes.ends_with('"') {
                let world_name = &name_with_quotes[1..name_with_quotes.len()-1];
                Ok(world_name.to_string())
            } else {
                Err("Invalid JSON format - world name not in quotes".into())
            }
        } else {
            Err("Invalid JSON format - no colon after world_name".into())
        }
    } else {
        Err("No world_name field found".into())
    }
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

