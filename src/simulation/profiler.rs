//! Tick Performance Profiler
//!
//! Tracks timing for individual tick systems to identify performance bottlenecks.
//!
//! Usage:
//! ```rust
//! // Start timing a system
//! profiler::start_timing("ai_planner");
//!
//! // ... system logic here ...
//!
//! // End timing
//! profiler::end_timing("ai_planner");
//! ```

use bevy::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance timing data for a single system
#[derive(Debug, Clone)]
pub struct SystemTiming {
    pub total_duration: Duration,
    pub call_count: u64,
    pub last_duration: Duration,
    pub max_duration: Duration,
    pub min_duration: Duration,
}

impl SystemTiming {
    pub fn new() -> Self {
        Self {
            total_duration: Duration::ZERO,
            call_count: 0,
            last_duration: Duration::ZERO,
            max_duration: Duration::ZERO,
            min_duration: Duration::MAX,
        }
    }

    pub fn add_timing(&mut self, duration: Duration) {
        self.total_duration += duration;
        self.last_duration = duration;
        self.call_count += 1;
        self.max_duration = self.max_duration.max(duration);
        self.min_duration = self.min_duration.min(duration);
    }

    pub fn average_duration(&self) -> Duration {
        if self.call_count == 0 {
            Duration::ZERO
        } else {
            self.total_duration / self.call_count as u32
        }
    }
}

impl Default for SystemTiming {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource that tracks timing data for all systems
#[derive(Resource, Debug)]
pub struct TickProfiler {
    pub systems: HashMap<String, SystemTiming>,
    pub current_frame_start: Option<Instant>,
    pub active_timings: HashMap<String, Instant>,
    pub report_interval: u64,
    pub last_report_tick: u64,
}

impl TickProfiler {
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
            current_frame_start: None,
            active_timings: HashMap::new(),
            report_interval: 50, // Report every 50 ticks
            last_report_tick: 0,
        }
    }

    /// Start timing a named system
    pub fn start_timing(&mut self, system_name: &str) {
        let now = Instant::now();
        self.active_timings.insert(system_name.to_string(), now);

        // Track frame start if this is the first system
        if self.current_frame_start.is_none() {
            self.current_frame_start = Some(now);
        }
    }

    /// End timing a named system
    pub fn end_timing(&mut self, system_name: &str) {
        if let Some(start_time) = self.active_timings.remove(system_name) {
            let duration = start_time.elapsed();

            let timing = self
                .systems
                .entry(system_name.to_string())
                .or_insert_with(SystemTiming::new);

            timing.add_timing(duration);
        }
    }

    /// Generate a performance report for the current tick
    pub fn generate_report(&self, tick: u64) -> String {
        debug!(
            "🔧 Generating report for tick {} with {} systems",
            tick,
            self.systems.len()
        );

        if self.systems.is_empty() {
            return "🔧 No timing data available".to_string();
        }

        let total_duration: Duration = self.systems.values().map(|t| t.last_duration).sum();

        let mut report = format!(
            "🔧 TICK PERFORMANCE - Tick {} | Total: {:.1}ms\n",
            tick,
            total_duration.as_secs_f64() * 1000.0
        );

        // Sort systems by last duration (descending)
        let mut systems: Vec<_> = self.systems.iter().collect();
        systems.sort_by(|a, b| b.1.last_duration.cmp(&a.1.last_duration));

        for (name, timing) in systems {
            let percentage = if total_duration.is_zero() {
                0.0
            } else {
                (timing.last_duration.as_secs_f64() / total_duration.as_secs_f64()) * 100.0
            };

            report.push_str(&format!(
                "├── {:<15}: {:>6.1}ms ({:>3.0}%)\n",
                name,
                timing.last_duration.as_secs_f64() * 1000.0,
                percentage
            ));
        }

        // Add average timing info
        let avg_total: f64 = self
            .systems
            .values()
            .map(|t| t.average_duration().as_secs_f64())
            .sum();

        report.push_str(&format!(
            "└── AVG TOTAL: {:.1}ms over {} systems\n",
            avg_total * 1000.0,
            self.systems.len()
        ));

        report
    }

    /// Check if we should report this tick
    pub fn should_report(&self, tick: u64) -> bool {
        tick % self.report_interval == 0 && tick > self.last_report_tick
    }

    /// Reset timing data for next reporting period
    pub fn reset_period(&mut self) {
        self.systems.clear();
    }

    /// Start a new tick frame
    pub fn start_frame(&mut self) {
        self.current_frame_start = Some(Instant::now());
        self.active_timings.clear();
    }

    /// End current tick frame
    pub fn end_frame(&mut self) -> Duration {
        if let Some(start_time) = self.current_frame_start.take() {
            let frame_duration = start_time.elapsed();

            // End any dangling active timings
            for system_name in self.active_timings.keys().cloned().collect::<Vec<_>>() {
                warn!("🔧 Unclosed timing for system: {}", system_name);
                self.end_timing(&system_name);
            }

            frame_duration
        } else {
            Duration::ZERO
        }
    }
}

impl Default for TickProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to start timing a system
/// Note: This should only be used within Bevy systems that have access to ResMut<TickProfiler>
pub fn start_timing_resource(profiler: &mut TickProfiler, system_name: &str) {
    profiler.start_timing(system_name);
}

/// Convenience function to end timing a system
/// Note: This should only be used within Bevy systems that have access to ResMut<TickProfiler>
pub fn end_timing_resource(profiler: &mut TickProfiler, system_name: &str) {
    profiler.end_timing(system_name);
}

/// RAII helper for automatic timing using Bevy resources
pub struct ScopedTimer<'a> {
    system_name: String,
    profiler: &'a mut TickProfiler,
}

impl<'a> ScopedTimer<'a> {
    pub fn new(profiler: &'a mut TickProfiler, system_name: impl Into<String>) -> Self {
        let name = system_name.into();
        start_timing_resource(profiler, &name);
        Self {
            system_name: name,
            profiler,
        }
    }
}

impl<'a> Drop for ScopedTimer<'a> {
    fn drop(&mut self) {
        end_timing_resource(self.profiler, &self.system_name);
    }
}

/// Macro for easy timing of code blocks
#[macro_export]
macro_rules! time_system {
    ($name:expr, $code:block) => {{
        let _timer = $crate::simulation::profiler::ScopedTimer::new($name);
        $code
    }};
}

/// Plugin to install the profiler
pub struct TickProfilerPlugin;

impl Plugin for TickProfilerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TickProfiler>()
            .add_systems(Update, profiler_system.run_if(should_report_profiler));
    }
}

/// System that handles profiler reporting
fn profiler_system(
    mut profiler: ResMut<TickProfiler>,
    tick: Res<crate::simulation::SimulationTick>,
) {
    if profiler.should_report(tick.get()) {
        let report = profiler.generate_report(tick.get());
        info!("{}", report);
        profiler.reset_period();
        profiler.last_report_tick = tick.get();
    }
}

/// Run condition for profiler reporting
fn should_report_profiler(tick: Res<crate::simulation::SimulationTick>) -> bool {
    tick.get() % 50 == 0 // Report every 50 ticks
}
