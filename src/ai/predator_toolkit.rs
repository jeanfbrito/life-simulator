//! Predator-specific AI helpers.
//!
//! Provides evaluation utilities for omnivores and carnivores.

use crate::ai::action::ActionType;
use crate::ai::herbivore_toolkit;
use crate::ai::planner::UtilityScore;
use crate::ai::behaviors::eating::HerbivoreDiet;
use crate::entities::entity_types::{Deer, Rabbit};
use crate::entities::reproduction::Age;
use crate::entities::stats::{Energy, Hunger, Thirst};
use crate::entities::{BehaviorConfig, Carcass, FearState, TilePosition};
use crate::vegetation::resource_grid::ResourceGrid;
use crate::world_loader::WorldLoader;
use bevy::prelude::*;

const BEAR_FAWN_RADIUS: f32 = 20.0;
// Foxes roam widely and can track rabbits over large territories; use an expansive
// hunt radius so they stop starving when prey wanders just outside their idle loop.
const FOX_HUNT_RADIUS: f32 = 120.0;
const WOLF_HUNT_RADIUS: f32 = 200.0;

const MIN_CARCASS_NUTRITION: f32 = 5.0;

const SCAVENGE_PRIORITY_BEAR: i32 = 400;
const SCAVENGE_PRIORITY_FOX: i32 = 320;
const SCAVENGE_PRIORITY_WOLF: i32 = 300;
const HUNT_PRIORITY_FOX: i32 = 360;
const HUNT_PRIORITY_BEAR: i32 = 320;
const HUNT_PRIORITY_WOLF: i32 = 420;

fn hunger_norm(hunger: &Hunger) -> f32 {
    hunger.0.normalized()
}

fn energy_norm(energy: &Energy) -> f32 {
    energy.0.normalized()
}

fn distance(a: IVec2, b: IVec2) -> f32 {
    let diff = a - b;
    diff.x.abs().max(diff.y.abs()) as f32
}

fn best_carcass(
    here: IVec2,
    search_radius: i32,
    carcasses: &Query<(Entity, &TilePosition, &Carcass)>,
) -> Option<(Entity, f32, f32)> {
    let mut best: Option<(Entity, f32, f32, f32)> = None;
    for (entity, tile, carcass) in carcasses.iter() {
        if carcass.nutrition <= MIN_CARCASS_NUTRITION {
            continue;
        }
        let dist = distance(here, tile.tile);
        if dist > search_radius as f32 {
            continue;
        }
        let score = carcass.nutrition * (search_radius as f32 - dist).max(0.0);
        if best
            .as_ref()
            .map(|(_, _, _, best_score)| score > *best_score)
            .unwrap_or(true)
        {
            best = Some((entity, dist, carcass.nutrition, score));
        }
    }
    best.map(|(entity, dist, nutrition, _)| (entity, dist, nutrition))
}

fn nearest_fawn(
    here: IVec2,
    radius: f32,
    deer: &Query<(Entity, &TilePosition, Option<&Age>), With<Deer>>,
) -> Option<(Entity, f32)> {
    deer.iter()
        .filter_map(|(entity, tile, age)| {
            let age = age?;
            if age.is_adult() {
                return None;
            }
            let dist = distance(here, tile.tile);
            (dist <= radius).then_some((entity, dist))
        })
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
}

fn nearest_rabbit(
    here: IVec2,
    radius: f32,
    rabbits: &Query<(Entity, &TilePosition, Option<&Age>), With<Rabbit>>,
) -> Option<(Entity, f32)> {
    rabbits
        .iter()
        .filter_map(|(entity, tile, _)| {
            let dist = distance(here, tile.tile);
            (dist <= radius).then_some((entity, dist))
        })
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
}

fn nearest_adult_deer(
    here: IVec2,
    radius: f32,
    deer: &Query<(Entity, &TilePosition, Option<&Age>), With<Deer>>,
) -> Option<(Entity, f32)> {
    deer.iter()
        .filter_map(|(entity, tile, age)| {
            if let Some(age) = age {
                if !age.is_adult() {
                    return None;
                }
            }
            let dist = distance(here, tile.tile);
            (dist <= radius).then_some((entity, dist))
        })
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
}

fn filter_out_graze(actions: &mut Vec<UtilityScore>) {
    actions.retain(|a| !matches!(a.action_type, ActionType::Graze { .. }));
}

#[allow(clippy::too_many_arguments)]
pub fn evaluate_bear_actions(
    _entity: Entity,
    position: &TilePosition,
    thirst: &Thirst,
    hunger: &Hunger,
    energy: &Energy,
    behavior_config: &BehaviorConfig,
    world_loader: &WorldLoader,
    fear_state: Option<&FearState>,
    carcasses: &Query<(Entity, &TilePosition, &Carcass)>,
    deer: &Query<(Entity, &TilePosition, Option<&Age>), With<Deer>>,
    vegetation: &ResourceGrid,
) -> Vec<UtilityScore> {
    // Bears are omnivores - use generalist diet with grass preference
    let diet = HerbivoreDiet::new(0.7, 0.4, 8.0);
    let mut actions = herbivore_toolkit::evaluate_core_actions(
        position,
        thirst,
        hunger,
        energy,
        behavior_config,
        world_loader,
        vegetation,
        fear_state,
        &diet,
    );

    let hunger_value = hunger_norm(hunger);
    if hunger_value >= behavior_config.hunger_threshold {
        if let Some((carcass_entity, dist, nutrition)) =
            best_carcass(position.tile, behavior_config.food_search_radius, carcasses)
        {
            let distance_factor = (behavior_config.food_search_radius as f32 - dist)
                / behavior_config.food_search_radius as f32;
            if distance_factor > 0.0 {
                let nutrition_factor = (nutrition / 160.0).min(1.0);
                let utility = ((0.3 + hunger_value * 0.6) * distance_factor
                    + nutrition_factor * 0.1)
                    .clamp(0.0, 0.95);
                actions.push(UtilityScore {
                    action_type: ActionType::Scavenge {
                        carcass: carcass_entity,
                    },
                    utility,
                    priority: SCAVENGE_PRIORITY_BEAR,
                });
            }
        }
    }

    if hunger_value >= 0.6 && energy_norm(energy) > 0.35 {
        if let Some((fawn_entity, dist)) = nearest_fawn(position.tile, BEAR_FAWN_RADIUS, deer) {
            let distance_factor = (BEAR_FAWN_RADIUS - dist) / BEAR_FAWN_RADIUS;
            if distance_factor > 0.0 {
                let utility = ((0.25 + hunger_value * 0.45) * distance_factor).clamp(0.0, 0.9);
                actions.push(UtilityScore {
                    action_type: ActionType::Hunt { prey: fawn_entity },
                    utility,
                    priority: HUNT_PRIORITY_BEAR,
                });
            }
        }
    }

    actions
}

#[allow(clippy::too_many_arguments)]
pub fn evaluate_fox_actions(
    _entity: Entity,
    position: &TilePosition,
    thirst: &Thirst,
    hunger: &Hunger,
    energy: &Energy,
    behavior_config: &BehaviorConfig,
    world_loader: &WorldLoader,
    fear_state: Option<&FearState>,
    carcasses: &Query<(Entity, &TilePosition, &Carcass)>,
    rabbits: &Query<(Entity, &TilePosition, Option<&Age>), With<Rabbit>>,
    vegetation: &ResourceGrid,
) -> Vec<UtilityScore> {
    // Foxes are carnivores but can eat some plant matter - use minimal plant diet
    let diet = HerbivoreDiet::new(0.2, 0.1, 5.0);
    let mut actions = herbivore_toolkit::evaluate_core_actions(
        position,
        thirst,
        hunger,
        energy,
        behavior_config,
        world_loader,
        vegetation,
        fear_state,
        &diet,
    );
    // Predators shouldn't nibble grass; strip graze options inherited from core helper.
    filter_out_graze(&mut actions);

    let hunger_value = hunger_norm(hunger);
    if hunger_value >= behavior_config.hunger_threshold && energy_norm(energy) > 0.3 {
        if let Some((rabbit, dist)) = nearest_rabbit(position.tile, FOX_HUNT_RADIUS, rabbits) {
            let distance_factor = (FOX_HUNT_RADIUS - dist) / FOX_HUNT_RADIUS;
            if distance_factor > 0.0 {
                let utility = ((0.35 + hunger_value * 0.45) * distance_factor).clamp(0.0, 0.95);
                actions.push(UtilityScore {
                    action_type: ActionType::Hunt { prey: rabbit },
                    utility,
                    priority: HUNT_PRIORITY_FOX,
                });
            }
        }
    }

    if hunger_value >= 0.4 {
        if let Some((carcass_entity, dist, nutrition)) =
            best_carcass(position.tile, behavior_config.food_search_radius, carcasses)
        {
            let distance_factor = (behavior_config.food_search_radius as f32 - dist)
                / behavior_config.food_search_radius as f32;
            if distance_factor > 0.0 {
                let nutrition_factor = (nutrition / 90.0).min(1.0);
                let utility = ((0.25 + hunger_value * 0.45) * distance_factor
                    + nutrition_factor * 0.1)
                    .clamp(0.0, 0.85);
                actions.push(UtilityScore {
                    action_type: ActionType::Scavenge {
                        carcass: carcass_entity,
                    },
                    utility,
                    priority: SCAVENGE_PRIORITY_FOX,
                });
            }
        }
    }

    actions
}

#[allow(clippy::too_many_arguments)]
pub fn evaluate_wolf_actions(
    _entity: Entity,
    position: &TilePosition,
    thirst: &Thirst,
    hunger: &Hunger,
    energy: &Energy,
    behavior_config: &BehaviorConfig,
    world_loader: &WorldLoader,
    fear_state: Option<&FearState>,
    carcasses: &Query<(Entity, &TilePosition, &Carcass)>,
    deer: &Query<(Entity, &TilePosition, Option<&Age>), With<Deer>>,
    vegetation: &ResourceGrid,
) -> Vec<UtilityScore> {
    // Wolves are obligate carnivores - use very minimal plant diet
    let diet = HerbivoreDiet::new(0.1, 0.05, 3.0);
    let mut actions = herbivore_toolkit::evaluate_core_actions(
        position,
        thirst,
        hunger,
        energy,
        behavior_config,
        world_loader,
        vegetation,
        fear_state,
        &diet,
    );
    // Wolves are obligate carnivores as well.
    filter_out_graze(&mut actions);

    let hunger_value = hunger_norm(hunger);
    if hunger_value >= 0.45 && energy_norm(energy) > 0.4 {
        if let Some((deer_entity, dist)) = nearest_adult_deer(position.tile, WOLF_HUNT_RADIUS, deer)
        {
            let distance_factor = (WOLF_HUNT_RADIUS - dist) / WOLF_HUNT_RADIUS;
            if distance_factor > 0.0 {
                let utility = ((0.4 + hunger_value * 0.45) * distance_factor).clamp(0.0, 0.98);
                actions.push(UtilityScore {
                    action_type: ActionType::Hunt { prey: deer_entity },
                    utility,
                    priority: HUNT_PRIORITY_WOLF,
                });
            }
        }
    }

    if hunger_value >= 0.35 {
        if let Some((carcass_entity, dist, nutrition)) =
            best_carcass(position.tile, behavior_config.food_search_radius, carcasses)
        {
            let distance_factor = (behavior_config.food_search_radius as f32 - dist)
                / behavior_config.food_search_radius as f32;
            if distance_factor > 0.0 {
                let nutrition_factor = (nutrition / 140.0).min(1.0);
                let utility = ((0.3 + hunger_value * 0.5) * distance_factor
                    + nutrition_factor * 0.1)
                    .clamp(0.0, 0.9);
                actions.push(UtilityScore {
                    action_type: ActionType::Scavenge {
                        carcass: carcass_entity,
                    },
                    utility,
                    priority: SCAVENGE_PRIORITY_WOLF,
                });
            }
        }
    }

    actions
}
