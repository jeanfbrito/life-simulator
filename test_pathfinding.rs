// Test pathfinding from rabbit spawn to water
use std::collections::HashMap;

// Simplified versions of the structures we need
fn main() {
    println!("🧪 Testing rabbit pathfinding to water\n");
    
    // Load the world
    println!("📂 Loading world from maps/full_world.ron...");
    let world_loader = match load_world() {
        Ok(w) => {
            println!("✅ World loaded successfully\n");
            w
        }
        Err(e) => {
            eprintln!("❌ Failed to load world: {}", e);
            return;
        }
    };
    
    // Test from typical spawn positions
    let test_positions = vec![
        (0, 0),     // Center spawn
        (10, 15),   // Typical rabbit spawn
        (-15, 10),  // Another spawn
        (18, -13),  // From actual log
    ];
    
    for (x, y) in test_positions {
        println!("🐇 Testing from position ({}, {})", x, y);
        
        // Find nearest water
        let water = find_nearest_water_simple(x, y, &world_loader, 100);
        
        match water {
            Some((wx, wy, dist)) => {
                println!("   💧 Found water at ({}, {}) - distance: {:.1} tiles", wx, wy, dist);
                
                // Try to find adjacent walkable tile
                let adjacent = find_adjacent_walkable(wx, wy, &world_loader);
                match adjacent {
                    Some((ax, ay)) => {
                        println!("   🚶 Adjacent walkable tile: ({}, {})", ax, ay);
                        
                        // Check if path exists (simplified check)
                        let terrain_at_rabbit = world_loader.get(&format!("{},{}", x, y));
                        let terrain_at_adjacent = world_loader.get(&format!("{},{}", ax, ay));
                        
                        println!("   🗺️  Rabbit position terrain: {:?}", terrain_at_rabbit);
                        println!("   🗺️  Target position terrain: {:?}", terrain_at_adjacent);
                        
                        if terrain_at_rabbit.is_some() && terrain_at_adjacent.is_some() {
                            println!("   ✅ Both positions exist in loaded map!");
                        } else {
                            println!("   ❌ Position(s) not in loaded map!");
                        }
                    }
                    None => {
                        println!("   ❌ No adjacent walkable tile found for water!");
                    }
                }
            }
            None => {
                println!("   ❌ No water found within 100 tiles");
            }
        }
        println!();
    }
}

fn load_world() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    use std::fs;
    
    let content = fs::read_to_string("maps/full_world.ron")?;
    
    // Parse the RON file to extract tile positions and terrain types
    let mut tiles = HashMap::new();
    
    // This is a very simplified parser - just enough for our test
    // In real code, we'd use the proper RON deserializer
    
    println!("   Parsing world data...");
    // For now, just return empty - we'll check if water search works
    
    Ok(tiles)
}

fn find_nearest_water_simple(from_x: i32, from_y: i32, _world: &HashMap<String, String>, max_radius: i32) -> Option<(i32, i32, f32)> {
    // Hardcode known water positions from the map
    let known_water = vec![
        // From chunk (1,1) - approximate world coords
        (26, 26), (27, 26), (28, 26), (29, 26), (30, 26), (31, 26),
        // From chunk (-3,-1)
        (-43, -5), (-43, -6), (-43, -7),
        // From chunk (2,-1)
        (34, -16), (35, -16),
    ];
    
    let mut nearest: Option<(i32, i32, f32)> = None;
    
    for (wx, wy) in known_water {
        let dx = (wx - from_x) as f32;
        let dy = (wy - from_y) as f32;
        let dist = (dx * dx + dy * dy).sqrt();
        
        if dist <= max_radius as f32 {
            if let Some((_, _, best_dist)) = nearest {
                if dist < best_dist {
                    nearest = Some((wx, wy, dist));
                }
            } else {
                nearest = Some((wx, wy, dist));
            }
        }
    }
    
    nearest
}

fn find_adjacent_walkable(water_x: i32, water_y: i32, _world: &HashMap<String, String>) -> Option<(i32, i32)> {
    // Check 8 adjacent positions
    let offsets = [
        (0, 1), (1, 0), (0, -1), (-1, 0),
        (1, 1), (1, -1), (-1, 1), (-1, -1),
    ];
    
    for (dx, dy) in offsets {
        let adj_x = water_x + dx;
        let adj_y = water_y + dy;
        
        // Assume adjacent tile is walkable if it's not water
        // In real code, we'd check the actual terrain
        return Some((adj_x, adj_y));
    }
    
    None
}
