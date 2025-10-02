# Tick System - Quick Start Guide

## What We Built

✅ **Core tick system** inspired by Dwarf Fortress, Factorio, and RimWorld  
✅ **10 TPS base rate** (100ms per tick) - balanced for simulation  
✅ **Multi-rate updates** - Different systems run at different frequencies  
✅ **Pause/Speed controls** - Space to pause, 1-4 for speeds  
✅ **Performance metrics** - Built-in TPS monitoring  

## Files Created

```
src/simulation/
  mod.rs           # SimulationPlugin and speed controls
  tick.rs          # Core tick resources and systems
```

## Quick Integration

### 1. Add to `src/lib.rs`

```rust
pub mod simulation;
```

### 2. Update `src/main.rs`

```rust
use bevy::prelude::*;
use life_simulator::{
    simulation::SimulationPlugin,
    pathfinding::{PathfindingGrid, process_pathfinding_requests},
    entities::{EntitiesPlugin, movement::tick_movement_system},
};

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0)),
        ))
        
        // Core systems
        .add_plugins(SimulationPlugin)    // ← Adds tick system
        .add_plugins(EntitiesPlugin)
        
        // Resources
        .init_resource::<PathfindingGrid>()
        
        // Async systems (run every frame ~60fps)
        .add_systems(Update, (
            process_pathfinding_requests,
        ))
        
        // Tick systems (run on fixed timestep ~10 TPS)
        .add_systems(FixedUpdate, (
            // Fast systems (every tick)
            tick_movement_system,
            
            // Medium systems (conditional)
            // ai_decision_system.run_if(every_n_ticks(5)),
            
            // Slow systems (conditional)
            // plant_growth_system.run_if(every_n_ticks(100)),
        ).chain())
        
        .run();
}
```

## Keyboard Controls

- **Space**: Pause/Unpause simulation
- **1**: 0.5x speed (slow)
- **2**: 1.0x speed (normal)
- **3**: 2.0x speed (fast)
- **4**: 3.0x speed (ultra)

## Core Components

### SimulationTick
```rust
#[derive(Resource)]
pub struct SimulationTick(pub u64);

// Usage in systems:
fn my_system(tick: Res<SimulationTick>) {
    let current_tick = tick.0;
    info!("Tick: {}", current_tick);
}
```

### SimulationSpeed
```rust
#[derive(Resource)]
pub struct SimulationSpeed {
    pub multiplier: f32,  // 0.5, 1.0, 2.0, 3.0
    paused: bool,
}

// Methods:
speed.set_speed(2.0);
speed.pause();
speed.resume();
speed.toggle_pause();
```

### TickMetrics
```rust
#[derive(Resource)]
pub struct TickMetrics { /* ... */ }

// Usage:
fn monitor_system(metrics: Res<TickMetrics>) {
    let avg_duration = metrics.average_duration();
    let actual_tps = metrics.actual_tps();
}
```

## Run Conditions

### Every N Ticks
```rust
use life_simulator::simulation::every_n_ticks;

.add_systems(FixedUpdate, (
    ai_system.run_if(every_n_ticks(5)),      // Every 5 ticks
    needs_system.run_if(every_n_ticks(10)),   // Every 10 ticks
    growth_system.run_if(every_n_ticks(100)), // Every 100 ticks
))
```

### When Paused/Not Paused
```rust
use life_simulator::simulation::{when_paused, when_not_paused};

.add_systems(FixedUpdate, (
    simulation_system.run_if(when_not_paused),
    debug_inspect_system.run_if(when_paused),
))
```

### On Specific Tick
```rust
use life_simulator::simulation::{on_tick, after_tick};

.add_systems(FixedUpdate, (
    special_event.run_if(on_tick(1000)),
    unlock_feature.run_if(after_tick(500)),
))
```

## System Categories

### Fast (Every Tick - 10 TPS)
Movement, combat, physics - things that need immediate updates

```rust
.add_systems(FixedUpdate, (
    tick_movement_system,
    tick_combat_system,
    tick_physics_system,
))
```

### Medium (Every 5-10 Ticks - 1-2 TPS)
AI decisions, job assignments, needs

```rust
.add_systems(FixedUpdate, (
    ai_decisions.run_if(every_n_ticks(5)),
    job_assignment.run_if(every_n_ticks(10)),
    needs_update.run_if(every_n_ticks(10)),
))
```

### Slow (Every 100+ Ticks - 0.1 TPS)
Plant growth, aging, weather

```rust
.add_systems(FixedUpdate, (
    plant_growth.run_if(every_n_ticks(100)),
    aging_system.run_if(every_n_ticks(1000)),
))
```

## Performance Monitoring

### Automatic Logging

Every 100 ticks (10 seconds), you'll see:

```
╔══════════════════════════════════════════╗
║       TICK METRICS - Tick 100            ║
╠══════════════════════════════════════════╣
║ Actual TPS:        10.1                  ║
║ Speed:             1.0x                  ║
║ Status:            RUNNING               ║
║                                          ║
║ Tick Duration:                           ║
║   Average:          1.23ms               ║
║   Min:              0.98ms               ║
║   Max:              2.45ms               ║
║                                          ║
║ Total Ticks:           100               ║
║ Uptime:            00:00:10              ║
╚══════════════════════════════════════════╝
```

### Custom Monitoring

```rust
fn my_debug_system(
    tick: Res<SimulationTick>,
    metrics: Res<TickMetrics>,
) {
    if tick.0 % 50 == 0 {
        info!("Tick {}: {:.1} TPS, {:.2}ms avg",
              tick.0,
              metrics.actual_tps(),
              metrics.average_duration().as_secs_f64() * 1000.0);
    }
}
```

## Example: Complete System

```rust
use bevy::prelude::*;
use life_simulator::simulation::{SimulationTick, every_n_ticks};

/// Example AI system that runs every 5 ticks
fn creature_ai_system(
    tick: Res<SimulationTick>,
    mut query: Query<(&mut Creature, &TilePosition)>,
) {
    for (mut creature, pos) in query.iter_mut() {
        // AI logic here
        debug!("Creature {} thinking at tick {}", creature.name, tick.0);
        
        // Pick a random destination
        creature.target = Some(random_nearby_tile(pos.tile));
    }
}

// Add to app:
.add_systems(FixedUpdate, 
    creature_ai_system.run_if(every_n_ticks(5))
)
```

## Testing the System

### 1. Run the simulator

```bash
cargo run --bin life-simulator
```

### 2. Observe logs

You should see:
```
[INFO] Simulation RUNNING
[INFO] Speed: 1.0x (Normal)
... (tick updates) ...
[INFO] ╔══════════════════════════════════════════╗
[INFO] ║       TICK METRICS - Tick 100            ║
```

### 3. Test controls

- Press **Space** → Should see "Simulation PAUSED"
- Press **3** → Should see "Speed: 2.0x (Fast)"
- Press **Space** → Should see "Simulation RESUMED"

## Common Patterns

### Spawning Entity on Specific Tick

```rust
fn spawn_enemy_wave(
    mut commands: Commands,
    tick: Res<SimulationTick>,
) {
    if tick.0 % 500 == 0 {  // Every 500 ticks (~50 seconds)
        commands.spawn((
            Enemy { health: 100 },
            TilePosition::from_tile(IVec2::new(0, 0)),
        ));
        info!("Enemy wave spawned at tick {}", tick.0);
    }
}
```

### Conditional System Based on Game State

```rust
fn harvest_system(
    tick: Res<SimulationTick>,
    query: Query<&Plant>,
) {
    // Only run when there are plants to harvest
    if query.is_empty() {
        return;
    }
    
    for plant in query.iter() {
        if plant.is_mature() {
            // Harvest logic
        }
    }
}
```

### Multi-Phase Tick

```rust
fn complex_tick_system(
    tick: Res<SimulationTick>,
    mut world: ResMut<World>,
) {
    // Phase 1: Every tick
    update_positions(&mut world);
    
    // Phase 2: Every 5 ticks
    if tick.0 % 5 == 0 {
        update_ai(&mut world);
    }
    
    // Phase 3: Every 100 ticks
    if tick.0 % 100 == 0 {
        world_events(&mut world);
    }
}
```

## UpdateFrequency Helper

```rust
use life_simulator::simulation::tick::UpdateFrequency;

#[derive(Component)]
struct MyComponent {
    update_rate: UpdateFrequency,
}

fn smart_update_system(
    tick: Res<SimulationTick>,
    query: Query<(&MyComponent, &mut State)>,
) {
    for (component, mut state) in query.iter_mut() {
        if component.update_rate.should_run(tick.0) {
            // Update logic
        }
    }
}
```

## Performance Tips

1. **Use multi-rate updates**: Don't run expensive systems every tick
2. **Profile your systems**: Use tick metrics to find slow systems
3. **Batch operations**: Process multiple entities at once
4. **Use spatial partitioning**: Only process active chunks
5. **Spread work**: Use work budgets for heavy operations

## Troubleshooting

### Ticks Running Too Slow
- Check `TickMetrics` - average duration should be < 100ms
- Use `.run_if(every_n_ticks(N))` for expensive systems
- Profile systems to find bottlenecks

### Ticks Running Too Fast
- Normal! The system will maintain 10 TPS automatically
- If you want faster simulation, increase speed with keyboard

### Pause Not Working
- Check if `SimulationSpeed` is being modified elsewhere
- Verify `handle_speed_controls` system is running
- Check keyboard input is working

## Next Steps

1. ✅ Tick system working
2. ⬜ Add entities that move based on ticks
3. ⬜ Implement needs system (updates every 10 ticks)
4. ⬜ Add AI that thinks every 5 ticks
5. ⬜ Create plant growth (updates every 100 ticks)
6. ⬜ Add web API endpoint to show current tick
7. ⬜ Implement save/load with tick counter

## References

- **Analysis**: `docs/TICK_SYSTEM_ANALYSIS.md`
- **Code**: `src/simulation/mod.rs`, `src/simulation/tick.rs`
- **Bevy Fixed Timestep**: https://bevyengine.org/learn/book/getting-started/ecs/#fixed-timestep
