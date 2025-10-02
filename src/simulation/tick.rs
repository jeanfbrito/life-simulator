/// Tick system resources and core functionality
use bevy::prelude::*;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

// ============================================================================
// RESOURCES
// ============================================================================

/// Current simulation tick counter
/// Increments every tick, never resets (except on save/load)
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct SimulationTick(pub u64);

impl SimulationTick {
    pub fn get(&self) -> u64 {
        self.0
    }
    
    pub fn increment(&mut self) {
        self.0 += 1;
    }
    
    pub fn set(&mut self, tick: u64) {
        self.0 = tick;
    }
}

/// Simulation speed and pause control
#[derive(Resource, Debug, Clone)]
pub struct SimulationSpeed {
    pub multiplier: f32,
    paused: bool,
}

impl Default for SimulationSpeed {
    fn default() -> Self {
        Self {
            multiplier: 1.0,
            paused: false,
        }
    }
}

impl SimulationSpeed {
    pub fn set_speed(&mut self, multiplier: f32) {
        self.multiplier = multiplier.max(0.1).min(10.0); // Clamp to reasonable range
    }
    
    pub fn pause(&mut self) {
        self.paused = true;
    }
    
    pub fn resume(&mut self) {
        self.paused = false;
    }
    
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
    
    pub fn is_paused(&self) -> bool {
        self.paused
    }
    
    pub fn effective_tps(&self, base_tps: f64) -> f64 {
        if self.paused {
            0.0
        } else {
            base_tps * self.multiplier as f64
        }
    }
}

/// Global simulation state
#[derive(Resource, Debug)]
pub struct SimulationState {
    pub should_tick: bool,
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            should_tick: false,
        }
    }
}


/// Performance metrics for tick monitoring
#[derive(Resource)]
pub struct TickMetrics {
    /// Last N tick durations (for averaging)
    tick_durations: VecDeque<Duration>,
    /// Maximum samples to keep
    max_samples: usize,
    /// Last tick start time
    last_tick_start: Option<Instant>,
    /// Current tick duration (ongoing)
    current_tick_start: Option<Instant>,
}

impl Default for TickMetrics {
    fn default() -> Self {
        Self {
            tick_durations: VecDeque::with_capacity(60),
            max_samples: 60, // Keep last 60 ticks (6 seconds at 10 TPS)
            last_tick_start: None,
            current_tick_start: None,
        }
    }
}

/// Accumulates frame time and determines when to run simulation ticks
/// Based on the world-simulator tick accumulator pattern
#[derive(Resource, Debug)]
pub struct TickAccumulator {
    /// Accumulated time since last tick
    pub accumulated: f32,
    /// Number of ticks that should run this frame
    pub pending_ticks: u32,
}

impl Default for TickAccumulator {
    fn default() -> Self {
        Self {
            accumulated: 0.0,
            pending_ticks: 0,
        }
    }
}

impl TickAccumulator {
    /// Update the accumulator with frame delta time
    /// Returns the number of ticks that should execute this frame
    pub fn update(&mut self, delta_seconds: f32, tick_duration: f32, speed_multiplier: f32) -> u32 {
        // Accumulate time based on speed multiplier
        self.accumulated += delta_seconds * speed_multiplier;

        // Calculate how many ticks to run
        let ticks = (self.accumulated / tick_duration) as u32;

        // Remove the consumed time
        self.accumulated -= ticks as f32 * tick_duration;

        // Cap accumulated time to prevent spiral of death
        if self.accumulated > tick_duration * 3.0 {
            self.accumulated = tick_duration * 3.0;
        }

        self.pending_ticks = ticks;
        ticks
    }

    pub fn should_tick(&self) -> bool {
        self.pending_ticks > 0
    }
}

impl TickMetrics {
    /// Start timing current tick
    pub fn start_tick(&mut self) {
        self.current_tick_start = Some(Instant::now());
    }
    
    /// End timing current tick and record duration
    pub fn end_tick(&mut self) {
        if let Some(start) = self.current_tick_start {
            let duration = start.elapsed();
            self.tick_durations.push_back(duration);
            
            // Keep only last N samples
            while self.tick_durations.len() > self.max_samples {
                self.tick_durations.pop_front();
            }
            
            self.last_tick_start = Some(start);
            self.current_tick_start = None;
        }
    }
    
    /// Get average tick duration over last N ticks
    pub fn average_duration(&self) -> Duration {
        if self.tick_durations.is_empty() {
            return Duration::ZERO;
        }
        
        let total: Duration = self.tick_durations.iter().sum();
        total / self.tick_durations.len() as u32
    }
    
    /// Get actual TPS based on measured tick durations
    pub fn actual_tps(&self) -> f64 {
        let avg = self.average_duration();
        if avg.is_zero() {
            0.0
        } else {
            1.0 / avg.as_secs_f64()
        }
    }
    
    /// Get minimum tick duration
    pub fn min_duration(&self) -> Option<Duration> {
        self.tick_durations.iter().min().copied()
    }
    
    /// Get maximum tick duration
    pub fn max_duration(&self) -> Option<Duration> {
        self.tick_durations.iter().max().copied()
    }
    
    /// Get last tick duration
    pub fn last_duration(&self) -> Option<Duration> {
        self.tick_durations.back().copied()
    }
}

// ============================================================================
// SYSTEMS
// ============================================================================

/// Core system that increments the tick counter
/// NOTE: This is no longer used - tick incrementing happens in run_simulation_ticks
pub fn increment_tick_counter(
    mut tick: ResMut<SimulationTick>,
    mut metrics: ResMut<TickMetrics>,
) {
    // End previous tick timing
    metrics.end_tick();
    
    // Start new tick timing
    metrics.start_tick();
    
    // Increment counter
    tick.increment();
}

/// System that logs tick metrics periodically
pub fn log_tick_metrics(
    tick: Res<SimulationTick>,
    metrics: Res<TickMetrics>,
    speed: Res<SimulationSpeed>,
) {
    let avg_duration = metrics.average_duration();
    let actual_tps = metrics.actual_tps();
    let min = metrics.min_duration().unwrap_or(Duration::ZERO);
    let max = metrics.max_duration().unwrap_or(Duration::ZERO);
    
    info!("╔══════════════════════════════════════════╗");
    info!("║       TICK METRICS - Tick {}           ║", tick.get());
    info!("╠══════════════════════════════════════════╣");
    info!("║ Actual TPS:      {:>6.1}                ║", actual_tps);
    info!("║ Speed:           {:>5.1}x                ║", speed.multiplier);
    info!("║ Status:          {:>9}             ║", if speed.is_paused() { "PAUSED" } else { "RUNNING" });
    info!("║                                          ║");
    info!("║ Tick Duration:                           ║");
    info!("║   Average:       {:>6.2}ms              ║", avg_duration.as_secs_f64() * 1000.0);
    info!("║   Min:           {:>6.2}ms              ║", min.as_secs_f64() * 1000.0);
    info!("║   Max:           {:>6.2}ms              ║", max.as_secs_f64() * 1000.0);
    info!("╚══════════════════════════════════════════╝");
}

// ============================================================================
// RUN CONDITIONS
// ============================================================================

/// Run condition: Execute system every N ticks
/// 
/// Usage:
/// ```rust
/// .add_systems(FixedUpdate, my_system.run_if(every_n_ticks(10)))
/// ```
pub fn every_n_ticks(n: u64) -> impl Fn(Res<SimulationTick>) -> bool + Clone {
    move |tick: Res<SimulationTick>| tick.0 % n == 0
}

/// Run condition: Execute only when simulation is not paused
pub fn when_not_paused(speed: Res<SimulationSpeed>) -> bool {
    !speed.is_paused()
}

/// Run condition: Execute only when simulation is paused
pub fn when_paused(speed: Res<SimulationSpeed>) -> bool {
    speed.is_paused()
}

/// Run condition: Execute only on specific tick
pub fn on_tick(target: u64) -> impl Fn(Res<SimulationTick>) -> bool + Clone {
    move |tick: Res<SimulationTick>| tick.0 == target
}

/// Run condition: Execute after a certain number of ticks have passed
pub fn after_tick(threshold: u64) -> impl Fn(Res<SimulationTick>) -> bool + Clone {
    move |tick: Res<SimulationTick>| tick.0 >= threshold
}

// ============================================================================
// HELPER TYPES
// ============================================================================

/// Update frequency for different system categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateFrequency {
    /// Every tick (10 TPS)
    EveryTick,
    /// Every N ticks
    EveryNTicks(u64),
    /// Rare updates (every 250 ticks = ~25 seconds)
    Rare,
    /// Very rare updates (every 1000 ticks = ~100 seconds)
    VeryRare,
}

impl UpdateFrequency {
    pub fn ticks(&self) -> u64 {
        match self {
            UpdateFrequency::EveryTick => 1,
            UpdateFrequency::EveryNTicks(n) => *n,
            UpdateFrequency::Rare => 250,
            UpdateFrequency::VeryRare => 1000,
        }
    }
    
    pub fn should_run(&self, tick: u64) -> bool {
        tick % self.ticks() == 0
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tick_increment() {
        let mut tick = SimulationTick::default();
        assert_eq!(tick.get(), 0);
        
        tick.increment();
        assert_eq!(tick.get(), 1);
        
        tick.increment();
        assert_eq!(tick.get(), 2);
    }
    
    #[test]
    fn test_speed_control() {
        let mut speed = SimulationSpeed::default();
        assert_eq!(speed.multiplier, 1.0);
        assert!(!speed.is_paused());
        
        speed.set_speed(2.0);
        assert_eq!(speed.multiplier, 2.0);
        
        speed.pause();
        assert!(speed.is_paused());
        
        speed.resume();
        assert!(!speed.is_paused());
        
        speed.toggle_pause();
        assert!(speed.is_paused());
    }
    
    #[test]
    fn test_update_frequency() {
        let every = UpdateFrequency::EveryTick;
        assert!(every.should_run(0));
        assert!(every.should_run(1));
        assert!(every.should_run(100));
        
        let every_5 = UpdateFrequency::EveryNTicks(5);
        assert!(every_5.should_run(0));
        assert!(!every_5.should_run(1));
        assert!(every_5.should_run(5));
        assert!(every_5.should_run(10));
        assert!(!every_5.should_run(11));
        
        let rare = UpdateFrequency::Rare;
        assert!(rare.should_run(0));
        assert!(!rare.should_run(100));
        assert!(rare.should_run(250));
        assert!(rare.should_run(500));
    }
    
    #[test]
    fn test_tick_metrics() {
        let mut metrics = TickMetrics::default();
        
        metrics.start_tick();
        std::thread::sleep(Duration::from_millis(1));
        metrics.end_tick();
        
        let last = metrics.last_duration();
        assert!(last.is_some());
        assert!(last.unwrap() >= Duration::from_millis(1));
        
        let avg = metrics.average_duration();
        assert!(avg > Duration::ZERO);
    }
}
