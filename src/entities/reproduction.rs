/// Reproduction components and systems (modular, species-agnostic with rabbit wrappers)
use bevy::prelude::*;
use rand::Rng;

use crate::entities::stats::{Energy, Health, Hunger, Thirst};
use crate::entities::{Rabbit, Raccoon, TilePosition};

// -----------------------------
// Config
// -----------------------------
mod config {
    use bevy::prelude::{Component, Resource};

    #[derive(Resource, Component, Debug, Clone, Copy)]
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
    pub enum Sex {
        Male,
        Female,
    }

    #[derive(Component, Debug, Clone, Copy)]
    pub struct Age {
        pub ticks_alive: u64,
        pub mature_at_ticks: u32,
    }
    impl Age {
        pub fn is_adult(&self) -> bool {
            self.ticks_alive >= self.mature_at_ticks as u64
        }
    }

    #[derive(Component, Debug, Clone, Copy)]
    pub struct ReproductionCooldown {
        pub remaining_ticks: u32,
    }
    impl Default for ReproductionCooldown {
        fn default() -> Self {
            Self { remaining_ticks: 0 }
        }
    }

    #[derive(Component, Debug, Clone, Copy)]
    pub struct Pregnancy {
        pub remaining_ticks: u32,
        pub litter_size: u8,
        pub father: Option<Entity>,
    }

    #[derive(Component, Debug, Clone, Copy)]
    pub struct Mother(pub Entity);

    #[derive(Component, Debug, Clone, Copy)]
    pub struct WellFedStreak {
        pub ticks: u32,
    }
    impl Default for WellFedStreak {
        fn default() -> Self {
            Self { ticks: 0 }
        }
    }

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
    use super::components::*;
    use super::config::ReproductionConfig;
    use super::*;

    pub fn update_age_and_wellfed_system(
        mut query: Query<(
            &mut Age,
            &Hunger,
            &Thirst,
            Option<&mut WellFedStreak>,
            Option<&ReproductionConfig>,
        )>,
    ) {
        for (mut age, hunger, thirst, wellfed_opt, config_opt) in query.iter_mut() {
            age.ticks_alive = age.ticks_alive.saturating_add(1);

            if let (Some(mut wellfed), Some(cfg)) = (wellfed_opt, config_opt) {
                let h = hunger.0.normalized();
                let t = thirst.0.normalized();
                if h <= cfg.well_fed_hunger_norm && t <= cfg.well_fed_thirst_norm {
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
            if cd.remaining_ticks > 0 {
                cd.remaining_ticks -= 1;
            }
        }
        for mut preg in pregnancies.iter_mut() {
            if preg.remaining_ticks > 0 {
                preg.remaining_ticks -= 1;
            }
        }
    }

    fn mate_matching_system_inner<M: Component>(
        commands: &mut Commands,
        animals: &Query<
            (
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
                &ReproductionConfig,
            ),
            With<M>,
        >,
        current_tick: u64,
        species_emoji: &str,
    ) {
        use std::collections::HashSet;

        let mut sampled_interval: Option<u64> = None;
        let mut males: Vec<(
            Entity,
            IVec2,
            &Age,
            &ReproductionCooldown,
            &Energy,
            &Health,
            &WellFedStreak,
            &ReproductionConfig,
        )> = Vec::new();
        let mut females: Vec<(
            Entity,
            IVec2,
            &Age,
            &ReproductionCooldown,
            &Energy,
            &Health,
            &WellFedStreak,
            &ReproductionConfig,
        )> = Vec::new();

        for (entity, pos, age, cd, en, hp, wf, preg_opt, sex_opt, intent_opt, cfg) in animals.iter()
        {
            if sampled_interval.is_none() {
                sampled_interval = Some(cfg.matching_interval_ticks as u64);
                if current_tick % cfg.matching_interval_ticks as u64 != 0 {
                    return;
                }
            }

            let Some(sex) = sex_opt.copied() else {
                continue;
            };
            if preg_opt.is_some() || intent_opt.is_some() {
                continue;
            }
            if !is_eligible(age, cd, en, hp, wf, cfg) {
                continue;
            }

            match sex {
                Sex::Male => males.push((entity, pos.tile, age, cd, en, hp, wf, cfg)),
                Sex::Female => females.push((entity, pos.tile, age, cd, en, hp, wf, cfg)),
            }
        }

        if males.is_empty() || females.is_empty() {
            return;
        }

        let mut used_males: HashSet<Entity> = HashSet::new();

        for (female_e, fpos, _fa, _fcd, _fen, _fhp, _fwf, fcfg) in females.into_iter() {
            let radius2 = (fcfg.mating_search_radius * fcfg.mating_search_radius) as i32;
            let mut best: Option<(usize, i32)> = None;

            for (idx, (male_e, mpos, .., _mcfg)) in males.iter().enumerate() {
                if used_males.contains(male_e) {
                    continue;
                }
                let d = fpos - *mpos;
                let d2 = d.x * d.x + d.y * d.y;
                if d2 <= radius2 && best.map(|(_, bd2)| d2 < bd2).unwrap_or(true) {
                    best = Some((idx, d2));
                }
            }

            let Some((mi, _)) = best else {
                continue;
            };

            let (male_e, _, .., mcfg) = males[mi];
            used_males.insert(male_e);

            let meet = fpos;
            commands.entity(female_e).insert(MatingIntent {
                partner: male_e,
                meeting_tile: meet,
                duration_ticks: fcfg.mating_duration_ticks,
            });
            commands.entity(male_e).insert(MatingIntent {
                partner: female_e,
                meeting_tile: meet,
                duration_ticks: mcfg.mating_duration_ticks,
            });
            info!(
                "{}💞 Pair formed: female {:?} with male {:?} -> rendezvous at {:?}",
                species_emoji, female_e, male_e, meet
            );
        }
    }

    pub fn rabbit_mate_matching_system(
        mut commands: Commands,
        rabbits: Query<
            (
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
                &ReproductionConfig,
            ),
            With<Rabbit>,
        >,
        tick: Res<crate::simulation::SimulationTick>,
    ) {
        mate_matching_system_inner(&mut commands, &rabbits, tick.0, "🐇");
    }

    // Shared birth helper
    fn birth_common<E: Component>(
        commands: &mut Commands,
        mothers: &mut Query<(Entity, &TilePosition, &mut Pregnancy, &ReproductionConfig), With<E>>,
        mut spawn_fn: impl FnMut(&mut Commands, String, IVec2) -> Entity,
        species_emoji: &str,
        baby_prefix: &str,
    ) {
        let mut to_clear: Vec<Entity> = Vec::new();
        for (mother, pos, mut preg, cfg) in mothers.iter_mut() {
            if preg.remaining_ticks == 0 {
                let litter = preg.litter_size as usize;
                for i in 0..litter {
                    let name = format!("{}_{}", baby_prefix, i);
                    let baby = spawn_fn(commands, name, pos.tile);
                    let mut rng = rand::thread_rng();
                    let sex = if rng.gen_bool(0.5) {
                        Sex::Male
                    } else {
                        Sex::Female
                    };
                    commands
                        .entity(baby)
                        .insert(sex)
                        .insert(Age {
                            ticks_alive: 0,
                            mature_at_ticks: cfg.maturity_ticks,
                        })
                        .insert(WellFedStreak::default())
                        .insert(ReproductionCooldown::default())
                        .insert(Mother(mother));
                }
                info!(
                    "{} Birth: mother {:?} gave birth to {} offspring at {:?}",
                    species_emoji, mother, preg.litter_size, pos.tile
                );
                to_clear.push(mother);
            }
        }
        for e in to_clear {
            commands.entity(e).remove::<Pregnancy>();
        }
    }

    pub fn rabbit_birth_system(
        mut commands: Commands,
        mut mothers: Query<
            (Entity, &TilePosition, &mut Pregnancy, &ReproductionConfig),
            With<Rabbit>,
        >,
    ) {
        birth_common::<Rabbit>(
            &mut commands,
            &mut mothers,
            |cmds, name, pos| crate::entities::entity_types::spawn_rabbit(cmds, name, pos),
            "🐇🍼",
            "Kit",
        );
    }

    pub fn deer_mate_matching_system(
        mut commands: Commands,
        deer: Query<
            (
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
                &ReproductionConfig,
            ),
            With<crate::entities::Deer>,
        >,
        tick: Res<crate::simulation::SimulationTick>,
    ) {
        mate_matching_system_inner(&mut commands, &deer, tick.0, "🦌");
    }

    pub fn deer_birth_system(
        mut commands: Commands,
        mut mothers: Query<
            (Entity, &TilePosition, &mut Pregnancy, &ReproductionConfig),
            With<crate::entities::Deer>,
        >,
    ) {
        birth_common::<crate::entities::Deer>(
            &mut commands,
            &mut mothers,
            |cmds, name, pos| crate::entities::entity_types::spawn_deer(cmds, name, pos),
            "🦌🍼",
            "Fawn",
        );
    }

    pub fn raccoon_mate_matching_system(
        mut commands: Commands,
        raccoons: Query<
            (
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
                &ReproductionConfig,
            ),
            With<Raccoon>,
        >,
        tick: Res<crate::simulation::SimulationTick>,
    ) {
        mate_matching_system_inner(&mut commands, &raccoons, tick.0, "🦝");
    }

    pub fn raccoon_birth_system(
        mut commands: Commands,
        mut mothers: Query<
            (Entity, &TilePosition, &mut Pregnancy, &ReproductionConfig),
            With<Raccoon>,
        >,
    ) {
        birth_common::<Raccoon>(
            &mut commands,
            &mut mothers,
            |cmds, name, pos| crate::entities::entity_types::spawn_raccoon(cmds, name, pos),
            "🦝🍼",
            "Kit",
        );
    }
}

// -----------------------------
// Re-exports
// -----------------------------
pub use components::{
    Age, MatingIntent, Mother, Pregnancy, ReproductionCooldown, Sex, WellFedStreak,
};
pub use config::ReproductionConfig;
pub use systems::{
    deer_birth_system, deer_mate_matching_system, rabbit_birth_system, rabbit_mate_matching_system,
    raccoon_birth_system, raccoon_mate_matching_system, tick_reproduction_timers_system,
    update_age_and_wellfed_system,
};
