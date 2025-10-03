/// Reproduction components and systems (modular, species-agnostic with rabbit wrappers)
use bevy::prelude::*;
use rand::Rng;

use crate::entities::{Rabbit, TilePosition};
use crate::entities::stats::{Hunger, Thirst, Energy, Health};

// -----------------------------
// Config
// -----------------------------
mod config {
    use bevy::prelude::Resource;

    #[derive(Resource, Debug, Clone, Copy)]
    pub struct ReproductionConfig {
        pub maturity_ticks: u32,
        pub gestation_ticks: u32,
        pub mating_cooldown_ticks: u32,
        pub postpartum_cooldown_ticks: u32,
        pub litter_size_range: (u8, u8),
        pub mating_search_radius: i32,
        pub well_fed_hunger_norm: f32,
        pub well_fed_thirst_norm: f32,
        pub well_fed_required_ticks: u32,
        pub matching_interval_ticks: u32,
        pub mating_duration_ticks: u32,
        pub min_energy_norm: f32,
        pub min_health_norm: f32,
    }
    pub use ReproductionConfig as Config;
}

// -----------------------------
// Components
// -----------------------------
mod components {
    use bevy::prelude::*;

    #[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Sex { Male, Female }

    #[derive(Component, Debug, Clone, Copy)]
    pub struct Age {
        pub ticks_alive: u64,
        pub mature_at_ticks: u32,
    }
    impl Age { pub fn is_adult(&self) -> bool { self.ticks_alive >= self.mature_at_ticks as u64 } }

    #[derive(Component, Debug, Clone, Copy)]
    pub struct ReproductionCooldown { pub remaining_ticks: u32 }
    impl Default for ReproductionCooldown { fn default() -> Self { Self { remaining_ticks: 0 } } }

    #[derive(Component, Debug, Clone, Copy)]
    pub struct Pregnancy {
        pub remaining_ticks: u32,
        pub litter_size: u8,
        pub father: Option<Entity>,
    }

    #[derive(Component, Debug, Clone, Copy)]
    pub struct Mother(pub Entity);

    #[derive(Component, Debug, Clone, Copy)]
    pub struct WellFedStreak { pub ticks: u32 }
    impl Default for WellFedStreak { fn default() -> Self { Self { ticks: 0 } } }

    #[derive(Component, Debug, Clone, Copy)]
    pub struct MatingIntent {
        pub partner: Entity,
        pub meeting_tile: IVec2,
        pub duration_ticks: u32,
    }

    pub use Age as AgeC;
}

// -----------------------------
// Systems (generic helpers + rabbit wrappers)
// -----------------------------
mod systems {
    use super::*;
    use super::components::*;
    use super::config::ReproductionConfig;

    pub fn update_age_and_wellfed_system(
        mut query: Query<(&mut Age, &Hunger, &Thirst, Option<&mut WellFedStreak>), With<Rabbit>>,
        config: Res<ReproductionConfig>,
    ) {
        for (mut age, hunger, thirst, wellfed_opt) in query.iter_mut() {
            age.ticks_alive = age.ticks_alive.saturating_add(1);

            if let Some(mut wellfed) = wellfed_opt {
                let h = hunger.0.normalized();
                let t = thirst.0.normalized();
                if h <= (*config).well_fed_hunger_norm && t <= (*config).well_fed_thirst_norm {
                    wellfed.ticks = wellfed.ticks.saturating_add(1);
                } else {
                    wellfed.ticks = wellfed.ticks.saturating_sub(1);
                }
            }
        }
    }

    fn is_eligible(
        age: &Age,
        cooldown: &ReproductionCooldown,
        energy: &Energy,
        health: &Health,
        wellfed: &WellFedStreak,
        config: &ReproductionConfig,
    ) -> bool {
        age.is_adult()
            && cooldown.remaining_ticks == 0
            && energy.0.normalized() >= config.min_energy_norm
            && health.0.normalized() >= config.min_health_norm
            && wellfed.ticks >= config.well_fed_required_ticks
    }

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

    pub fn rabbit_mate_matching_system(
        mut commands: Commands,
        config: Res<ReproductionConfig>,
        rabbits: Query<(
            Entity, &TilePosition, &Age, &ReproductionCooldown, &Energy, &Health, &WellFedStreak,
            Option<&Pregnancy>, Option<&Sex>, Option<&MatingIntent>
        ), With<Rabbit>>,
        tick: Res<crate::simulation::SimulationTick>,
    ) {
        if tick.0 % (*config).matching_interval_ticks as u64 != 0 { return; }

        let mut males: Vec<(Entity, IVec2, &Age, &ReproductionCooldown, &Energy, &Health, &WellFedStreak)> = Vec::new();
        let mut females: Vec<(Entity, IVec2, &Age, &ReproductionCooldown, &Energy, &Health, &WellFedStreak)> = Vec::new();

        for (e, pos, age, cd, en, hp, wf, preg_opt, sex_opt, intent_opt) in rabbits.iter() {
            let Some(sex) = sex_opt.copied() else { continue; };
            if preg_opt.is_some() || intent_opt.is_some() { continue; }
            if !is_eligible(age, cd, en, hp, wf, &*config) { continue; }
            match sex {
                Sex::Male => males.push((e, pos.tile, age, cd, en, hp, wf)),
                Sex::Female => females.push((e, pos.tile, age, cd, en, hp, wf)),
            }
        }

        if males.is_empty() || females.is_empty() { return; }
        use std::collections::HashSet;
        let mut used_males: HashSet<Entity> = HashSet::new();
        let radius2 = ((*config).mating_search_radius * (*config).mating_search_radius) as i32;

        for (female_e, fpos, _fa, _fcd, _fen, _fhp, _fwf) in females.into_iter() {
            let mut best: Option<(usize, i32)> = None;
            for (idx, (me, mpos, ..)) in males.iter().enumerate() {
                if used_males.contains(me) { continue; }
                let dx = fpos.x - mpos.x;
                let dy = fpos.y - mpos.y;
                let d2 = dx*dx + dy*dy;
                if d2 <= radius2 {
                    if best.map(|(_, bd2)| d2 < bd2).unwrap_or(true) { best = Some((idx, d2)); }
                }
            }
            let Some((mi, _)) = best else { continue; };
            let male_e = males[mi].0;
            used_males.insert(male_e);

            let meet = fpos;
            let duration = (*config).mating_duration_ticks;
            commands.entity(female_e).insert(MatingIntent { partner: male_e, meeting_tile: meet, duration_ticks: duration });
            commands.entity(male_e).insert(MatingIntent { partner: female_e, meeting_tile: meet, duration_ticks: duration });
            info!("🐇💞 Pair formed: female {:?} with male {:?} -> rendezvous at {:?}", female_e, male_e, meet);
        }
    }

    // Shared birth helper
    fn birth_common<E: Component>(
        commands: &mut Commands,
        mothers: &mut Query<(Entity, &TilePosition, &mut Pregnancy), With<E>>,
        maturity_ticks: u32,
        mut spawn_fn: impl FnMut(&mut Commands, String, IVec2) -> Entity,
        species_emoji: &str,
        baby_prefix: &str,
    ) {
        let mut to_clear: Vec<Entity> = Vec::new();
        for (mother, pos, mut preg) in mothers.iter_mut() {
            if preg.remaining_ticks == 0 {
                let litter = preg.litter_size as usize;
                for i in 0..litter {
                    let name = format!("{}_{}", baby_prefix, i);
                    let baby = spawn_fn(commands, name, pos.tile);
                    let mut rng = rand::thread_rng();
                    let sex = if rng.gen_bool(0.5) { Sex::Male } else { Sex::Female };
                    commands.entity(baby)
                        .insert(sex)
                        .insert(Age { ticks_alive: 0, mature_at_ticks: maturity_ticks })
                        .insert(WellFedStreak::default())
                        .insert(ReproductionCooldown::default())
                        .insert(Mother(mother));
                }
                info!("{} Birth: mother {:?} gave birth to {} offspring at {:?}", species_emoji, mother, preg.litter_size, pos.tile);
                to_clear.push(mother);
            }
        }
        for e in to_clear { commands.entity(e).remove::<Pregnancy>(); }
    }

    pub fn rabbit_birth_system(
        mut commands: Commands,
        mut mothers: Query<(Entity, &TilePosition, &mut Pregnancy), With<Rabbit>>,
        config: Res<ReproductionConfig>,
    ) {
        birth_common::<Rabbit>(&mut commands, &mut mothers, config.maturity_ticks, |cmds, name, pos| {
            crate::entities::entity_types::spawn_rabbit(cmds, name, pos)
        }, "🐇🍼", "Kit");
    }

    pub fn deer_mate_matching_system(
        mut commands: Commands,
        config: Res<ReproductionConfig>,
        deer: Query<(
            Entity, &TilePosition, &Age, &ReproductionCooldown, &Energy, &Health, &WellFedStreak,
            Option<&Pregnancy>, Option<&Sex>, Option<&MatingIntent>
        ), With<crate::entities::Deer>>,
        tick: Res<crate::simulation::SimulationTick>,
    ) {
        if tick.0 % (*config).matching_interval_ticks as u64 != 0 { return; }

        use std::collections::HashSet;
        let mut males: Vec<(Entity, IVec2, &Age, &ReproductionCooldown, &Energy, &Health, &WellFedStreak)> = Vec::new();
        let mut females: Vec<(Entity, IVec2, &Age, &ReproductionCooldown, &Energy, &Health, &WellFedStreak)> = Vec::new();
        for (e, pos, age, cd, en, hp, wf, preg_opt, sex_opt, intent_opt) in deer.iter() {
            let Some(sex) = sex_opt.copied() else { continue; };
            if preg_opt.is_some() || intent_opt.is_some() { continue; }
            if !is_eligible(age, cd, en, hp, wf, &*config) { continue; }
            match sex {
                Sex::Male => males.push((e, pos.tile, age, cd, en, hp, wf)),
                Sex::Female => females.push((e, pos.tile, age, cd, en, hp, wf)),
            }
        }
        if males.is_empty() || females.is_empty() { return; }
        let mut used_males: HashSet<Entity> = HashSet::new();
        let radius2 = ((*config).mating_search_radius * (*config).mating_search_radius) as i32;
        for (female_e, fpos, ..) in females.into_iter() {
            let mut best: Option<(usize, i32)> = None;
            for (idx, (me, mpos, ..)) in males.iter().enumerate() {
                if used_males.contains(me) { continue; }
                let d2 = { let d = fpos - *mpos; let a = d.x*d.x + d.y*d.y; a };
                if d2 <= radius2 {
                    if best.map(|(_, bd2)| d2 < bd2).unwrap_or(true) { best = Some((idx, d2)); }
                }
            }
            let Some((mi, _)) = best else { continue; };
            let male_e = males[mi].0; used_males.insert(male_e);
            let meet = fpos; let duration = (*config).mating_duration_ticks;
            commands.entity(female_e).insert(MatingIntent { partner: male_e, meeting_tile: meet, duration_ticks: duration });
            commands.entity(male_e).insert(MatingIntent { partner: female_e, meeting_tile: meet, duration_ticks: duration });
            info!("🦌💞 Pair formed: female {:?} with male {:?} -> rendezvous at {:?}", female_e, male_e, meet);
        }
    }

    pub fn deer_birth_system(
        mut commands: Commands,
        mut mothers: Query<(Entity, &TilePosition, &mut Pregnancy), With<crate::entities::Deer>>,
        config: Res<ReproductionConfig>,
    ) {
        birth_common::<crate::entities::Deer>(&mut commands, &mut mothers, config.maturity_ticks, |cmds, name, pos| {
            crate::entities::entity_types::spawn_deer(cmds, name, pos)
        }, "🦌🍼", "Fawn");
    }
}

// -----------------------------
// Re-exports
// -----------------------------
pub use config::ReproductionConfig;
pub use components::{Sex, Age, ReproductionCooldown, Pregnancy, WellFedStreak, Mother, MatingIntent};
pub use systems::{
    update_age_and_wellfed_system,
    tick_reproduction_timers_system,
    rabbit_mate_matching_system,
    rabbit_birth_system,
    deer_mate_matching_system,
    deer_birth_system,
};
