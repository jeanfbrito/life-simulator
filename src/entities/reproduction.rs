/// Reproduction components and systems for herbivores (initially Rabbits)
use bevy::prelude::*;
use rand::Rng;

use crate::entities::{Rabbit, TilePosition};
use crate::entities::stats::{Hunger, Thirst, Energy, Health};
use crate::entities::types::rabbit::RabbitReproductionConfig;

/// Sex of an entity
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sex { Male, Female }

/// Age tracking; adulthood when ticks_alive >= mature_at_ticks
#[derive(Component, Debug, Clone, Copy)]
pub struct Age {
    pub ticks_alive: u64,
    pub mature_at_ticks: u32,
}
impl Age {
    pub fn is_adult(&self) -> bool { self.ticks_alive >= self.mature_at_ticks as u64 }
}

/// Reproduction cooldown (prevents immediate re-mating)
#[derive(Component, Debug, Clone, Copy)]
pub struct ReproductionCooldown { pub remaining_ticks: u32 }
impl Default for ReproductionCooldown { fn default() -> Self { Self { remaining_ticks: 0 } } }

/// Pregnancy state (females only)
#[derive(Component, Debug, Clone, Copy)]
pub struct Pregnancy {
    pub remaining_ticks: u32,
    pub litter_size: u8,
    pub father: Option<Entity>,
}

/// Mother reference for juveniles to enable following behavior
#[derive(Component, Debug, Clone, Copy)]
pub struct Mother(pub Entity);

/// Tracks sustained well-fed state
#[derive(Component, Debug, Clone, Copy)]
pub struct WellFedStreak { pub ticks: u32 }
impl Default for WellFedStreak { fn default() -> Self { Self { ticks: 0 } } }

/// Intent to rendezvous and mate with a partner at a meeting tile
#[derive(Component, Debug, Clone, Copy)]
pub struct MatingIntent {
    pub partner: Entity,
    pub meeting_tile: IVec2,
    pub duration_ticks: u32,
}

/// System: increment Age each tick; update WellFedStreak based on hunger/thirst
pub fn update_age_and_wellfed_system(
    mut query: Query<(&mut Age, &Hunger, &Thirst, Option<&mut WellFedStreak>), With<Rabbit>>,
    config: Res<RabbitReproductionConfig>,
) {
    for (mut age, hunger, thirst, wellfed_opt) in query.iter_mut() {
        age.ticks_alive = age.ticks_alive.saturating_add(1);

        if let Some(mut wellfed) = wellfed_opt {
            let h = hunger.0.normalized();
            let t = thirst.0.normalized();
            if h <= (*config).well_fed_hunger_norm && t <= (*config).well_fed_thirst_norm {
                wellfed.ticks = wellfed.ticks.saturating_add(1);
            } else {
                // Decay instead of reset to allow progress to persist across brief lapses
                wellfed.ticks = wellfed.ticks.saturating_sub(1);
            }
        }
    }
}

/// Helper predicate: is rabbit eligible to mate (general checks)
fn is_eligible(
    age: &Age,
    cooldown: &ReproductionCooldown,
    energy: &Energy,
    health: &Health,
    wellfed: &WellFedStreak,
    config: &RabbitReproductionConfig,
) -> bool {
    age.is_adult()
        && cooldown.remaining_ticks == 0
        && energy.0.normalized() >= config.min_energy_norm
        && health.0.normalized() >= config.min_health_norm
        && wellfed.ticks >= config.well_fed_required_ticks
}

/// System: decrement cooldowns and pregnancy timers (except birth, handled separately)
pub fn tick_reproduction_timers_system(
    mut cooldowns: Query<&mut ReproductionCooldown>,
    mut pregnancies: Query<&mut Pregnancy>,
) {
    for mut cd in cooldowns.iter_mut() {
        if cd.remaining_ticks > 0 { cd.remaining_ticks -= 1; }
    }
    for mut preg in pregnancies.iter_mut() {
        if preg.remaining_ticks > 0 { preg.remaining_ticks -= 1; }
    }
}

/// System: MVP mate matching (instant pairing if within radius)
pub fn rabbit_mate_matching_system(
    mut commands: Commands,
    config: Res<RabbitReproductionConfig>,
    rabbits: Query<(
        Entity,
        &TilePosition,
        &Age,
        &ReproductionCooldown,
        &Energy,
        &Health,
        &WellFedStreak,
        Option<&Pregnancy>,
        Option<&Sex>,
        Option<&MatingIntent>,
    ), With<Rabbit>>,
    tick: Res<crate::simulation::SimulationTick>,
) {
    // Run only every interval to reduce cost
    if tick.0 % (*config).matching_interval_ticks as u64 != 0 { return; }

    // Collect eligible males and females
    let mut males: Vec<(Entity, IVec2, &Age, &ReproductionCooldown, &Energy, &Health, &WellFedStreak)> = Vec::new();
    let mut females: Vec<(Entity, IVec2, &Age, &ReproductionCooldown, &Energy, &Health, &WellFedStreak)> = Vec::new();

    for (e, pos, age, cd, en, hp, wf, preg_opt, sex_opt, intent_opt) in rabbits.iter() {
        let Some(sex) = sex_opt.copied() else { continue; };
        // Skip if already pregnant or already has a mating intent
        if preg_opt.is_some() || intent_opt.is_some() { continue; }
        if !is_eligible(age, cd, en, hp, wf, &*config) { continue; }
        match sex {
            Sex::Male => males.push((e, pos.tile, age, cd, en, hp, wf)),
            Sex::Female => females.push((e, pos.tile, age, cd, en, hp, wf)),
        }
    }

    if males.is_empty() || females.is_empty() { return; }

    // Greedy pairing: for each female, find nearest available male within radius
    use std::collections::HashSet;
    let mut used_males: HashSet<Entity> = HashSet::new();
    let radius2 = ((*config).mating_search_radius * (*config).mating_search_radius) as i32;
    let _rng = rand::thread_rng();

    for (female_e, fpos, _fa, _fcd, _fen, _fhp, _fwf) in females.into_iter() {
        // Find nearest available male
        let mut best: Option<(usize, i32)> = None; // (index in males, dist2)
        for (idx, (me, mpos, ..)) in males.iter().enumerate() {
            if used_males.contains(me) { continue; }
            let dx = fpos.x - mpos.x;
            let dy = fpos.y - mpos.y;
            let d2 = dx*dx + dy*dy;
            if d2 <= radius2 {
                if best.map(|(_, bd2)| d2 < bd2).unwrap_or(true) {
                    best = Some((idx, d2));
                }
            }
        }
        let Some((mi, _)) = best else { continue; };
        let male_e = males[mi].0;
        used_males.insert(male_e);

        // Issue mating intents for a rendezvous at the female's current tile
        let meet = fpos;
        let duration = (*config).mating_duration_ticks;
        commands.entity(female_e).insert(MatingIntent { partner: male_e, meeting_tile: meet, duration_ticks: duration });
        commands.entity(male_e).insert(MatingIntent { partner: female_e, meeting_tile: meet, duration_ticks: duration });

        info!("🐇💞 Pair formed: female {:?} with male {:?} -> rendezvous at {:?}", female_e, male_e, meet);
    }
}

/// System: handle births when pregnancy timer reaches zero
pub fn rabbit_birth_system(
    mut commands: Commands,
    mut mothers: Query<(Entity, &TilePosition, &mut Pregnancy), With<Rabbit>>,
) {
    let mut to_clear: Vec<Entity> = Vec::new();

    for (mother, pos, mut preg) in mothers.iter_mut() {
        if preg.remaining_ticks == 0 {
            // Spawn kits
            let litter = preg.litter_size as usize;
            for i in 0..litter {
                // Use spawn_rabbit then set juvenile age and random sex
                let name = format!("Kit_{}", i);
                let kit = crate::entities::entity_types::spawn_rabbit(&mut commands, name, pos.tile);
                // Assign juvenile age and sex
                let mut rng = rand::thread_rng();
                let sex = if rng.gen_bool(0.5) { Sex::Male } else { Sex::Female };
                // Default maturity: reuse rabbit config
                let cfg = crate::entities::types::rabbit::RabbitBehavior::reproduction_config();
                commands.entity(kit)
                    .insert(sex)
                    .insert(Age { ticks_alive: 0, mature_at_ticks: cfg.maturity_ticks })
                    .insert(WellFedStreak::default())
                    .insert(ReproductionCooldown::default())
                    .insert(Mother(mother));
            }
            info!("🐇🍼 Birth: mother {:?} gave birth to {} kits at {:?}", mother, preg.litter_size, pos.tile);
            to_clear.push(mother);
        }
    }

    for e in to_clear { commands.entity(e).remove::<Pregnancy>(); }
}