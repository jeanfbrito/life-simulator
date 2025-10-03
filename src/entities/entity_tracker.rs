/// Entity tracker for web API access
/// Maintains a global list of entity positions that can be queried by the web server
use bevy::prelude::*;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use crate::entities::{Creature, movement::TilePosition, stats::{Hunger, Thirst, Energy, Health}, CurrentAction};
use crate::entities::reproduction::{Sex, Age, WellFedStreak, ReproductionCooldown, Pregnancy};
use crate::entities::types::rabbit::RabbitReproductionConfig;

// ============================================================================
// GLOBAL STATE
// ============================================================================

/// Entity data for web API
#[derive(Debug, Clone)]
pub struct EntityData {
    pub entity_id: u32,
    pub name: String,
    pub species: String,
    pub position: IVec2,
    pub hunger: Option<f32>,
    pub thirst: Option<f32>,
    pub energy: Option<f32>,
    pub health: Option<f32>,
    pub current_action: Option<String>,
    pub sex: Option<String>,
    pub is_juvenile: Option<bool>,
    // Reproduction diagnostics (primarily for rabbits)
    pub well_fed_streak: Option<u32>,
    pub well_fed_required_ticks: Option<u32>,
    pub eligible_to_mate: Option<bool>,
    pub pregnancy_remaining_ticks: Option<u32>,
    pub gestation_total_ticks: Option<u32>,
    pub reproduction_cooldown_ticks: Option<u32>,
    pub ticks_to_adult: Option<u32>,
}

/// Global entity tracker
static mut ENTITY_TRACKER: Option<Arc<RwLock<EntityTracker>>> = None;

#[derive(Debug, Default)]
pub struct EntityTracker {
    entities: HashMap<u32, EntityData>,
}

impl EntityTracker {
    /// Initialize the global tracker
    pub fn init() {
        unsafe {
            ENTITY_TRACKER = Some(Arc::new(RwLock::new(EntityTracker::default())));
        }
    }
    
    /// Get a clone of the global tracker
    pub fn global() -> Option<Arc<RwLock<EntityTracker>>> {
        unsafe { ENTITY_TRACKER.as_ref().cloned() }
    }
    
    /// Update entity data
    pub fn update(&mut self, entity_id: u32, data: EntityData) {
        self.entities.insert(entity_id, data);
    }
    
    /// Remove entity
    pub fn remove(&mut self, entity_id: u32) {
        self.entities.remove(&entity_id);
    }
    
    /// Get all entities as JSON
    pub fn to_json(&self) -> String {
        let entities_json: Vec<String> = self.entities.values()
            .map(|e| {
                let mut parts = vec![
                    format!(r#""id": {}"#, e.entity_id),
                    format!(r#""name": "{}""#, e.name),
                    format!(r#""entity_type": "{}""#, e.species),
                    format!(r#""position": {{"x": {}, "y": {}}}"#, e.position.x, e.position.y),
                ];
                
                // Add stats if present
                if let Some(hunger) = e.hunger {
                    parts.push(format!(r#""hunger": {:.1}"#, hunger));
                }
                if let Some(thirst) = e.thirst {
                    parts.push(format!(r#""thirst": {:.1}"#, thirst));
                }
                if let Some(energy) = e.energy {
                    parts.push(format!(r#""energy": {:.1}"#, energy));
                }
                if let Some(health) = e.health {
                    parts.push(format!(r#""health": {:.1}"#, health));
                }
                if let Some(ref action) = e.current_action {
                    parts.push(format!(r#""current_action": "{}""#, action));
                }
                if let Some(ref sex) = e.sex {
                    parts.push(format!(r#""sex": "{}""#, sex));
                }
                if let Some(is_juv) = e.is_juvenile {
                    parts.push(format!(r#""is_juvenile": {}"#, is_juv));
                }
                if let Some(v) = e.well_fed_streak { parts.push(format!(r#""well_fed_streak": {}"#, v)); }
                if let Some(v) = e.well_fed_required_ticks { parts.push(format!(r#""well_fed_required_ticks": {}"#, v)); }
                if let Some(v) = e.eligible_to_mate { parts.push(format!(r#""eligible_to_mate": {}"#, v)); }
                if let Some(v) = e.pregnancy_remaining_ticks { parts.push(format!(r#""pregnancy_remaining_ticks": {}"#, v)); }
                if let Some(v) = e.gestation_total_ticks { parts.push(format!(r#""gestation_total_ticks": {}"#, v)); }
                if let Some(v) = e.reproduction_cooldown_ticks { parts.push(format!(r#""reproduction_cooldown_ticks": {}"#, v)); }
                if let Some(v) = e.ticks_to_adult { parts.push(format!(r#""ticks_to_adult": {}"#, v)); }
                
                format!(r#"{{{}}}"#, parts.join(", "))
            })
            .collect();
        
        format!(r#"{{"entities": [{}]}}"#, entities_json.join(","))
    }
    
    /// Get entity count
    pub fn count(&self) -> usize {
        self.entities.len()
    }
}

// ============================================================================
// BEVY SYSTEMS
// ============================================================================

/// System that syncs entity data to the global tracker
/// Runs every frame to keep web API up to date
pub fn sync_entities_to_tracker(
    query: Query<(
        Entity,
        &Creature,
        &TilePosition,
        Option<&Hunger>,
        Option<&Thirst>,
        Option<&Energy>,
        Option<&Health>,
        Option<&CurrentAction>,
        Option<&Sex>,
        Option<&Age>,
        Option<&WellFedStreak>,
        Option<&ReproductionCooldown>,
        Option<&Pregnancy>,
    )>,
    rabbit_cfg: Option<Res<RabbitReproductionConfig>>,
) {
    if let Some(tracker) = EntityTracker::global() {
        if let Ok(mut tracker) = tracker.write() {
            // Clear and rebuild (simple approach)
            tracker.entities.clear();
            
            for (entity, creature, position, hunger, thirst, energy, health, current_action, sex, age, wellfed, cooldown, pregnancy) in query.iter() {
                let sex_str = sex.map(|s| match s { Sex::Male => "male".to_string(), Sex::Female => "female".to_string() });
                let is_juvenile = age.map(|a| !a.is_adult());

                // Defaults
                let mut well_fed_streak = None;
                let mut well_fed_required_ticks = None;
                let mut eligible_to_mate = None;
                let mut pregnancy_remaining_ticks = None;
                let mut gestation_total_ticks = None;
                let mut reproduction_cooldown_ticks = None;
                let mut ticks_to_adult = None;

                // Provide reproduction diagnostics for rabbits
                if creature.species == "Rabbit" {
                    if let Some(cfg) = rabbit_cfg.as_deref() {
                        // Well-fed
                        if let Some(wf) = wellfed { 
                            well_fed_streak = Some(wf.ticks);
                            well_fed_required_ticks = Some(cfg.well_fed_required_ticks);
                        }
                        // Cooldown
                        if let Some(cd) = cooldown { reproduction_cooldown_ticks = Some(cd.remaining_ticks); }
                        // Pregnancy
                        if let Some(preg) = pregnancy { 
                            pregnancy_remaining_ticks = Some(preg.remaining_ticks);
                            gestation_total_ticks = Some(cfg.gestation_ticks);
                        }
                        // Maturity
                        if let Some(a) = age {
                            if !a.is_adult() {
                                let remain = a.mature_at_ticks.saturating_sub(a.ticks_alive as u32);
                                ticks_to_adult = Some(remain);
                            }
                        }
                        // Eligibility (only if we have all needed values and not pregnant)
                        if let (Some(a), Some(cd), Some(en), Some(hp), Some(wf), Some(_sx)) = (age, cooldown, energy, health, wellfed, sex) {
                            let has_preg = pregnancy.is_some();
                            let basic = a.is_adult()
                                && cd.remaining_ticks == 0
                                && en.0.normalized() >= cfg.min_energy_norm
                                && hp.0.normalized() >= cfg.min_health_norm
                                && wf.ticks >= cfg.well_fed_required_ticks;
                            let eligible = basic && !has_preg;
                            eligible_to_mate = Some(eligible);
                        }
                    }
                }

                let data = EntityData {
                    entity_id: entity.index(),
                    name: creature.name.clone(),
                    species: creature.species.clone(),
                    position: position.tile,
                    hunger: hunger.map(|h| h.0.percentage()),
                    thirst: thirst.map(|t| t.0.percentage()),
                    energy: energy.map(|e| e.0.percentage()),
                    health: health.map(|h| h.0.percentage()),
                    current_action: current_action.map(|a| a.action_name.clone()),
                    sex: sex_str,
                    is_juvenile,
                    well_fed_streak,
                    well_fed_required_ticks,
                    eligible_to_mate,
                    pregnancy_remaining_ticks,
                    gestation_total_ticks,
                    reproduction_cooldown_ticks,
                    ticks_to_adult,
                };
                tracker.update(entity.index(), data);
            }
        }
    }
}

/// Startup system to initialize the tracker
pub fn init_entity_tracker() {
    EntityTracker::init();
    info!("Entity tracker initialized");
}

// ============================================================================
// WEB API HELPER
// ============================================================================

/// Get entities as JSON string (for web server)
pub fn get_entities_json() -> String {
    if let Some(tracker) = EntityTracker::global() {
        if let Ok(tracker) = tracker.read() {
            return tracker.to_json();
        }
    }
    r#"{"entities": []}"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_tracker() {
        let mut tracker = EntityTracker::default();
        
        let data = EntityData {
            entity_id: 1,
            name: "Test".to_string(),
            species: "Human".to_string(),
            position: IVec2::new(5, 10),
            hunger: Some(50.0),
            thirst: Some(30.0),
            energy: Some(80.0),
            health: Some(100.0),
            current_action: None,
        };
        
        tracker.update(1, data);
        assert_eq!(tracker.count(), 1);
        
        tracker.remove(1);
        assert_eq!(tracker.count(), 0);
    }
    
    #[test]
    fn test_to_json() {
        let mut tracker = EntityTracker::default();
        
        tracker.update(1, EntityData {
            entity_id: 1,
            name: "Bob".to_string(),
            species: "Human".to_string(),
            position: IVec2::new(0, 0),
            hunger: Some(50.0),
            thirst: Some(30.0),
            energy: Some(80.0),
            health: Some(100.0),
            current_action: None,
        });
        
        let json = tracker.to_json();
        assert!(json.contains("\"name\": \"Bob\""));
        assert!(json.contains("\"x\": 0"));
    }
}
