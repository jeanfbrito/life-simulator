#![allow(unused_imports, unused_mut)]
use bevy::math::IVec2;
use life_simulator::vegetation::{constants, ResourceGrid};
use tracing_subscriber::EnvFilter;

fn main() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .try_init();

    let mut grid = ResourceGrid::new();

    // Seed a small world of chunks with partial biomass so they need regrowth.
    for chunk_x in -2..=2 {
        for chunk_y in -2..=2 {
            for local_x in 0..constants::performance::CHUNK_SIZE as i32 {
                for local_y in 0..constants::performance::CHUNK_SIZE as i32 {
                    let world_x = chunk_x * constants::performance::CHUNK_SIZE as i32 + local_x;
                    let world_y = chunk_y * constants::performance::CHUNK_SIZE as i32 + local_y;
                    let tile = IVec2::new(world_x, world_y);
                    if let Ok(mut veg) = grid.get_or_create_cell(tile, 100.0, 1.0) {
                        // Start most tiles partially depleted so the queue has work to do.
                        if veg.total_biomass > 20.0 {
                            veg.total_biomass = 20.0;
                        }
                    }
                }
            }
        }
    }

    println!("tick,active_cells,pending_events,events_processed,processing_time_us");

    for tick in 0..400u64 {
        // Simulate a grazing event midway through to re-enqueue chunks.
        if tick == 150 {
            let tile = IVec2::new(0, 0);
            let _ = grid.consume_at(tile, 40.0, 1.0); // max_fraction = 1.0 for ResourceGrid
        }

        grid.update(tick);
        let metrics = grid.get_metrics();
        println!(
            "{tick},{},{},{},{}",
            metrics.active_cells,
            grid.pending_events(),
            metrics.events_processed,
            metrics.processing_time_us
        );
    }
}
