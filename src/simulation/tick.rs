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
#[derive(Resource, Default, Debug)]
pub struct SimulationState {
    pub running: bool,
    pub started_at: Option<Instant>,
    pub total_ticks: u64,
}

impl SimulationState {
    pub fn start(&mut self) {
        self.running = true;
        self.started_at = Some(Instant::now());
    }
    
    pub fn stop(&mut self) {
        self.running = false;
    }
    
    pub fn uptime(&self) -> Option<Duration> {
        self.started_at.map(|start| start.elapsed())
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
/// Should run first in FixedUpdate schedule
pub fn increment_tick_counter(
    mut tick: ResMut<SimulationTick>,
    mut metrics: ResMut<TickMetrics>,
    mut state: ResMut<SimulationState>,
) {
    // End previous tick timing
    metrics.end_tick();
    
    // Start new tick timing
    metrics.start_tick();
    
    // Increment counter
    tick.increment();
    state.total_ticks += 1;
}

/// System that logs tick metrics periodically
pub fn log_tick_metrics(
    tick: Res<SimulationTick>,
    metrics: Res<TickMetrics>,
    speed: Res<SimulationSpeed>,
    state: Res<SimulationState>,
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
    info!("║                                          ║");
    info!("║ Total Ticks:     {:>8}               ║", state.total_ticks);
    
    if let Some(uptime) = state.uptime() {
        let uptime_secs = uptime.as_secs();
        let hours = uptime_secs / 3600;
        let minutes = (uptime_secs % 3600) / 60;
        let seconds = uptime_secs % 60;
        info!("║ Uptime:          {:02}:{:02}:{:02}               ║", hours, minutes, seconds);
    }
    
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
