use bevy::prelude::*;

use crate::ai::action::ActionType;
use crate::ai::behaviors::{
    evaluate_drinking_behavior, evaluate_eating_behavior, evaluate_follow_behavior,
    evaluate_grazing_behavior, evaluate_resting_behavior,
};
use crate::ai::planner::UtilityScore;
use crate::entities::reproduction::{Age, MatingIntent, Mother, ReproductionConfig};
use crate::entities::stats::{Energy, Hunger, Thirst};
use crate::entities::{BehaviorConfig, FearState, TilePosition};
use crate::vegetation::resource_grid::ResourceGrid;
use crate::world_loader::WorldLoader;

/// Evaluate the baseline herbivore actions (drink, eat, rest, graze).
///
/// Species-specific modules can call this helper and then push any additional
/// actions (social, unique behaviours) afterwards.
pub fn evaluate_core_actions(
    position: &TilePosition,
    thirst: &Thirst,
    hunger: &Hunger,
    energy: &Energy,
    behavior_config: &BehaviorConfig,
    world_loader: &WorldLoader,
    resource_grid: &ResourceGrid,
    fear_state: Option<&FearState>,
) -> Vec<UtilityScore> {
    let mut actions = Vec::new();

    // Get fear utility modifier if fear state is available
    let fear_modifier = fear_state.map_or(1.0, |f| f.get_utility_modifier());

    if let Some(drink) = evaluate_drinking_behavior(
        position,
        thirst,
        world_loader,
        behavior_config.thirst_threshold,
        behavior_config.water_search_radius,
    ) {
        actions.push(drink);
    }

    if let Some(eat) = evaluate_eating_behavior(
        position,
        hunger,
        world_loader,
        resource_grid,
        behavior_config.hunger_threshold,
        behavior_config.food_search_radius,
        behavior_config.foraging_strategy,
    ) {
        actions.push(eat);
    }

    if let Some(rest) =
        evaluate_resting_behavior(position, energy, behavior_config.energy_threshold)
    {
        actions.push(rest);
    }

    if let Some(graze) =
        evaluate_grazing_behavior(position, world_loader, behavior_config.graze_range)
    {
        actions.push(graze);
    }

    // Apply fear modifiers to action utilities
    if fear_modifier < 1.0 {
        for action in &mut actions {
            // Reduce feeding-related utilities under fear
            match action.action_type {
                ActionType::Graze { .. } => {
                    action.utility *= fear_modifier;
                }
                // Drinking and resting utilities might be less affected
                ActionType::DrinkWater { .. } | ActionType::Rest { .. } => {
                    action.utility *= (1.0 + fear_modifier) / 2.0; // Moderate reduction
                }
                // Other actions (wandering, following, mating, etc.) are not affected
                _ => {}
            }

            // Log fear modification for debugging
            if fear_modifier < 0.9 {
                debug!(
                    "🦊 Fear modified action utility: {:?} {:.2} → {:.2} (modifier: {:.2})",
                    action.action_type,
                    action.utility / fear_modifier,
                    action.utility,
                    fear_modifier
                );
            }
        }
    }

    actions
}

/// Configuration for follow-the-mother behaviour.
#[derive(Debug, Clone, Copy)]
pub struct FollowConfig {
    pub stop_distance: i32,
    pub max_distance: i32,
}

/// Attempt to add a follow-mother action for juvenile herbivores.
///
/// Returns true if an action was added.
pub fn maybe_add_follow_mother(
    actions: &mut Vec<UtilityScore>,
    entity: Entity,
    position: &TilePosition,
    hunger: &Hunger,
    thirst: &Thirst,
    energy: &Energy,
    behavior_config: &BehaviorConfig,
    age: Option<&Age>,
    mother: Option<&Mother>,
    mother_position: Option<IVec2>,
    follow_cfg: FollowConfig,
) -> bool {
    let Some(age) = age else {
        return false;
    };
    if age.is_adult() {
        return false;
    }

    let Some(mother) = mother else {
        return false;
    };
    let Some(mother_pos) = mother_position else {
        return false;
    };

    let hunger_ok = hunger.0.normalized() < behavior_config.hunger_threshold;
    let thirst_ok = thirst.0.normalized() < behavior_config.thirst_threshold;
    let energy_ok = energy.0.normalized() > behavior_config.energy_threshold;
    if !(hunger_ok && thirst_ok && energy_ok) {
        return false;
    }

    let slice = [(mother.0, mother_pos)];
    if let Some(follow) = evaluate_follow_behavior(
        entity,
        position,
        &slice,
        follow_cfg.stop_distance,
        follow_cfg.max_distance,
    ) {
        actions.push(follow);
        true
    } else {
        false
    }
}

/// Parameters controlling mate-action queuing for a species.
#[derive(Debug, Clone, Copy)]
pub struct MateActionParams {
    pub utility: f32,
    pub priority: i32,
    /// Extra slack applied to hunger/thirst thresholds compared to the
    /// reproduction config requirements.
    pub threshold_margin: f32,
    /// Extra slack applied to minimum energy requirement.
    pub energy_margin: f32,
}

/// Add a mate action if the entity currently has a mating intent and meets
/// the reproduction requirements. Returns true if an action was added.
pub fn maybe_add_mate_action(
    actions: &mut Vec<UtilityScore>,
    mating_intent: Option<&MatingIntent>,
    repro_cfg: Option<&ReproductionConfig>,
    thirst: &Thirst,
    hunger: &Hunger,
    energy: &Energy,
    params: MateActionParams,
) -> bool {
    let Some(intent) = mating_intent else {
        return false;
    };
    let Some(cfg) = repro_cfg else {
        return false;
    };

    let thirst_level = thirst.0.normalized();
    let hunger_level = hunger.0.normalized();
    let energy_level = energy.0.normalized();

    let thirst_safe = thirst_level <= cfg.well_fed_thirst_norm + params.threshold_margin;
    let hunger_safe = hunger_level <= cfg.well_fed_hunger_norm + params.threshold_margin;
    let energy_safe = energy_level >= (cfg.min_energy_norm + params.energy_margin).min(1.0);

    if thirst_safe && hunger_safe && energy_safe {
        actions.push(UtilityScore {
            action_type: ActionType::Mate {
                partner: intent.partner,
                meeting_tile: intent.meeting_tile,
                duration_ticks: intent.duration_ticks,
            },
            utility: params.utility,
            priority: params.priority,
        });
        true
    } else {
        false
    }
}
