/// Sparse, event-driven resource grid for vegetation system
///
/// This module implements Phase 1 of the vegetation rewrite plan, replacing the
/// dense tile-by-tile updates with a sparse hash grid that only stores cells
/// with biomass and processes them through events.
use bevy::prelude::*;
use rand;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use crate::resources::{ResourceType, HarvestProfile, RESOURCE_DEFINITIONS};
use crate::errors::{LifeSimulatorError, Result};

/// Current simulation tick for event timing
pub type CurrentTick = u64;

/// Sparse grid storage for vegetation cells
/// Only stores cells that contain biomass, not empty tiles
#[derive(Resource, Debug, Clone)]
pub struct ResourceGrid {
    /// Sparse storage: world coordinates -> grazing cell data
    cells: HashMap<IVec2, GrazingCell>,

    /// Event scheduler for regrowth and consumption events
    event_scheduler: VegetationScheduler,

    /// Current simulation tick
    current_tick: CurrentTick,

    /// Performance metrics
    metrics: ResourceGridMetrics,
}

/// A single vegetation cell that can be grazed by herbivores
#[derive(Debug, Clone)]
pub struct GrazingCell {
    /// Resource type at this location
    pub resource_type: Option<ResourceType>,

    /// Total biomass available in this cell (0.0 to max_biomass)
    pub total_biomass: f32,

    /// Consumption pressure from herbivores (0.0 to 1.0)
    /// Higher values mean more frequent grazing
    pub consumption_pressure: f32,

    /// Last tick when this cell was updated
    pub last_update_tick: u64,

    /// Maximum biomass this cell can support (based on resource profile)
    pub max_biomass: f32,

    /// Growth rate modifier (from resource profile)
    pub growth_rate_modifier: f32,

    /// Tick when this cell can regrow after harvest (for collectables)
    pub regrowth_available_tick: u64,
}

impl GrazingCell {
    /// Create a new grazing cell with resource type and initial biomass
    pub fn new(
        resource_type: Option<ResourceType>,
        initial_biomass: f32,
        max_biomass: f32,
        growth_rate_modifier: f32,
        current_tick: u64,
    ) -> Self {
        Self {
            resource_type,
            total_biomass: initial_biomass.clamp(0.0, max_biomass),
            consumption_pressure: 0.0,
            last_update_tick: current_tick,
            max_biomass,
            growth_rate_modifier,
            regrowth_available_tick: current_tick,
        }
    }

    /// Create a new grazing cell from a resource type
    pub fn from_resource_type(resource_type: ResourceType, current_tick: u64) -> Option<Self> {
        let profile = RESOURCE_DEFINITIONS.get(&resource_type)?;
        Some(Self::new(
            Some(resource_type),
            profile.biomass_cap * 0.3, // Start at 30% of max biomass
            profile.biomass_cap,
            profile.growth_rate_multiplier,
            current_tick,
        ))
    }

    /// Get the harvest profile for this cell's resource type
    pub fn get_profile(&self) -> Option<&'static HarvestProfile> {
        self.resource_type.as_ref().and_then(|rt| RESOURCE_DEFINITIONS.get(rt))
    }

    /// Check if this cell is available for consumption based on regrowth delay
    pub fn is_available_for_consumption(&self, current_tick: u64) -> bool {
        current_tick >= self.regrowth_available_tick
    }

    /// Set regrowth delay after consumption
    pub fn apply_regrowth_delay(&mut self, current_tick: u64) {
        if let Some(profile) = self.get_profile() {
            self.regrowth_available_tick = current_tick + profile.regrowth_delay_ticks;
        }
    }

    /// Check if this cell is depleted (below minimum forageable biomass)
    pub fn is_depleted(&self) -> bool {
        self.total_biomass < 10.0 // FORAGE_MIN_BIOMASS from old system
    }

    /// Get current biomass as fraction of maximum
    pub fn biomass_fraction(&self) -> f32 {
        if self.max_biomass > 0.0 {
            self.total_biomass / self.max_biomass
        } else {
            0.0
        }
    }

    /// Apply regrowth to this cell based on logistic growth equation
    /// Returns the actual amount of biomass added
    pub fn apply_regrowth(&mut self, delta_ticks: u64, current_tick: u64) -> f32 {
        // Check if regrowth is available
        if !self.is_available_for_consumption(current_tick) {
            return 0.0;
        }

        if self.total_biomass >= self.max_biomass {
            return 0.0;
        }

        // Convert delta_ticks to time factor (assuming 10 TPS)
        let time_factor = (delta_ticks as f32) / 10.0;

        // Logistic growth: B(t+1) = B(t) + r * B(t) * (1 - B(t)/Bmax)
        // Use growth rate from resource profile
        let base_growth_rate = 0.05 * self.growth_rate_modifier; // GROWTH_RATE from old system
        let growth =
            base_growth_rate * self.total_biomass * (1.0 - self.total_biomass / self.max_biomass);
        let actual_growth = (growth * time_factor).min(self.max_biomass - self.total_biomass);

        self.total_biomass += actual_growth;
        self.last_update_tick = current_tick;
        actual_growth
    }

    /// Consume biomass from this cell
    /// Returns actual amount consumed
    pub fn consume_biomass(&mut self, requested: f32, max_fraction: f32, current_tick: u64) -> f32 {
        // Check if consumption is allowed
        if !self.is_available_for_consumption(current_tick) {
            return 0.0;
        }

        // Apply consumption rules: min(requested, max_fraction * available)
        let max_by_fraction = self.total_biomass * max_fraction;
        let actual_consumed = requested.min(max_by_fraction);

        if actual_consumed > 0.0 {
            self.total_biomass -= actual_consumed;
            self.consumption_pressure = (self.consumption_pressure + actual_consumed).min(1.0);
            self.last_update_tick = current_tick;

            // Apply regrowth delay for collectable resources
            if let Some(profile) = self.get_profile() {
                match profile.consumption_kind {
                    crate::resources::ConsumptionKind::HumanGather => {
                        self.apply_regrowth_delay(current_tick);
                    }
                    _ => {} // No delay for herbivore browsing
                }
            }
        }

        actual_consumed
    }

    /// Decay consumption pressure over time
    pub fn decay_pressure(&mut self, delta_ticks: u64) {
        let decay_factor = (delta_ticks as f32) / 100.0; // Full decay over 10 seconds
        self.consumption_pressure = (self.consumption_pressure - decay_factor).max(0.0);
    }
}

/// Events that drive vegetation updates
#[derive(Debug, Clone)]
pub enum GrowthEvent {
    /// Consume biomass at a location and schedule regrowth
    Consume {
        location: IVec2,
        amount_consumed: f32,
        scheduled_tick: u64,
    },

    /// Regrowth event for a specific cell
    Regrow {
        location: IVec2,
        scheduled_tick: u64,
    },

    /// Random sampling for ambient regrowth
    RandomSample {
        locations: Vec<IVec2>,
        scheduled_tick: u64,
    },
}

impl PartialEq for GrowthEvent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                GrowthEvent::Consume {
                    location: l1,
                    amount_consumed: a1,
                    scheduled_tick: s1,
                },
                GrowthEvent::Consume {
                    location: l2,
                    amount_consumed: a2,
                    scheduled_tick: s2,
                },
            ) => l1 == l2 && (a1 - a2).abs() < f32::EPSILON && s1 == s2,
            (
                GrowthEvent::Regrow {
                    location: l1,
                    scheduled_tick: s1,
                },
                GrowthEvent::Regrow {
                    location: l2,
                    scheduled_tick: s2,
                },
            ) => l1 == l2 && s1 == s2,
            (
                GrowthEvent::RandomSample {
                    locations: l1,
                    scheduled_tick: s1,
                },
                GrowthEvent::RandomSample {
                    locations: l2,
                    scheduled_tick: s2,
                },
            ) => l1 == l2 && s1 == s2,
            _ => false,
        }
    }
}

impl Eq for GrowthEvent {}

impl GrowthEvent {
    /// Get the scheduled execution tick for this event
    pub fn scheduled_tick(&self) -> u64 {
        match self {
            GrowthEvent::Consume { scheduled_tick, .. } => *scheduled_tick,
            GrowthEvent::Regrow { scheduled_tick, .. } => *scheduled_tick,
            GrowthEvent::RandomSample { scheduled_tick, .. } => *scheduled_tick,
        }
    }

    /// Get the location(s) this event affects
    pub fn locations(&self) -> Vec<IVec2> {
        match self {
            GrowthEvent::Consume { location, .. } => vec![*location],
            GrowthEvent::Regrow { location, .. } => vec![*location],
            GrowthEvent::RandomSample { locations, .. } => locations.clone(),
        }
    }
}

/// Event scheduler using a priority queue (min-heap) keyed by tick
#[derive(Debug, Clone)]
pub struct VegetationScheduler {
    /// Priority queue of events sorted by execution time
    event_queue: BinaryHeap<Reverse<ScheduledEvent>>,

    /// Random tick budget for ambient updates
    random_tick_budget: usize,
}

/// Event with execution time for queue ordering
#[derive(Debug, Clone, Eq, PartialEq)]
struct ScheduledEvent {
    event: GrowthEvent,
    scheduled_tick: u64,
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.scheduled_tick
            .cmp(&other.scheduled_tick)
            .then_with(|| {
                self.event
                    .locations()
                    .len()
                    .cmp(&other.event.locations().len())
            })
    }
}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl VegetationScheduler {
    /// Create new scheduler with default budget
    pub fn new() -> Self {
        Self {
            event_queue: BinaryHeap::new(),
            random_tick_budget: 50, // Process up to 50 random cells per tick
        }
    }

    /// Schedule an event to be executed at a specific tick
    pub fn schedule(&mut self, event: GrowthEvent) {
        let scheduled_event = ScheduledEvent {
            scheduled_tick: event.scheduled_tick(),
            event,
        };
        self.event_queue.push(Reverse(scheduled_event));
    }

    /// Get all events due for execution at or before current tick
    pub fn pop_due_events(&mut self, current_tick: u64) -> Vec<GrowthEvent> {
        let mut due_events = Vec::new();

        while let Some(Reverse(scheduled_event)) = self.event_queue.peek() {
            if scheduled_event.scheduled_tick > current_tick {
                break;
            }

            // Safe to unwrap here because we just peeked and confirmed it exists
            let Reverse(scheduled_event) = self.event_queue.pop()
                .expect("Event queue peek succeeded but pop failed");
            due_events.push(scheduled_event.event);
        }

        due_events
    }

    /// Get number of pending events
    pub fn pending_count(&self) -> usize {
        self.event_queue.len()
    }

    /// Set random tick budget
    pub fn set_random_tick_budget(&mut self, budget: usize) {
        self.random_tick_budget = budget;
    }

    /// Get random tick budget
    pub fn random_tick_budget(&self) -> usize {
        self.random_tick_budget
    }
}

/// Performance metrics for the resource grid
#[derive(Debug, Clone, Default)]
pub struct ResourceGridMetrics {
    /// Total number of cells with biomass
    pub active_cells: usize,

    /// Number of events processed in last update
    pub events_processed: usize,

    /// Number of random cells sampled in last update
    pub random_cells_sampled: usize,

    /// Total biomass added in last update
    pub biomass_grown: f32,

    /// Total biomass consumed in last update
    pub biomass_consumed: f32,

    /// CPU time spent processing events (microseconds)
    pub processing_time_us: u64,
}

impl ResourceGrid {
    /// Create a new empty resource grid
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            event_scheduler: VegetationScheduler::new(),
            current_tick: 0,
            metrics: ResourceGridMetrics::default(),
        }
    }

    /// Get the current simulation tick
    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    /// Get the number of cells with biomass
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Get the number of pending events
    pub fn pending_events(&self) -> usize {
        self.event_scheduler.pending_count()
    }

    /// Get a reference to a cell at the given position
    pub fn get_cell(&self, pos: IVec2) -> Option<&GrazingCell> {
        self.cells.get(&pos)
    }

    /// Get a mutable reference to a cell at the given position
    pub fn get_cell_mut(&mut self, pos: IVec2) -> Option<&mut GrazingCell> {
        self.cells.get_mut(&pos)
    }

    /// Get or create a cell at the given position
    pub fn get_or_create_cell(
        &mut self,
        pos: IVec2,
        max_biomass: f32,
        growth_modifier: f32,
    ) -> Result<&mut GrazingCell> {
        if !self.cells.contains_key(&pos) {
            let initial_biomass = 5.0_f32.min(max_biomass); // INITIAL_BIOMASS from old system
            let cell = GrazingCell::new(
                None, // No resource type specified for backward compatibility
                initial_biomass,
                max_biomass,
                growth_modifier,
                self.current_tick,
            );
            self.cells.insert(pos, cell);
            self.metrics.active_cells = self.cells.len();
        }
        
        self.cells.get_mut(&pos)
            .ok_or_else(|| LifeSimulatorError::resource_grid(
                format!("Failed to get or create cell at position {:?}", pos)
            ))
    }

    /// Get or create a cell with a specific resource type
    pub fn get_or_create_cell_with_resource(
        &mut self,
        pos: IVec2,
        resource_type: ResourceType,
    ) -> Option<&mut GrazingCell> {
        if !self.cells.contains_key(&pos) {
            let cell = GrazingCell::from_resource_type(resource_type, self.current_tick)?;
            self.cells.insert(pos, cell);
            self.metrics.active_cells = self.cells.len();
        }
        self.cells.get_mut(&pos)
    }

    /// Apply a resource profile to an existing cell
    pub fn apply_profile(&mut self, pos: IVec2, resource_type: ResourceType) -> bool {
        if let Some(profile) = RESOURCE_DEFINITIONS.get(&resource_type) {
            match self.get_or_create_cell(
                pos,
                profile.biomass_cap,
                profile.growth_rate_multiplier,
            ) {
                Ok(mut cell) => {
                    cell.resource_type = Some(resource_type);
                    cell.max_biomass = profile.biomass_cap;
                    cell.growth_rate_modifier = profile.growth_rate_multiplier;
                    true
                }
                Err(e) => {
                    error!("Failed to get or create cell for resource type {:?}: {}", resource_type, e);
                    false
                }
            }
        } else {
            false
        }
    }

    /// Consume biomass at a specific location
    /// Returns amount actually consumed
    pub fn consume_at(&mut self, pos: IVec2, requested: f32, max_fraction: f32) -> f32 {
        // Collect current data before any modifications
        let current_tick = self.current_tick;
        let max_biomass = self.get_cell(pos).map(|c| c.max_biomass).unwrap_or(0.0);
        let total_biomass = self.get_cell(pos).map(|c| c.total_biomass).unwrap_or(0.0);

        // Calculate what would be consumed
        let max_by_fraction = total_biomass * max_fraction;
        let consumed = requested.min(max_by_fraction);

        if consumed > 0.0 {
            // Apply the consumption using the cell's method
            let actual_consumed = if let Some(cell) = self.get_cell_mut(pos) {
                cell.consume_biomass(requested, max_fraction, current_tick)
            } else {
                0.0
            };

            self.metrics.biomass_consumed += actual_consumed;

            // Schedule regrowth event based on resource profile
            let regrowth_delay = if let Some(cell) = self.get_cell(pos) {
                cell.get_profile()
                    .map(|p| p.regrowth_delay_ticks)
                    .unwrap_or(100) // Default delay
            } else {
                100
            };

            self.event_scheduler.schedule(GrowthEvent::Regrow {
                location: pos,
                scheduled_tick: current_tick + regrowth_delay,
            });
        }

        consumed
    }

    /// Process regrowth for a specific cell
    pub fn regrow_cell(&mut self, pos: IVec2) -> f32 {
        let current_tick = self.current_tick;

        // Collect cell data before borrowing
        let (last_update_tick, total_biomass, max_biomass) = if let Some(cell) = self.get_cell(pos) {
            (cell.last_update_tick, cell.total_biomass, cell.max_biomass)
        } else {
            return 0.0;
        };

        // Skip if already at max capacity
        if total_biomass >= max_biomass {
            return 0.0;
        }

        // Calculate growth
        let delta_ticks = current_tick.saturating_sub(last_update_tick);
        let time_factor = (delta_ticks as f32) / 10.0;
        let base_growth_rate = 0.05; // Will be multiplied by cell's growth_rate_modifier
        let growth = base_growth_rate * total_biomass * (1.0 - total_biomass / max_biomass);
        let actual_growth = (growth * time_factor).min(max_biomass - total_biomass);

        if actual_growth > 0.0 {
            // Apply the growth to the actual cell
            if let Some(cell) = self.get_cell_mut(pos) {
                cell.total_biomass += actual_growth;
                cell.last_update_tick = current_tick;

                // Schedule next regrowth if not at max capacity
                if cell.total_biomass < cell.max_biomass {
                    let biomass_fraction = cell.biomass_fraction();
                    let next_delay = calculate_regrowth_interval(biomass_fraction);
                    self.event_scheduler.schedule(GrowthEvent::Regrow {
                        location: pos,
                        scheduled_tick: current_tick + next_delay,
                    });
                }
            }

            self.metrics.biomass_grown += actual_growth;
        }

        actual_growth
    }

    /// Find the best grazing cell within a radius
    /// Returns position and biomass amount
    pub fn find_best_cell(&self, center: IVec2, radius: i32) -> Option<(IVec2, f32)> {
        let mut best_cell: Option<(IVec2, f32)> = None;

        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let pos = center + IVec2::new(dx, dy);
                if let Some(cell) = self.get_cell(pos) {
                    if cell.total_biomass >= 10.0 && !cell.is_depleted() {
                        // FORAGE_MIN_BIOMASS
                        let distance = center.as_vec2().distance(pos.as_vec2());
                        let utility = cell.total_biomass / (1.0 + distance * 0.1);

                        if let Some((_, best_utility)) = best_cell {
                            if utility > best_utility {
                                best_cell = Some((pos, cell.total_biomass));
                            }
                        } else {
                            best_cell = Some((pos, cell.total_biomass));
                        }
                    }
                }
            }
        }

        best_cell
    }

    /// Update the grid by processing due events
    pub fn update(&mut self, current_tick: u64) {
        let start_time = std::time::Instant::now();
        self.current_tick = current_tick;

        // Process due events ONLY - no per-tick loops
        let due_events = self.event_scheduler.pop_due_events(current_tick);
        self.metrics.events_processed = due_events.len();

        for event in due_events {
            match event {
                GrowthEvent::Consume { location, .. } => {
                    // Consumption already handled when scheduled
                    self.regrow_cell(location);
                }
                GrowthEvent::Regrow { location, .. } => {
                    self.regrow_cell(location);
                }
                GrowthEvent::RandomSample { locations, .. } => {
                    for location in locations {
                        self.regrow_cell(location);
                        self.metrics.random_cells_sampled += 1;
                    }
                }
            }
        }

        // NO per-tick processing - only event-driven updates
        // Removed: process_random_tick_sample() and decay_all_pressure()

        // Update processing time metric
        let elapsed = start_time.elapsed().as_micros() as u64;
        self.metrics.processing_time_us = elapsed;
    }

    /// Process a random sample of cells for ambient regrowth
    fn process_random_tick_sample(&mut self) {
        use rand::seq::IteratorRandom;
        use rand::thread_rng;

        let sample_size = self
            .event_scheduler
            .random_tick_budget()
            .min(self.cells.len());
        if sample_size == 0 {
            return;
        }

        // Efficient random sampling without allocating all keys
        let mut rng = thread_rng();
        let sample_positions: Vec<IVec2> = self
            .cells
            .keys()
            .copied()
            .choose_multiple(&mut rng, sample_size);

        if !sample_positions.is_empty() {
            self.event_scheduler.schedule(GrowthEvent::RandomSample {
                locations: sample_positions,
                scheduled_tick: self.current_tick,
            });
        }
    }

    /// Decay consumption pressure for all cells
    fn decay_all_pressure(&mut self) {
        for cell in self.cells.values_mut() {
            cell.decay_pressure(1); // Decay by 1 tick worth
        }
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> &ResourceGridMetrics {
        &self.metrics
    }

    /// Reset performance metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = ResourceGridMetrics::default();
        self.metrics.active_cells = self.cells.len();
    }
}

/// Calculate regrowth delay based on amount consumed
fn calculate_regrowth_delay(amount_consumed: f32, max_biomass: f32) -> u64 {
    // More consumption = longer regrowth delay
    let fraction_consumed = amount_consumed / max_biomass;
    let base_delay = 50; // 5 seconds at 10 TPS
    let variable_delay = (fraction_consumed * 200.0) as u64; // Up to 20 seconds
    base_delay + variable_delay
}

/// Calculate regrowth interval based on current biomass fraction
fn calculate_regrowth_interval(biomass_fraction: f32) -> u64 {
    // Lower biomass = faster regrowth (more frequent updates)
    let urgency = 1.0 - biomass_fraction;
    let base_interval = 100; // 10 seconds at 10 TPS
    let min_interval = 20; // 2 seconds minimum
    let calculated = base_interval - (urgency * 80.0) as u64;
    calculated.max(min_interval)
}

impl Default for ResourceGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GrazingCell {
    fn default() -> Self {
        Self {
            resource_type: None,
            total_biomass: 0.0,
            consumption_pressure: 0.0,
            last_update_tick: 0,
            max_biomass: 100.0,
            growth_rate_modifier: 1.0,
            regrowth_available_tick: 0,
        }
    }
}

impl Default for VegetationScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for world coordinate conversion
pub mod grid_helpers {
    use super::*;

    /// Convert world coordinates to cell coordinates (1:1 mapping for now)
    pub fn world_to_cell(world_pos: IVec2) -> IVec2 {
        world_pos
    }

    /// Convert cell coordinates to world coordinates
    pub fn cell_to_world(cell_pos: IVec2) -> IVec2 {
        cell_pos
    }

    /// Get chunk coordinates for a cell (16x16 chunks)
    pub fn cell_to_chunk(cell_pos: IVec2) -> IVec2 {
        IVec2::new(cell_pos.x.div_euclid(16), cell_pos.y.div_euclid(16))
    }

    /// Get all cells in a chunk
    pub fn chunk_to_cells(chunk_pos: IVec2) -> Vec<IVec2> {
        let mut cells = Vec::new();
        let chunk_size = 16;

        for dx in 0..chunk_size {
            for dy in 0..chunk_size {
                let world_x = chunk_pos.x * chunk_size + dx;
                let world_y = chunk_pos.y * chunk_size + dy;
                cells.push(IVec2::new(world_x, world_y));
            }
        }

        cells
    }
}

#[cfg(test)]
mod tests {
    use super::grid_helpers::*;
    use super::*;

    #[test]
    fn test_grazing_cell_creation() {
        let cell = GrazingCell::new(None, 50.0, 100.0, 1.0, 0);
        assert_eq!(cell.total_biomass, 50.0);
        assert_eq!(cell.max_biomass, 100.0);
        assert_eq!(cell.growth_rate_modifier, 1.0);
        assert_eq!(cell.consumption_pressure, 0.0);
        assert_eq!(cell.last_update_tick, 0);
        assert_eq!(cell.regrowth_available_tick, 0);
    }

    #[test]
    fn test_grazing_cell_biomass_fraction() {
        let cell = GrazingCell::new(None, 50.0, 100.0, 1.0, 0);
        assert_eq!(cell.biomass_fraction(), 0.5);

        let empty_cell = GrazingCell::new(None, 0.0, 100.0, 1.0, 0);
        assert_eq!(empty_cell.biomass_fraction(), 0.0);

        let full_cell = GrazingCell::new(None, 100.0, 100.0, 1.0, 0);
        assert_eq!(full_cell.biomass_fraction(), 1.0);
    }

    #[test]
    fn test_grazing_cell_is_depleted() {
        let depleted_cell = GrazingCell::new(None, 5.0, 100.0, 1.0, 0);
        assert!(depleted_cell.is_depleted());

        let good_cell = GrazingCell::new(None, 15.0, 100.0, 1.0, 0);
        assert!(!good_cell.is_depleted());
    }

    #[test]
    fn test_grazing_cell_regrowth() {
        let mut cell = GrazingCell::new(None, 50.0, 100.0, 1.0, 0);
        let initial_biomass = cell.total_biomass;

        let growth = cell.apply_regrowth(10, 10); // 1 second at 10 TPS
        assert!(growth > 0.0);
        assert!(cell.total_biomass > initial_biomass);
        assert!(cell.total_biomass <= cell.max_biomass);
    }

    #[test]
    fn test_grazing_cell_consumption() {
        let mut cell = GrazingCell::new(None, 80.0, 100.0, 1.0, 0);

        // Test normal consumption
        let consumed = cell.consume_biomass(20.0, 0.3, 0);
        assert_eq!(consumed, 20.0);
        assert_eq!(cell.total_biomass, 60.0);
        assert!(cell.consumption_pressure > 0.0);

        // Test max fraction limit
        let consumed_limited = cell.consume_biomass(50.0, 0.3, 0); // 30% of 60 = 18
        assert_eq!(consumed_limited, 18.0);
        assert_eq!(cell.total_biomass, 42.0);
    }

    #[test]
    fn test_grazing_cell_pressure_decay() {
        let mut cell = GrazingCell::new(None, 80.0, 100.0, 1.0, 0);
        cell.consume_biomass(20.0, 1.0, 0);
        assert!(cell.consumption_pressure > 0.0);

        let initial_pressure = cell.consumption_pressure;
        cell.decay_pressure(50); // 5 seconds at 10 TPS
        assert!(cell.consumption_pressure < initial_pressure);
    }

    #[test]
    fn test_vegetation_scheduler_basic() {
        let mut scheduler = VegetationScheduler::new();
        assert_eq!(scheduler.pending_count(), 0);

        let event = GrowthEvent::Regrow {
            location: IVec2::new(5, 10),
            scheduled_tick: 100,
        };

        scheduler.schedule(event.clone());
        assert_eq!(scheduler.pending_count(), 1);

        // No events due yet
        let due_events = scheduler.pop_due_events(50);
        assert_eq!(due_events.len(), 0);
        assert_eq!(scheduler.pending_count(), 1);

        // Event should be due now
        let due_events = scheduler.pop_due_events(150);
        assert_eq!(due_events.len(), 1);
        assert_eq!(scheduler.pending_count(), 0);
    }

    #[test]
    fn test_vegetation_scheduler_ordering() {
        let mut scheduler = VegetationScheduler::new();

        // Schedule events out of order
        scheduler.schedule(GrowthEvent::Regrow {
            location: IVec2::new(1, 1),
            scheduled_tick: 200,
        });

        scheduler.schedule(GrowthEvent::Regrow {
            location: IVec2::new(2, 2),
            scheduled_tick: 100,
        });

        scheduler.schedule(GrowthEvent::Regrow {
            location: IVec2::new(3, 3),
            scheduled_tick: 150,
        });

        // Events should come out in chronological order
        let due_events = scheduler.pop_due_events(250);
        assert_eq!(due_events.len(), 3);

        match &due_events[0] {
            GrowthEvent::Regrow { location, .. } => assert_eq!(*location, IVec2::new(2, 2)),
            _ => panic!("Expected Regrow event"),
        }

        match &due_events[1] {
            GrowthEvent::Regrow { location, .. } => assert_eq!(*location, IVec2::new(3, 3)),
            _ => panic!("Expected Regrow event"),
        }

        match &due_events[2] {
            GrowthEvent::Regrow { location, .. } => assert_eq!(*location, IVec2::new(1, 1)),
            _ => panic!("Expected Regrow event"),
        }
    }

    #[test]
    fn test_grid_helpers() {
        let world_pos = IVec2::new(32, 48);
        let cell_pos = world_to_cell(world_pos);
        assert_eq!(cell_pos, world_pos);

        let back_to_world = cell_to_world(cell_pos);
        assert_eq!(back_to_world, world_pos);

        let chunk_pos = cell_to_chunk(cell_pos);
        assert_eq!(chunk_pos, IVec2::new(2, 3)); // 32/16=2, 48/16=3

        let cells_in_chunk = chunk_to_cells(chunk_pos);
        assert_eq!(cells_in_chunk.len(), 256); // 16x16 = 256 cells
        assert!(cells_in_chunk.contains(&IVec2::new(32, 48)));
    }

    #[test]
    fn test_growth_event_properties() {
        let consume_event = GrowthEvent::Consume {
            location: IVec2::new(5, 10),
            amount_consumed: 20.0,
            scheduled_tick: 100,
        };

        assert_eq!(consume_event.scheduled_tick(), 100);
        assert_eq!(consume_event.locations(), vec![IVec2::new(5, 10)]);

        let random_event = GrowthEvent::RandomSample {
            locations: vec![IVec2::new(1, 1), IVec2::new(2, 2)],
            scheduled_tick: 200,
        };

        assert_eq!(random_event.scheduled_tick(), 200);
        assert_eq!(random_event.locations().len(), 2);
    }

    #[test]
    fn test_resource_grid_creation() {
        let grid = ResourceGrid::new();
        assert_eq!(grid.cell_count(), 0);
        assert_eq!(grid.current_tick(), 0);
        assert_eq!(grid.pending_events(), 0);
    }

    #[test]
    fn test_resource_grid_get_or_create_cell() {
        let mut grid = ResourceGrid::new();
        let pos = IVec2::new(5, 10);

        // Cell doesn't exist initially
        assert!(grid.get_cell(pos).is_none());

        // Create cell
        let cell = grid.get_or_create_cell(pos, 100.0, 1.0).unwrap();
        assert_eq!(cell.total_biomass, 5.0); // INITIAL_BIOMASS
        assert_eq!(grid.cell_count(), 1);

        // Get existing cell
        let existing_cell = grid.get_or_create_cell(pos, 100.0, 1.0).unwrap();
        assert_eq!(existing_cell.total_biomass, 5.0); // Same cell
        assert_eq!(grid.cell_count(), 1); // No new cell created
    }

    #[test]
    fn test_resource_grid_consume_at() {
        let mut grid = ResourceGrid::new();
        let pos = IVec2::new(5, 10);

        // Create cell with full biomass
        let cell = grid.get_or_create_cell(pos, 100.0, 1.0);
        cell.total_biomass = 80.0;

        // Consume biomass
        let consumed = grid.consume_at(pos, 20.0, 0.3);
        assert_eq!(consumed, 20.0);

        let updated_cell = grid.get_cell(pos).unwrap();
        assert_eq!(updated_cell.total_biomass, 60.0);
        assert!(grid.pending_events() > 0); // Regrowth event scheduled

        // Try to consume from non-existent cell
        let consumed_empty = grid.consume_at(IVec2::new(99, 99), 10.0, 0.3);
        assert_eq!(consumed_empty, 0.0);
    }

    #[test]
    fn test_resource_grid_find_best_cell() {
        let mut grid = ResourceGrid::new();

        // Create cells with different biomass levels
        let pos1 = IVec2::new(0, 0);
        let pos2 = IVec2::new(3, 0);
        let pos3 = IVec2::new(1, 1);

        if let Ok(cell) = grid.get_or_create_cell(pos1, 100.0, 1.0) {
            cell.total_biomass = 50.0;
        }
        if let Ok(cell) = grid.get_or_create_cell(pos2, 100.0, 1.0) {
            cell.total_biomass = 80.0;
        }
        if let Ok(cell) = grid.get_or_create_cell(pos3, 100.0, 1.0) {
            cell.total_biomass = 30.0;
        }

        // Find best cell near origin
        let best = grid.find_best_cell(IVec2::new(0, 0), 5);
        assert!(best.is_some());

        let (best_pos, best_biomass) = best.unwrap();
        // Should prefer the highest biomass considering distance
        // (3,0) has 80.0 biomass but distance 3, (0,0) has 50.0 but distance 0
        // Utility calculation should determine the winner
        assert!(best_pos == pos1 || best_pos == pos2);
        assert!(best_biomass >= 30.0);
    }

    #[test]
    fn test_resource_grid_update() {
        let mut grid = ResourceGrid::new();
        let pos = IVec2::new(5, 10);

        // Create cell and schedule event
        grid.get_or_create_cell(pos, 100.0, 1.0).unwrap();
        grid.event_scheduler.schedule(GrowthEvent::Regrow {
            location: pos,
            scheduled_tick: 100,
        });

        // Update before event is due
        grid.update(50);
        assert_eq!(grid.current_tick(), 50);
        assert_eq!(grid.pending_events(), 1); // Event still pending

        // Update after event is due
        grid.update(150);
        assert_eq!(grid.current_tick(), 150);
        assert!(grid.get_metrics().events_processed > 0);
        assert!(
            grid.pending_events() >= 1,
            "Regrowth should schedule follow-up events"
        );
    }

    #[test]
    fn test_regrowth_delay_calculation() {
        let delay1 = calculate_regrowth_delay(10.0, 100.0); // 10% consumed
        let delay2 = calculate_regrowth_delay(50.0, 100.0); // 50% consumed

        assert!(delay2 > delay1); // More consumption = longer delay
        assert!(delay1 >= 50); // Base delay
        assert!(delay2 <= 250); // Base + max variable
    }

    #[test]
    fn test_regrowth_interval_calculation() {
        let interval_low = calculate_regrowth_interval(0.2); // 20% biomass
        let interval_high = calculate_regrowth_interval(0.8); // 80% biomass

        assert!(interval_low < interval_high); // Lower biomass = faster regrowth
        assert!(interval_low >= 20); // Minimum interval
        assert!(interval_high <= 100); // Base interval
    }

    #[test]
    fn test_sparse_storage_efficiency() {
        let mut grid = ResourceGrid::new();

        // Only store cells that have biomass
        let positions = vec![
            IVec2::new(0, 0),
            IVec2::new(100, 100),
            IVec2::new(-50, 75),
            IVec2::new(25, -30),
        ];

        for pos in positions {
            grid.get_or_create_cell(pos, 100.0, 1.0).unwrap();
        }

        // Grid should only store the 4 cells we created
        assert_eq!(grid.cell_count(), 4);

        // Non-existent positions should return None
        assert!(grid.get_cell(IVec2::new(999, 999)).is_none());
        assert!(grid.get_cell(IVec2::new(1, 1)).is_none());

        // This demonstrates sparse storage - we don't store empty cells
    }

    #[test]
    fn test_resource_cell_from_type() {
        use crate::resources::ResourceType;

        let cell = GrazingCell::from_resource_type(ResourceType::BerryBush, 1000).unwrap();

        assert_eq!(cell.resource_type, Some(ResourceType::BerryBush));
        assert!(cell.total_biomass > 0.0);
        assert!(cell.max_biomass > 0.0);
        assert_eq!(cell.last_update_tick, 1000);
    }

    #[test]
    fn test_resource_cell_profile_access() {
        use crate::resources::ResourceType;

        let cell = GrazingCell::from_resource_type(ResourceType::MushroomPatch, 1000).unwrap();
        let profile = cell.get_profile().unwrap();

        assert_eq!(profile.category, crate::resources::ResourceCategory::Collectable);
        assert_eq!(profile.consumption_kind, crate::resources::ConsumptionKind::HumanGather);
        assert!(profile.biomass_cap > 0.0);
    }

    #[test]
    fn test_resource_cell_consumption_with_delay() {
        use crate::resources::ResourceType;

        let mut cell = GrazingCell::from_resource_type(ResourceType::WildRoot, 1000).unwrap();
        let initial_biomass = cell.total_biomass;

        // Consume some biomass
        let consumed = cell.consume_biomass(5.0, 0.5, 1000);
        assert!(consumed > 0.0);
        assert!(cell.total_biomass < initial_biomass);

        // Check that regrowth delay was applied for collectable
        assert!(cell.regrowth_available_tick > 1000);

        // Try to consume again before regrowth is available
        let consumed_again = cell.consume_biomass(5.0, 0.5, 1001);
        assert_eq!(consumed_again, 0.0); // Should be blocked by regrowth delay
    }

    #[test]
    fn test_resource_grid_with_resource_type() {
        use crate::resources::ResourceType;

        let mut grid = ResourceGrid::new();
        let pos = IVec2::new(5, 10);

        // Create a cell with a specific resource type
        let cell = grid.get_or_create_cell_with_resource(pos, ResourceType::BerryBush);
        assert!(cell.is_some());

        let created_cell = cell.unwrap();
        assert_eq!(created_cell.resource_type, Some(ResourceType::BerryBush));
        assert!(created_cell.total_biomass > 0.0);
    }

    #[test]
    fn test_apply_profile_to_existing_cell() {
        use crate::resources::ResourceType;

        let mut grid = ResourceGrid::new();
        let pos = IVec2::new(5, 10);

        // Create a basic cell first
        let basic_cell = grid.get_or_create_cell(pos, 50.0, 1.0).unwrap();
        assert!(basic_cell.resource_type.is_none());

        // Apply a resource profile
        let applied = grid.apply_profile(pos, ResourceType::HazelShrub);
        assert!(applied);

        // Check that the cell now has the resource type
        let updated_cell = grid.get_cell(pos).unwrap();
        assert_eq!(updated_cell.resource_type, Some(ResourceType::HazelShrub));
        assert!(updated_cell.max_biomass > 50.0); // Should be updated to profile value
    }
}
