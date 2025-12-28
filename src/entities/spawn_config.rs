/// Spawn Bootstrap Configuration
///
/// Configuration for entity spawning that can be modified without recompilation.
/// This replaces hard-coded spawn logic in main.rs with data-driven configuration.
use bevy::prelude::*;
use rand::Rng;
use ron::ser::to_string_pretty;
use serde::{Deserialize, Serialize};

use crate::entities::reproduction::Sex;
use crate::entities::spawn_using_registry;
use crate::pathfinding::PathfindingGrid;
use crate::tilemap::TerrainType;
use crate::world_loader::WorldLoader;

/// Configuration for spawning a group of entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnGroup {
    /// Species type to spawn
    pub species: String,

    /// Number of entities to spawn
    pub count: usize,

    /// Names to assign to spawned entities (if count > names.len, names will be reused)
    pub names: Vec<String>,

    /// Spawn position configuration
    pub spawn_area: SpawnArea,

    /// Optional sequence of sexes to apply to spawned entities (cycles if shorter than count)
    pub sex_sequence: Option<Vec<SpawnSex>>,

    /// Optional custom messages for console output
    pub messages: Option<SpawnMessages>,
}

/// Sex assignment options for configured spawns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SpawnSex {
    Male,
    Female,
}

/// Area configuration for spawning entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnArea {
    /// Center position for spawning (x, y)
    pub center: (i32, i32),

    /// Search radius for walkable tiles
    pub search_radius: i32,

    /// Maximum spawn attempts per entity
    pub max_attempts: usize,
}

/// Custom messages for spawn logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnMessages {
    /// Message shown when starting to spawn this group
    pub start_message: String,

    /// Message template for successful spawn (supports {name}, {index}, {pos} placeholders)
    pub success_template: String,

    /// Message shown when spawning is complete
    pub completion_message: String,
}

/// Complete spawn configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnConfig {
    /// Groups of entities to spawn
    pub spawn_groups: Vec<SpawnGroup>,

    /// Global spawn settings
    pub settings: SpawnSettings,
}

/// Global spawn settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnSettings {
    /// Whether to show detailed spawn messages
    pub verbose_logging: bool,

    /// Whether to spawn entities at all (useful for testing)
    pub enable_spawning: bool,

    /// Message shown when demo spawning is complete
    pub completion_message: String,

    /// Additional informational messages printed after spawning succeeds
    pub post_spawn_messages: Vec<String>,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            spawn_groups: vec![
                // Rabbits group
                SpawnGroup {
                    species: "Rabbit".to_string(),
                    count: 5,
                    names: vec![
                        "Bugs".to_string(),
                        "Roger".to_string(),
                        "Thumper".to_string(),
                        "Peter".to_string(),
                        "Clover".to_string(),
                    ],
                    spawn_area: SpawnArea {
                        center: (0, 0),
                        search_radius: 15,
                        max_attempts: 30,
                    },
                    sex_sequence: None,
                    messages: Some(SpawnMessages {
                        start_message: "🎯 LIFE_SIMULATOR: Spawning 5 rabbits for testing...".to_string(),
                        success_template: "   ✅ Spawned rabbit #{index}: {name} 🐇 at {pos}".to_string(),
                        completion_message: "✅ LIFE_SIMULATOR: Spawned {count} rabbits successfully!".to_string(),
                    }),
                },

                // Deer pair group
                SpawnGroup {
                    species: "Deer".to_string(),
                    count: 2,
                    names: vec![
                        "Stag".to_string(),
                        "Doe".to_string(),
                    ],
                    spawn_area: SpawnArea {
                        center: (0, 0),
                        search_radius: 5,
                        max_attempts: 50,
                    },
                    sex_sequence: Some(vec![SpawnSex::Male, SpawnSex::Female]),
                    messages: Some(SpawnMessages {
                        start_message: "🦌 Spawning deer pair near origin...".to_string(),
                        success_template: "   🦌 Spawned deer {name} ({sex}) at {pos}".to_string(),
                        completion_message: "🦌 Deer pair ready for testing ({count} spawned)".to_string(),
                    }),
                },

                // Raccoon pair group
                SpawnGroup {
                    species: "Raccoon".to_string(),
                    count: 2,
                    names: vec![
                        "Bandit".to_string(),
                        "Maple".to_string(),
                    ],
                    spawn_area: SpawnArea {
                        center: (-30, 10),     // Away from water/beach areas
                        search_radius: 30,     // Moderate radius for forest areas
                        max_attempts: 200,     // Many attempts to find suitable terrain
                    },
                    sex_sequence: Some(vec![SpawnSex::Male, SpawnSex::Female]),
                    messages: Some(SpawnMessages {
                        start_message: "🦝 Spawning raccoon pair near watering hole...".to_string(),
                        success_template: "   🦝 Spawned raccoon {name} ({sex}) at {pos}".to_string(),
                        completion_message: "🦝 Raccoon pair active near campsite ({count} spawned)".to_string(),
                    }),
                },
            ],
            settings: SpawnSettings {
                verbose_logging: true,
                enable_spawning: true,
                completion_message: "🌐 LIFE_SIMULATOR: View at http://127.0.0.1:54321/viewer.html\n🌐 LIFE_SIMULATOR: Entity API at http://127.0.0.1:54321/api/entities".to_string(),
                post_spawn_messages: vec![
                    "📊 Rabbits will only move when thirsty/hungry (no wandering)".to_string(),
                    "🧠 Behavior: Drinks at 15% thirst, grazes at 3-8 tile range".to_string(),
                    "🦌 Example: Deer follow their mothers while idle".to_string(),
                ],
            },
        }
    }
}

impl SpawnConfig {
    /// Load spawn configuration from RON file
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: SpawnConfig = ron::from_str(&content)?;
        Ok(config)
    }

    /// Load spawn configuration from default location, or create default if file doesn't exist
    pub fn load_or_default() -> Self {
        // Check for environment variable to override config file
        let config_path = std::env::var("SPAWN_CONFIG")
            .unwrap_or_else(|_| "config/spawn_config.ron".to_string());

        match Self::load_from_file(&config_path) {
            Ok(config) => {
                println!("📋 Loaded spawn configuration from {}", config_path);
                config
            }
            Err(e) => {
                println!(
                    "📋 Could not load spawn config from {} ({}), using defaults",
                    config_path, e
                );
                let default_config = Self::default();

                // Only create default config file if using the default path
                // Don't overwrite test configs!
                if config_path == "config/spawn_config.ron" {
                    if let Err(e) = default_config.save_to_file(&config_path) {
                        println!("⚠️  Could not create default config file: {}", e);
                    } else {
                        println!("💾 Created default config file at {}", config_path);
                    }
                }

                default_config
            }
        }
    }

    /// Save spawn configuration to RON file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure directory exists
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get the next name for a group (cycles through names if needed)
    pub fn get_name_for_group(&self, group_index: usize, entity_index: usize) -> String {
        if let Some(group) = self.spawn_groups.get(group_index) {
            if group.names.is_empty() {
                format!("{}_{}", group.species, entity_index + 1)
            } else {
                let name_index = entity_index % group.names.len();
                group.names[name_index].clone()
            }
        } else {
            format!("Entity_{}", entity_index + 1)
        }
    }
}

/// System to spawn entities based on configuration
pub fn spawn_entities_from_config(
    mut commands: Commands,
    pathfinding_grid: Res<PathfindingGrid>,
    world_loader: Res<WorldLoader>,
) {
    let config = SpawnConfig::load_or_default();
    let verbose = config.settings.verbose_logging;

    if !config.settings.enable_spawning {
        println!("🚫 Entity spawning disabled in configuration");
        return;
    }

    let mut total_spawned = 0;
    let mut group_index = 0;

    for group in &config.spawn_groups {
        if verbose {
            if let Some(messages) = &group.messages {
                if !messages.start_message.is_empty() {
                    println!("{}", messages.start_message);
                }
            }
        }

        let mut group_spawned = 0;
        let mut rng = rand::thread_rng();

        for entity_index in 0..group.count {
            let name = config.get_name_for_group(group_index, entity_index);

            // Try to find a walkable spawn position on grass or forest
            let spawn_pos = (0..group.spawn_area.max_attempts).find_map(|attempt| {
                let dx =
                    rng.gen_range(-group.spawn_area.search_radius..=group.spawn_area.search_radius);
                let dy =
                    rng.gen_range(-group.spawn_area.search_radius..=group.spawn_area.search_radius);
                let candidate = IVec2::new(
                    group.spawn_area.center.0 + dx,
                    group.spawn_area.center.1 + dy,
                );

                // Check if walkable AND on grass/forest terrain
                if pathfinding_grid.is_walkable(candidate) {
                    if let Some(terrain) = world_loader.get_terrain_at(candidate.x, candidate.y) {
                        // Only spawn on Grass or Forest (fertile grazing areas)
                        if terrain == "Grass" || terrain == "Forest" {
                            if verbose && (attempt == 0 || attempt % 50 == 0) {
                                println!("   ✅ Found suitable terrain {} at {} (attempt {})", terrain, candidate, attempt);
                            }
                            return Some(candidate);
                        } else if verbose && attempt < 5 {
                            println!("   ⚠️ Rejected terrain {} at {} (attempt {})", terrain, candidate, attempt);
                        }
                    }
                } else if verbose && attempt < 5 {
                    println!("   ⚠️ Not walkable at {} (attempt {})", candidate, attempt);
                }
                None
            });

            if let Some(spawn_pos) = spawn_pos {
                // Spawn the entity using the registry
                let entity =
                    spawn_using_registry(&mut commands, &group.species, name.clone(), spawn_pos);

                // Apply configured sex pattern if provided
                let mut applied_sex: Option<Sex> = None;
                if let Some(sequence) = &group.sex_sequence {
                    if let Some(pattern) = sequence.get(entity_index % sequence.len()) {
                        let sex = match pattern {
                            SpawnSex::Male => Sex::Male,
                            SpawnSex::Female => Sex::Female,
                        };
                        commands.entity(entity).insert(sex);
                        applied_sex = Some(sex);
                    }
                }

                group_spawned += 1;

                // Log successful spawn if messages are configured
                if verbose {
                    if let Some(messages) = &group.messages {
                        if !messages.success_template.is_empty() {
                            let sex_label = applied_sex
                                .map(|s| match s {
                                    Sex::Male => "Male",
                                    Sex::Female => "Female",
                                })
                                .unwrap_or("");
                            let log_message = messages
                                .success_template
                                .replace("{name}", &name)
                                .replace("{index}", &(entity_index + 1).to_string())
                                .replace("{pos}", &format!("{:?}", spawn_pos))
                                .replace("{species}", &group.species)
                                .replace("{sex}", sex_label);
                            println!("{}", log_message);
                        }
                    }
                }
            } else {
                eprintln!("   ❌ Failed to find walkable spawn position for {}!", name);
            }
        }

        total_spawned += group_spawned;

        // Log completion message if configured
        if verbose {
            if let Some(messages) = &group.messages {
                if !messages.completion_message.is_empty() {
                    let completion_msg = messages
                        .completion_message
                        .replace("{count}", &group_spawned.to_string())
                        .replace("{species}", &group.species);
                    println!("{}", completion_msg);
                }
            }
        }

        group_index += 1;
    }

    // Log final messages
    if total_spawned > 0 {
        if verbose {
            for message in &config.settings.post_spawn_messages {
                println!("{}", message);
            }
        }

        if !config.settings.completion_message.is_empty() {
            println!("{}", config.settings.completion_message);
        }
    } else {
        eprintln!("❌ LIFE_SIMULATOR: Failed to spawn any entities!");
    }
}
