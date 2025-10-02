/// Entity tracker for web API access
/// Maintains a global list of entity positions that can be queried by the web server
use bevy::prelude::*;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use crate::entities::{Creature, movement::TilePosition, stats::{Hunger, Thirst, Energy, Health}, CurrentAction};

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
    )>,
) {
    if let Some(tracker) = EntityTracker::global() {
        if let Ok(mut tracker) = tracker.write() {
            // Clear and rebuild (simple approach)
            tracker.entities.clear();
            
            for (entity, creature, position, hunger, thirst, energy, health, current_action) in query.iter() {
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
