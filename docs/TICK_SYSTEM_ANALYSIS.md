# Tick System Architecture - Deep Analysis

## Overview

A comprehensive study of tick-based simulation architectures from successful games like Dwarf Fortress, Factorio, RimWorld, and others to design the heartbeat of our life simulator.

## What is a Tick-Based System?

A **tick** is a discrete time step where all game logic is executed synchronously. Unlike real-time systems that update continuously, tick-based systems process the world in atomic, reproducible chunks.

### Key Properties

1. **Deterministic**: Same inputs + same tick = same outputs (always)
2. **Atomic**: Each tick completes fully before the next begins
3. **Reproducible**: Can save/load at any tick and replay exactly
4. **Independent of FPS**: Rendering and logic are decoupled

## Case Studies

### 1. Dwarf Fortress (Tarn Adams)

**Architecture:**
- **72 ticks per day** in-game (configurable)
- **Variable real-time duration** per tick (can slow down with complexity)
- **Single-threaded** simulation with priority queues
- **Pause system** allows infinite inspection time

**Tick Structure:**
```
TICK N:
├── Temperature updates (every 10 ticks)
├── Weather calculations (every tick)
├── Creature actions (job system)
│   ├── Pathfinding (can span multiple ticks)
│   ├── Item hauling
│   └── Skill checks
├── Fluid simulation (every tick)
├── Plant growth (slow ticks - every 1000+)
└── World events
```

**Key Insights:**
- **Variable tick rates for different systems**
  - Fast: Combat, temperature, fluids (every tick)
  - Medium: Creature needs, jobs (every 10-20 ticks)
  - Slow: Plant growth, aging (every 1000+ ticks)
- **Budget-based execution**: Systems can run over multiple ticks if needed
- **Priority queuing**: Important events (combat) interrupt slower processing
- **Save on tick boundaries**: Perfect state serialization

### 2. Factorio (Wube Software)

**Architecture:**
- **Fixed 60 UPS** (Updates Per Second) = 60 ticks/second
- **Deterministic lockstep multiplayer**
- **Highly optimized single-threaded**
- **Chunk-based processing** for spatial optimization

**Tick Structure:**
```
TICK N (16.67ms budget):
├── Input processing
├── Entity updates (by chunk)
│   ├── Inserters (1 tick action)
│   ├── Assemblers (recipe progress)
│   ├── Belts (item movement)
│   └── Power grid (instant)
├── Circuit network evaluation
├── Train pathfinding (chunked over ticks)
├── Combat updates
└── Rendering preparation (separate thread)
```

**Key Insights:**
- **Everything is tick-aligned**: No sub-tick precision
- **Chunk activation**: Only update chunks with entities/players
- **Work spreading**: Heavy operations (pathfinding) spread over multiple ticks
- **Determinism first**: Every action must be reproducible
- **Performance budget**: 16.67ms max per tick for 60 UPS
- **Separate render thread**: Logic at 60 UPS, render at any FPS

### 3. RimWorld (Ludeon Studios)

**Architecture:**
- **60 ticks per second** at normal speed
- **3 speed settings**: Normal (60), Fast (120), Ultra (180)
- **Pause freely** for decision-making
- **Multi-threaded** pathfinding

**Tick Structure:**
```
TICK N:
├── Pawn (colonist) thinker
│   ├── Need evaluation (hunger, rest, joy)
│   ├── Job queue check
│   ├── Job execution (multi-tick jobs)
│   └── Social interactions
├── Thing (object) updates
│   ├── Temperature diffusion
│   ├── Plant growth ticks
│   └── Item deterioration
├── Map updates
│   ├── Weather
│   ├── Fire spread
│   └── Roof collapse checks
└── Asynchronous systems
    ├── Pathfinding (threaded)
    └── World map travel
```

**Key Insights:**
- **AI thinks every N ticks**: Pawns reconsider jobs every 30-60 ticks
- **Rare tickers**: Different update frequencies (every tick, rare, long)
  - `TickerType.Normal`: Every tick
  - `TickerType.Rare`: Every 250 ticks (~4 seconds)
  - `TickerType.Long`: Every 2000 ticks (~33 seconds)
- **Job system**: Long-running tasks broken into tick-sized chunks
- **Speed independence**: Logic works at any tick rate

### 4. Oxygen Not Included (Klei Entertainment)

**Architecture:**
- **5 cycles per second** (200ms per cycle) at normal speed
- **Sub-tick simulation** for fluids/gases
- **Work-stealing job system**
- **Spatial hashing** for entity queries

**Tick Structure:**
```
CYCLE (0.2s = 200ms):
├── Gas/Liquid simulation (sub-ticks)
├── Temperature propagation
├── Duplicant AI updates
│   ├── Breath oxygen consumption
│   ├── Bladder/hunger ticks
│   ├── Task priority evaluation
│   └── Movement execution
├── Building operations
│   ├── Power consumers/producers
│   ├── Gas/liquid pumps
│   └── Auto-sweepers
└── Germ spread calculations
```

**Key Insights:**
- **Slower base tick rate** (5 TPS) but sub-simulations run faster
- **Physical simulation** gets more budget than AI
- **Spatial partitioning**: Buildings only process relevant tiles
- **Interpolated rendering**: Smooth visuals between ticks

## Tick System Design Patterns

### Pattern 1: Fixed Time Step (Factorio-style)

```rust
const TICK_RATE: f64 = 60.0; // 60 TPS
const TICK_DURATION: Duration = Duration::from_secs_f64(1.0 / TICK_RATE);

fn main_loop() {
    let mut accumulator = Duration::ZERO;
    let mut last_time = Instant::now();
    
    loop {
        let current_time = Instant::now();
        let frame_time = current_time - last_time;
        last_time = current_time;
        
        accumulator += frame_time;
        
        // Process ticks
        while accumulator >= TICK_DURATION {
            tick_simulation(); // Always same duration
            accumulator -= TICK_DURATION;
        }
        
        // Render (interpolated)
        render(accumulator.as_secs_f64() / TICK_DURATION.as_secs_f64());
    }
}
```

**Pros:**
- Deterministic
- Multiplayer-friendly
- Predictable performance

**Cons:**
- Can spiral if tick takes > 16.67ms
- No dynamic slowdown

### Pattern 2: Variable Time Step (Dwarf Fortress-style)

```rust
const TARGET_TICK_RATE: f64 = 100.0; // Target 100 TPS

fn main_loop() {
    let mut last_time = Instant::now();
    
    loop {
        let current_time = Instant::now();
        let delta = current_time - last_time;
        last_time = current_time;
        
        // Tick takes however long it takes
        tick_simulation();
        
        // Display actual TPS
        let actual_tps = 1.0 / delta.as_secs_f64();
        println!("TPS: {:.1}", actual_tps);
        
        // Optional: Sleep to target rate if we're fast
        if delta < Duration::from_secs_f64(1.0 / TARGET_TICK_RATE) {
            std::thread::sleep(
                Duration::from_secs_f64(1.0 / TARGET_TICK_RATE) - delta
            );
        }
    }
}
```

**Pros:**
- Naturally handles complexity (slows down)
- Simple implementation
- User sees "real" performance

**Cons:**
- Non-deterministic
- Multiplayer difficult
- Time dilation under load

### Pattern 3: Hybrid (RimWorld-style)

```rust
const BASE_TICK_RATE: f64 = 60.0;

enum GameSpeed {
    Paused,
    Normal,    // 1x = 60 TPS
    Fast,      // 2x = 120 TPS
    Ultrafast, // 3x = 180 TPS
}

fn main_loop(speed: GameSpeed) {
    let tick_rate = match speed {
        GameSpeed::Paused => return,
        GameSpeed::Normal => BASE_TICK_RATE,
        GameSpeed::Fast => BASE_TICK_RATE * 2.0,
        GameSpeed::Ultrafast => BASE_TICK_RATE * 3.0,
    };
    
    let tick_duration = Duration::from_secs_f64(1.0 / tick_rate);
    let mut accumulator = Duration::ZERO;
    let mut last_time = Instant::now();
    
    loop {
        let current_time = Instant::now();
        let frame_time = current_time - last_time;
        last_time = current_time;
        
        accumulator += frame_time;
        
        // Process ticks at current speed
        while accumulator >= tick_duration {
            tick_simulation();
            accumulator -= tick_duration;
            
            // Safety: Don't spiral
            if accumulator > tick_duration * 5.0 {
                accumulator = Duration::ZERO; // Drop frames
            }
        }
        
        render();
    }
}
```

**Pros:**
- User controls speed
- Deterministic at each speed setting
- Pause for inspection

**Cons:**
- More complex state management
- Speed changes need careful handling

## Multi-Rate Update Systems

### Ticker System (RimWorld Pattern)

Different entities update at different rates:

```rust
#[derive(Debug, Clone, Copy)]
enum TickerType {
    Normal,      // Every tick (60/sec)
    Rare,        // Every 250 ticks (~4/sec)
    Long,        // Every 2000 ticks (~0.5/sec)
}

trait Tickable {
    fn ticker_type(&self) -> TickerType;
    fn tick(&mut self);
}

struct TickManager {
    normal_tickers: Vec<EntityId>,
    rare_tickers: Vec<EntityId>,
    long_tickers: Vec<EntityId>,
    current_tick: u64,
}

impl TickManager {
    fn tick(&mut self, world: &mut World) {
        // Always tick normal
        for &entity in &self.normal_tickers {
            world.get_mut(entity).tick();
        }
        
        // Rare every 250 ticks
        if self.current_tick % 250 == 0 {
            for &entity in &self.rare_tickers {
                world.get_mut(entity).tick();
            }
        }
        
        // Long every 2000 ticks
        if self.current_tick % 2000 == 0 {
            for &entity in &self.long_tickers {
                world.get_mut(entity).tick();
            }
        }
        
        self.current_tick += 1;
    }
}
```

### Budget-Based System (Factorio Pattern)

Spread heavy work across multiple ticks:

```rust
struct PathfindingBudget {
    max_nodes_per_tick: usize,
    pending_requests: VecDeque<PathRequest>,
}

impl PathfindingBudget {
    fn tick(&mut self) -> Vec<CompletedPath> {
        let mut nodes_processed = 0;
        let mut completed = Vec::new();
        
        while nodes_processed < self.max_nodes_per_tick {
            if let Some(request) = self.pending_requests.front_mut() {
                // Process some nodes
                let result = request.process_nodes(
                    self.max_nodes_per_tick - nodes_processed
                );
                
                nodes_processed += result.nodes_processed;
                
                if result.completed {
                    completed.push(self.pending_requests.pop_front().unwrap());
                }
            } else {
                break;
            }
        }
        
        completed
    }
}
```

## Recommended Architecture for Life Simulator

### Core Design: Hybrid Multi-Rate System

Combine the best aspects of all approaches:

```rust
// Configuration
const BASE_TICK_RATE: f64 = 10.0;  // 10 TPS base (100ms per tick)
const FRAME_RATE: f64 = 60.0;       // 60 FPS rendering

// Tick rates for different systems
pub enum UpdateRate {
    EveryTick,           // Movement, combat (10 TPS)
    EveryNTicks(u32),    // Jobs, AI (every 5 ticks = 2 TPS)
    EveryNSeconds(f32),  // Plant growth (every 10 sec = 0.1 TPS)
}

pub struct SimulationConfig {
    pub base_tps: f64,
    pub speed_multiplier: f32,  // 1.0 = normal, 2.0 = 2x, 0.0 = paused
    pub max_tick_duration: Duration, // Safety limit
}

pub struct TickScheduler {
    current_tick: u64,
    tick_budget: Duration,
    systems: Vec<(UpdateRate, Box<dyn TickSystem>)>,
}

trait TickSystem {
    fn tick(&mut self, world: &mut World, tick: u64);
    fn name(&self) -> &str;
}
```

### System Categories

#### Fast Systems (Every Tick - 10 TPS)
```rust
// src/simulation/fast_systems.rs

/// Movement execution - entities advance along paths
pub fn tick_movement_system(/* ... */) { }

/// Combat resolution - attacks, damage
pub fn tick_combat_system(/* ... */) { }

/// Immediate interactions - picking up items, opening doors
pub fn tick_interaction_system(/* ... */) { }

/// Physics - falling objects, projectiles
pub fn tick_physics_system(/* ... */) { }
```

#### Medium Systems (Every 5-10 Ticks - 1-2 TPS)
```rust
// src/simulation/medium_systems.rs

/// AI decision making - choose new goals
pub fn tick_ai_decisions(/* ... */) { }

/// Job assignment - find work for idle entities
pub fn tick_job_assignment(/* ... */) { }

/// Need updates - hunger, thirst, fatigue
pub fn tick_needs_system(/* ... */) { }

/// Social interactions - conversations, relationships
pub fn tick_social_system(/* ... */) { }
```

#### Slow Systems (Every 100+ Ticks - 0.1 TPS)
```rust
// src/simulation/slow_systems.rs

/// Plant growth - trees, crops
pub fn tick_plant_growth(/* ... */) { }

/// Aging - creatures get older
pub fn tick_aging_system(/* ... */) { }

/// Weather - seasonal changes
pub fn tick_weather_system(/* ... */) { }

/// World events - migrations, disasters
pub fn tick_world_events(/* ... */) { }
```

#### Async Systems (Not Tick-Bound)
```rust
// Run continuously, results consumed on ticks

/// Pathfinding calculation
pub fn async_pathfinding_system(/* ... */) { }

/// Terrain generation for new chunks
pub fn async_chunk_generation(/* ... */) { }

/// Save/load operations
pub fn async_persistence(/* ... */) { }
```

### Complete Tick Structure

```rust
// src/simulation/tick.rs

pub struct Simulation {
    world: World,
    scheduler: TickScheduler,
    config: SimulationConfig,
    tick_counter: u64,
    paused: bool,
}

impl Simulation {
    pub fn tick(&mut self) {
        if self.paused {
            return;
        }
        
        let tick_start = Instant::now();
        
        // Phase 1: Input/Events (pre-tick)
        self.process_events();
        
        // Phase 2: Fast systems (every tick)
        self.tick_movement();
        self.tick_combat();
        self.tick_interactions();
        self.tick_physics();
        
        // Phase 3: Medium systems (conditional)
        if self.tick_counter % 5 == 0 {
            self.tick_ai_decisions();
            self.tick_job_assignment();
            self.tick_needs();
        }
        
        if self.tick_counter % 10 == 0 {
            self.tick_social();
        }
        
        // Phase 4: Slow systems (conditional)
        if self.tick_counter % 100 == 0 {
            self.tick_plant_growth();
        }
        
        if self.tick_counter % 1000 == 0 {
            self.tick_aging();
            self.tick_weather();
        }
        
        // Phase 5: Async result collection
        self.collect_completed_paths();
        self.collect_loaded_chunks();
        
        // Phase 6: Post-tick cleanup
        self.cleanup_dead_entities();
        self.update_cached_data();
        
        self.tick_counter += 1;
        
        // Performance monitoring
        let tick_duration = tick_start.elapsed();
        if tick_duration > self.config.max_tick_duration {
            warn!("Tick {} took {:?} (limit: {:?})", 
                  self.tick_counter, 
                  tick_duration, 
                  self.config.max_tick_duration);
        }
    }
    
    pub fn set_speed(&mut self, multiplier: f32) {
        self.config.speed_multiplier = multiplier;
    }
    
    pub fn pause(&mut self) {
        self.paused = true;
    }
    
    pub fn resume(&mut self) {
        self.paused = false;
    }
}
```

## Main Loop Integration with Bevy

```rust
// src/main.rs

use bevy::prelude::*;
use std::time::Duration;

const TARGET_TPS: f64 = 10.0;  // 10 ticks per second
const TICK_DURATION: Duration = Duration::from_millis(100);

fn main() {
    App::new()
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
            Duration::from_secs_f64(1.0 / 60.0) // 60 FPS frame rate
        )))
        
        // Resources
        .insert_resource(SimulationTick::default())
        .insert_resource(SimulationSpeed::default())
        .insert_resource(Time::<Fixed>::from_duration(TICK_DURATION))
        
        // Async systems (run every frame)
        .add_systems(Update, (
            process_pathfinding_requests,  // Pathfinding calculation
            handle_user_input,              // Input processing
            update_web_clients,             // Network updates
        ))
        
        // Tick systems (run on fixed timestep)
        .add_systems(FixedUpdate, (
            // Fast systems (every tick)
            tick_movement_system,
            tick_combat_system,
            tick_interactions_system,
            
            // Medium systems (conditional)
            tick_ai_system.run_if(every_n_ticks(5)),
            tick_needs_system.run_if(every_n_ticks(5)),
            
            // Slow systems (conditional)
            tick_plant_growth.run_if(every_n_ticks(100)),
            
            // Always run
            increment_tick_counter,
        ).chain())
        
        .run();
}

// Run condition helper
fn every_n_ticks(n: u64) -> impl Fn(Res<SimulationTick>) -> bool {
    move |tick: Res<SimulationTick>| tick.0 % n == 0
}

#[derive(Resource, Default)]
pub struct SimulationTick(pub u64);

fn increment_tick_counter(mut tick: ResMut<SimulationTick>) {
    tick.0 += 1;
}

#[derive(Resource)]
pub struct SimulationSpeed {
    pub multiplier: f32,
    pub paused: bool,
}

impl Default for SimulationSpeed {
    fn default() -> Self {
        Self {
            multiplier: 1.0,
            paused: false,
        }
    }
}
```

## Performance Optimization Strategies

### 1. Spatial Partitioning (Factorio-style)

Only process active chunks:

```rust
pub struct ActiveChunkSystem {
    active_chunks: HashSet<ChunkCoord>,
    chunk_entities: HashMap<ChunkCoord, Vec<Entity>>,
}

impl ActiveChunkSystem {
    pub fn tick(&mut self, world: &World) {
        for &chunk_coord in &self.active_chunks {
            if let Some(entities) = self.chunk_entities.get(&chunk_coord) {
                for &entity in entities {
                    // Process only entities in active chunks
                    world.process_entity(entity);
                }
            }
        }
    }
}
```

### 2. Dirty Flags

Only update what changed:

```rust
#[derive(Component)]
pub struct Dirty {
    position_changed: bool,
    state_changed: bool,
}

pub fn tick_system(query: Query<(&Transform, &mut Dirty)>) {
    for (transform, mut dirty) in query.iter_mut() {
        if dirty.position_changed {
            // Recalculate spatial hash
            dirty.position_changed = false;
        }
    }
}
```

### 3. Work Budgets

Spread expensive operations:

```rust
pub struct WorkBudget {
    max_items_per_tick: usize,
    pending_work: VecDeque<Work>,
}

impl WorkBudget {
    pub fn tick(&mut self) {
        let mut processed = 0;
        while processed < self.max_items_per_tick {
            if let Some(work) = self.pending_work.pop_front() {
                work.execute();
                processed += 1;
            } else {
                break;
            }
        }
    }
}
```

## Tick Debugging & Monitoring

```rust
#[derive(Resource)]
pub struct TickMetrics {
    pub tick_durations: VecDeque<Duration>,
    pub system_times: HashMap<String, Duration>,
    pub tick_count: u64,
    pub average_tps: f64,
}

impl TickMetrics {
    pub fn record_tick(&mut self, duration: Duration, systems: HashMap<String, Duration>) {
        self.tick_durations.push_back(duration);
        if self.tick_durations.len() > 60 {
            self.tick_durations.pop_front();
        }
        
        self.system_times = systems;
        self.tick_count += 1;
        
        // Calculate average TPS over last 60 ticks
        let total: Duration = self.tick_durations.iter().sum();
        self.average_tps = self.tick_durations.len() as f64 / total.as_secs_f64();
    }
    
    pub fn report(&self) {
        info!("=== Tick {} ===", self.tick_count);
        info!("TPS: {:.1}", self.average_tps);
        info!("System times:");
        for (system, duration) in &self.system_times {
            info!("  {}: {:?}", system, duration);
        }
    }
}
```

## Recommended Starting Configuration

```rust
// For life simulator - balanced approach

pub fn default_simulation_config() -> SimulationConfig {
    SimulationConfig {
        base_tps: 10.0,              // 10 ticks/sec = 100ms per tick
        speed_multiplier: 1.0,        // Normal speed
        max_tick_duration: Duration::from_millis(200), // 2x budget
        
        // Update rates
        movement_rate: UpdateRate::EveryTick,
        ai_rate: UpdateRate::EveryNTicks(5),      // 2 TPS
        needs_rate: UpdateRate::EveryNTicks(10),  // 1 TPS
        growth_rate: UpdateRate::EveryNTicks(100), // 0.1 TPS
        
        // Performance
        max_pathfinding_per_tick: 10,
        max_ai_thinks_per_tick: 50,
        active_chunk_radius: 3,
    }
}
```

## Summary & Recommendations

### Choose This Architecture:
1. **Base rate**: 10 TPS (100ms per tick) - good balance
2. **Multi-rate**: Different systems at different frequencies
3. **Hybrid timing**: Fixed timestep with pause/speed controls
4. **Async pathfinding**: Calculations off-tick, results consumed on-tick
5. **Chunk activation**: Only process relevant areas
6. **Work budgets**: Spread heavy operations

### Don't:
- ❌ Try to tick at 60 TPS initially (too fast for simulation)
- ❌ Put everything in one tick (use multi-rate)
- ❌ Block ticks for expensive operations (use budgets)
- ❌ Process entire world every tick (use spatial partitioning)

### Do:
- ✅ Start at 10 TPS, measure, adjust
- ✅ Use Bevy's FixedUpdate for deterministic ticks
- ✅ Keep rendering at 60 FPS (separate from logic)
- ✅ Add pause/speed controls from day 1
- ✅ Instrument tick performance early
- ✅ Use condition systems for multi-rate updates

## Next Steps

1. **Implement core tick loop** with Bevy FixedUpdate
2. **Add SimulationTick resource** for tracking
3. **Convert movement system** to run on ticks
4. **Add tick metrics** and logging
5. **Test with simple entities** moving around
6. **Add speed controls** (pause, 1x, 2x, 3x)
7. **Optimize** based on actual performance data

---

**References:**
- Dwarf Fortress Development: http://www.bay12games.com/dwarves/dev.html
- Factorio FFF (Friday Facts): https://factorio.com/blog/
- RimWorld Modding: https://rimworldwiki.com/wiki/Modding_Tutorials
- Game Programming Patterns: http://gameprogrammingpatterns.com/game-loop.html
