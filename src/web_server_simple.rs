use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::collections::HashMap;
use rand::{Rng, SeedableRng};

pub fn start_simple_web_server() {
    println!("🌐 WEB_SERVER: Starting web server on port 54321");
    thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:54321").unwrap_or_else(|e| {
            eprintln!("❌ WEB_SERVER: Failed to bind to port 54321: {}", e);
            std::process::exit(1);
        });
        println!("✅ WEB_SERVER: Server listening on http://127.0.0.1:54321");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(move || {
                        handle_connection(stream);
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

fn handle_connection(mut stream: TcpStream) {
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
            let json = r#"{"center_chunk": {"x": 0, "y": 0}, "world_size": {"width": 20, "height": 20}}"#;
            send_response(&mut stream, "200 OK", "application/json", json);
        }
        path if path.starts_with("/api/chunks") => {
            // Generate procedural terrain for requested chunks
            let json = generate_chunks_json(path);
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

fn generate_chunks_json(path: &str) -> String {
    // Parse coordinates from path like /api/chunks?coords=0,0&coords=1,0
    let coords = parse_chunk_coords(path);
    let mut chunk_data = HashMap::new();

    for &(chunk_x, chunk_y) in &coords {
        let chunk_key = format!("{},{}", chunk_x, chunk_y);
        let terrain_data = generate_procedural_chunk(chunk_x, chunk_y);
        chunk_data.insert(chunk_key, terrain_data);
    }

    // Convert to JSON string
    let mut json_parts = Vec::new();
    for (key, data) in chunk_data {
        let data_str = data.iter()
            .map(|row| format!("[{}]", row.iter().map(|tile| format!("\"{}\"", tile)).collect::<Vec<_>>().join(", ")))
            .collect::<Vec<_>>()
            .join(", ");
        json_parts.push(format!("\"{}\": [{}]", key, data_str));
    }

    format!("{{\"chunk_data\": {{{}}}}}", json_parts.join(", "))
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

fn generate_procedural_chunk(chunk_x: i32, chunk_y: i32) -> Vec<Vec<String>> {
    let mut chunk = Vec::with_capacity(16);
    let seed = (chunk_x as u64).wrapping_mul(1000).wrapping_add(chunk_y as u64);
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    for y in 0..16 {
        let mut row = Vec::with_capacity(16);
        for x in 0..16 {
            let world_x = chunk_x * 16 + x;
            let world_y = chunk_y * 16 + y;

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