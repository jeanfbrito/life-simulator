use bevy::math::IVec2;
use life_simulator::pathfinding::{find_path, PathfindingGrid};
use life_simulator::tilemap::TerrainType;
use life_simulator::world_loader::WorldLoader;

#[test]
fn test_rabbit_can_path_to_water() {
    println!("🧪 Testing rabbit pathfinding to water");

    // Load the actual world
    println!("📂 Loading world...");
    let world_loader =
        WorldLoader::load_default().expect("Failed to load world - run map_generator first");

    println!("✅ World loaded: {}", world_loader.get_name());
    println!("   Chunks: {}", world_loader.get_chunk_count());

    // Build pathfinding grid (same as main.rs)
    println!("🗺️  Building pathfinding grid...");
    let mut pathfinding_grid = PathfindingGrid::new();

    let ((min_x, min_y), (max_x, max_y)) = world_loader.get_world_bounds();
    let tile_min_x = min_x * 16 - 16;
    let tile_min_y = min_y * 16 - 16;
    let tile_max_x = (max_x + 1) * 16 + 16;
    let tile_max_y = (max_y + 1) * 16 + 16;

    let mut tiles_processed = 0;
    let mut tiles_blocked = 0;
    let mut tiles_walkable = 0;

    for y in tile_min_y..=tile_max_y {
        for x in tile_min_x..=tile_max_x {
            let pos = IVec2::new(x, y);

            let terrain_str = world_loader.get_terrain_at(x, y);
            let terrain_cost = if let Some(terrain_str) = terrain_str {
                if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                    let cost = terrain.movement_cost();
                    if cost >= 1000.0 {
                        u32::MAX
                    } else {
                        tiles_walkable += 1;
                        cost as u32
                    }
                } else {
                    u32::MAX
                }
            } else {
                u32::MAX
            };

            let has_resource = world_loader
                .get_resource_at(x, y)
                .map(|r| !r.is_empty())
                .unwrap_or(false);

            let final_cost = if has_resource && terrain_cost != u32::MAX {
                tiles_blocked += 1;
                u32::MAX
            } else {
                terrain_cost
            };

            pathfinding_grid.set_cost(pos, final_cost);
            tiles_processed += 1;
        }
    }

    println!(
        "✅ Grid built: {} tiles processed, {} walkable, {} blocked by resources",
        tiles_processed, tiles_walkable, tiles_blocked
    );

    // Test typical rabbit spawn positions
    let test_cases = vec![
        ("Center spawn", IVec2::new(0, 0)),
        ("North spawn", IVec2::new(10, 15)),
        ("West spawn", IVec2::new(-15, 10)),
        ("East spawn", IVec2::new(18, -13)),
    ];

    for (name, spawn_pos) in test_cases {
        println!("\n🐇 Testing from {}: {:?}", name, spawn_pos);

        // Find nearest water (simplified search)
        let water_pos = find_nearest_shallow_water(spawn_pos, &world_loader, 100);

        match water_pos {
            Some((water_x, water_y)) => {
                let distance =
                    ((water_x - spawn_pos.x).pow(2) + (water_y - spawn_pos.y).pow(2)) as f32;
                let distance = distance.sqrt();
                println!(
                    "   💧 Found water at ({}, {}) - distance: {:.1} tiles",
                    water_x, water_y, distance
                );

                // Find adjacent walkable tile
                let adjacent =
                    find_adjacent_walkable_to_water(IVec2::new(water_x, water_y), &world_loader);

                match adjacent {
                    Some(adj_pos) => {
                        println!("   🚶 Adjacent walkable: {:?}", adj_pos);

                        // Try pathfinding
                        println!(
                            "   🧭 Finding path from {:?} to {:?}...",
                            spawn_pos, adj_pos
                        );
                        let path = find_path(
                            spawn_pos,
                            adj_pos,
                            &pathfinding_grid,
                            true, // Enable diagonal movement
                            Some(1000),
                        );

                        match path {
                            Some(p) => {
                                println!("   ✅ PATH FOUND! {} waypoints", p.all_waypoints().len());
                            }
                            None => {
                                println!("   ❌ PATH NOT FOUND!");
                                println!("      This explains why rabbits can't reach water!");

                                // Check if both tiles are walkable
                                let spawn_walkable = pathfinding_grid.is_walkable(spawn_pos);
                                let target_walkable = pathfinding_grid.is_walkable(adj_pos);
                                println!("      Spawn walkable: {}", spawn_walkable);
                                println!("      Target walkable: {}", target_walkable);

                                // Sample terrain along the straight line to see what's blocking
                                println!("\n      🔍 Sampling terrain along straight line:");
                                let dx = adj_pos.x - spawn_pos.x;
                                let dy = adj_pos.y - spawn_pos.y;
                                let distance = ((dx * dx + dy * dy) as f32).sqrt();
                                let steps = distance.ceil() as i32;

                                let mut impassable_count = 0;
                                for i in 0..=steps.min(20) {
                                    // Sample up to 20 points
                                    let t = if steps > 0 {
                                        i as f32 / steps as f32
                                    } else {
                                        0.0
                                    };
                                    let sample_x =
                                        (spawn_pos.x as f32 + dx as f32 * t).round() as i32;
                                    let sample_y =
                                        (spawn_pos.y as f32 + dy as f32 * t).round() as i32;
                                    let sample_pos = IVec2::new(sample_x, sample_y);

                                    if let Some(terrain_str) =
                                        world_loader.get_terrain_at(sample_x, sample_y)
                                    {
                                        let walkable = pathfinding_grid.is_walkable(sample_pos);
                                        if !walkable {
                                            impassable_count += 1;
                                            let resource_str = world_loader
                                                .get_resource_at(sample_x, sample_y)
                                                .unwrap_or_else(|| String::from("None"));
                                            println!("         ({:3}, {:3}): {} [BLOCKED] - Resource: {}", 
                                                sample_x, sample_y, terrain_str, resource_str);
                                        }
                                    }
                                }
                                println!(
                                    "      Impassable tiles found: {}/{}",
                                    impassable_count,
                                    steps.min(20) + 1
                                );
                            }
                        }
                    }
                    None => {
                        println!("   ❌ No adjacent walkable tile to water!");
                    }
                }
            }
            None => {
                println!("   ❌ No water found within 100 tiles!");
            }
        }
    }
}

fn find_nearest_shallow_water(
    from: IVec2,
    world_loader: &WorldLoader,
    max_radius: i32,
) -> Option<(i32, i32)> {
    let mut nearest: Option<(i32, i32, f32)> = None;

    for radius in 1..=max_radius {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if dx.abs() < radius && dy.abs() < radius {
                    continue;
                }

                let check_pos = from + IVec2::new(dx, dy);

                if let Some(terrain_str) = world_loader.get_terrain_at(check_pos.x, check_pos.y) {
                    if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                        if matches!(terrain, TerrainType::ShallowWater) {
                            let distance = from.as_vec2().distance(check_pos.as_vec2());

                            if let Some((_, _, best_dist)) = nearest {
                                if distance < best_dist {
                                    nearest = Some((check_pos.x, check_pos.y, distance));
                                }
                            } else {
                                nearest = Some((check_pos.x, check_pos.y, distance));
                            }
                        }
                    }
                }
            }
        }

        if nearest.is_some() {
            break;
        }
    }

    nearest.map(|(x, y, _)| (x, y))
}

fn find_adjacent_walkable_to_water(water_pos: IVec2, world_loader: &WorldLoader) -> Option<IVec2> {
    let adjacent_offsets = [
        IVec2::new(0, 1),
        IVec2::new(1, 0),
        IVec2::new(0, -1),
        IVec2::new(-1, 0),
        IVec2::new(1, 1),
        IVec2::new(1, -1),
        IVec2::new(-1, 1),
        IVec2::new(-1, -1),
    ];

    for offset in adjacent_offsets {
        let check_pos = water_pos + offset;

        if let Some(terrain_str) = world_loader.get_terrain_at(check_pos.x, check_pos.y) {
            if let Some(terrain) = TerrainType::from_str(&terrain_str) {
                if terrain.is_walkable()
                    && !matches!(
                        terrain,
                        TerrainType::ShallowWater | TerrainType::DeepWater | TerrainType::Water
                    )
                {
                    return Some(check_pos);
                }
            }
        }
    }

    None
}
