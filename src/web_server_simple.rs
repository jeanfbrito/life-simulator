use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, RwLock};
use crate::tilemap::{WorldGenerator, WorldConfig};

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
        path if path.starts_with("/api/chunks") => {
            // Generate procedural terrain for requested chunks using WorldGenerator
            let json = world_generator.read().unwrap().generate_chunks_json(path);
            send_response(&mut stream, "200 OK", "application/json", &json);
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

