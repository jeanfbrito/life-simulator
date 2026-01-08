/// Reproduction components and systems (modular, species-agnostic with rabbit wrappers)
use bevy::prelude::*;
use rand::Rng;

use crate::entities::stats::{Energy, Health, Hunger, Thirst};
use crate::entities::TilePosition;

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
    
}

// -----------------------------
// Components
// -----------------------------
mod components {
    use bevy::prelude::*;

    /// Phase 4: Required Components
    /// Sex automatically requires Creature - compile-time guarantee.
    #[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
    #[require(crate::entities::Creature)]
    pub enum Sex {
        #[default]
        Male,
        Female,
    }

    /// Phase 4: Required Components
    /// Age automatically requires Creature - compile-time guarantee.
    #[derive(Component, Debug, Clone, Copy, Default)]
    #[require(crate::entities::Creature)]
    pub struct Age {
        pub ticks_alive: u64,
        pub mature_at_ticks: u32,
    }
    impl Age {
        pub fn is_adult(&self) -> bool {
            self.ticks_alive >= self.mature_at_ticks as u64
        }
    }

    /// Phase 4: Required Components
    /// ReproductionCooldown automatically requires Creature - compile-time guarantee.
    #[derive(Component, Debug, Clone, Copy)]
    #[require(crate::entities::Creature)]
    pub struct ReproductionCooldown {
        pub remaining_ticks: u32,
    }
    impl Default for ReproductionCooldown {
        fn default() -> Self {
            Self { remaining_ticks: 0 }
        }
    }

    /// Phase 4: Required Components
    /// Pregnancy automatically requires Age and Sex - compile-time guarantee
    /// that pregnant entities have age and sex attributes.
    #[derive(Component, Debug, Clone, Copy)]
    #[require(Age, Sex)]
    pub struct Pregnancy {
        pub remaining_ticks: u32,
        pub litter_size: u8,
        pub father: Option<Entity>,
    }

    #[derive(Component, Debug, Clone, Copy)]
    pub struct Mother(pub Entity);

    /// Phase 4: Required Components
    /// WellFedStreak automatically requires Creature - compile-time guarantee.
    #[derive(Component, Debug, Clone, Copy)]
    #[require(crate::entities::Creature)]
    pub struct WellFedStreak {
        pub ticks: u32,
    }
    impl Default for WellFedStreak {
        fn default() -> Self {
            Self { ticks: 0 }
        }
    }

    /// DEPRECATED: Use ActiveMate/MatingTarget relationship components instead.
    /// This component is kept for backward compatibility during migration.
    ///
    /// The new relationship system uses:
    /// - `ActiveMate` component on the pursuing entity (typically male)
    /// - `MatingTarget` component on the pursued entity (typically female)
    ///
    /// This provides type-safe bidirectional relationships with automatic cleanup.
    #[deprecated(
        since = "0.1.0",
        note = "Use ActiveMate/MatingTarget relationship components from mating_relationships module instead"
    )]
    #[derive(Component, Debug, Clone, Copy)]
    pub struct MatingIntent {
        pub partner: Entity,
        pub meeting_tile: IVec2,
        pub duration_ticks: u32,
    }

    
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
                    // Decay by 10% per tick (retain 90%) instead of losing 1 tick
                    wellfed.ticks = (wellfed.ticks as f32 * 0.90) as u32;
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

    /// DEPRECATED: Use mate_matching_system_with_relationships instead.
    /// This function uses the old MatingIntent component which is being phased out
    /// in favor of the ActiveMate/MatingTarget relationship system.
    #[deprecated(
        since = "0.1.0",
        note = "Use mate_matching_system_with_relationships for type-safe relationship tracking"
    )]
    #[allow(deprecated)]
    pub fn mate_matching_system<M: Component, const EMOJI: char>(
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
            (With<M>, Or<(Changed<TilePosition>, Changed<ReproductionCooldown>, Changed<Pregnancy>, Changed<WellFedStreak>)>),
        >,
        current_tick: u64,
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
                "{emoji}💞 Pair formed: female {:?} with male {:?} -> rendezvous at {:?}",
                female_e,
                male_e,
                meet,
                emoji = EMOJI,
            );
        }
    }

    /// DEPRECATED: Use mate_matching_system_with_relationships instead.
    /// This function uses the old MatingIntent component which is being phased out
    /// in favor of the ActiveMate/MatingTarget relationship system.
    ///
    /// Optimized mate matching system using Bevy Children component queries
    ///
    /// Phase 4.3: Replaces HashMap-based spatial queries with hierarchical Parent/Child queries.
    /// Maintains O(M*k) performance where k = entities in nearby chunks.
    ///
    /// Change Detection: Only processes entities that have moved (Changed<TilePosition>)
    /// or changed reproductive state (Changed<ReproductiveState> via Age, ReproductionCooldown, etc.)
    #[deprecated(
        since = "0.1.0",
        note = "Use mate_matching_system_with_relationships for type-safe relationship tracking"
    )]
    #[allow(deprecated)]
    pub fn mate_matching_system_with_children<M: Component, const EMOJI: char>(
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
            (With<M>, Or<(Changed<TilePosition>, Changed<ReproductionCooldown>, Changed<Pregnancy>, Changed<WellFedStreak>)>),
        >,
        grid: &crate::entities::SpatialCellGrid,
        cells: &Query<&bevy::prelude::Children, With<crate::entities::SpatialCell>>,
        current_tick: u64,
    ) {
        use std::collections::HashSet;

        let mut sampled_interval: Option<u64> = None;
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

        // First pass: collect eligible females only
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

            // Only process females in this optimized version
            if sex != Sex::Female {
                continue;
            }

            if preg_opt.is_some() || intent_opt.is_some() {
                continue;
            }
            if !is_eligible(age, cd, en, hp, wf, cfg) {
                continue;
            }

            females.push((entity, pos.tile, age, cd, en, hp, wf, cfg));
        }

        if females.is_empty() {
            return;
        }

        let mut used_males: HashSet<Entity> = HashSet::new();

        // For each female, use Children-based spatial query to find nearby males
        for (female_e, fpos, _fa, _fcd, _fen, _fhp, _fwf, fcfg) in females.into_iter() {
            // O(k) spatial query using Children component
            let nearby_entities = crate::entities::entities_in_radius_via_children(
                grid,
                cells,
                fpos,
                fcfg.mating_search_radius as f32,
            );

            let mut best: Option<(Entity, i32)> = None;
            let radius2 = (fcfg.mating_search_radius * fcfg.mating_search_radius) as i32;

            // Check nearby entities for compatible males
            for nearby_entity in nearby_entities {
                if used_males.contains(&nearby_entity) {
                    continue;
                }

                // Verify the entity is in our query and meets criteria
                if let Ok((
                    male_e,
                    male_pos,
                    male_age,
                    male_cd,
                    male_en,
                    male_hp,
                    male_wf,
                    male_preg_opt,
                    male_sex_opt,
                    male_intent_opt,
                    mcfg,
                )) = animals.get(nearby_entity)
                {
                    let Some(male_sex) = male_sex_opt.copied() else {
                        continue;
                    };

                    // Must be male and not already mating
                    if male_sex != Sex::Male {
                        continue;
                    }

                    if male_preg_opt.is_some() || male_intent_opt.is_some() {
                        continue;
                    }

                    if !is_eligible(male_age, male_cd, male_en, male_hp, male_wf, mcfg) {
                        continue;
                    }

                    // Check distance is within radius
                    let d = fpos - male_pos.tile;
                    let d2 = d.x * d.x + d.y * d.y;

                    if d2 <= radius2 && best.map(|(_, bd2)| d2 < bd2).unwrap_or(true) {
                        best = Some((male_e, d2));
                    }
                }
            }

            let Some((male_e, _)) = best else {
                continue;
            };

            used_males.insert(male_e);

            let meet = fpos;
            commands.entity(female_e).insert(MatingIntent {
                partner: male_e,
                meeting_tile: meet,
                duration_ticks: fcfg.mating_duration_ticks,
            });

            // Get male's config for mating duration
            if let Ok((_, _, _, _, _, _, _, _, _, _, mcfg)) = animals.get(male_e) {
                commands.entity(male_e).insert(MatingIntent {
                    partner: female_e,
                    meeting_tile: meet,
                    duration_ticks: mcfg.mating_duration_ticks,
                });
            }

            info!(
                "{emoji}💞 Pair formed: female {:?} with male {:?} -> rendezvous at {:?}",
                female_e,
                male_e,
                meet,
                emoji = EMOJI,
            );
        }
    }

    /// Mate matching system using type-safe relationship components
    ///
    /// Phase 11: Replaces MatingIntent with MatingTarget/ActiveMate relationship components
    /// for type-safe, bidirectional mating pair tracking with automatic cleanup.
    ///
    /// Establishes relationships via the relationship system functions rather than
    /// directly inserting MatingIntent components.
    pub fn mate_matching_system_with_relationships<M: Component, const EMOJI: char>(
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
                Option<&crate::entities::ActiveMate>,
                &ReproductionConfig,
            ),
            (With<M>, Or<(Changed<TilePosition>, Changed<ReproductionCooldown>, Changed<Pregnancy>, Changed<WellFedStreak>)>),
        >,
        current_tick: u64,
    ) {
        use std::collections::HashSet;
        use crate::ai::establish_mating_relationship;

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

        for (entity, pos, age, cd, en, hp, wf, preg_opt, sex_opt, mate_opt, cfg) in animals.iter()
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
            // Skip if pregnant or already has an active mating relationship
            if preg_opt.is_some() || mate_opt.is_some() {
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

            let (male_e, _, .., _mcfg) = males[mi];
            used_males.insert(male_e);

            let meet = fpos;
            // Use relationship system to establish bidirectional mating pair
            establish_mating_relationship(male_e, female_e, meet, current_tick, commands);

            info!(
                "{emoji}💞 Pair formed: female {:?} with male {:?} -> rendezvous at {:?}",
                female_e,
                male_e,
                meet,
                emoji = EMOJI,
            );
        }
    }

    // Shared birth helper used by species modules
    pub fn birth_common<E: Component>(
        commands: &mut Commands,
        mothers: &mut Query<(Entity, &TilePosition, &mut Pregnancy, &ReproductionConfig), With<E>>,
        mut spawn_fn: impl FnMut(&mut Commands, String, IVec2) -> Entity,
        species_emoji: &str,
        baby_prefix: &str,
    ) {
        let mut to_clear: Vec<Entity> = Vec::new();
        for (mother, pos, preg, cfg) in mothers.iter_mut() {
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
                    // Note: Parent-child relationship is established automatically by
                    // birth_relationships::establish_birth_relationships system which runs
                    // after birth systems and creates Bevy Parent/Children + BirthInfo components.
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

    // Species-specific systems live in species modules; only the helper stays shared here.
}

// -----------------------------
// Re-exports
// -----------------------------
#[allow(deprecated)]
pub use components::{
    Age, MatingIntent, Mother, Pregnancy, ReproductionCooldown, Sex, WellFedStreak,
};
pub use config::ReproductionConfig;
#[allow(deprecated)]
pub use systems::{
    birth_common, mate_matching_system, mate_matching_system_with_children,
    mate_matching_system_with_relationships,
    tick_reproduction_timers_system, update_age_and_wellfed_system,
};

// -----------------------------
// Tests
// -----------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_well_fed_streak_percentage_decay() {
        let mut streak = WellFedStreak { ticks: 200 };

        // Simulate 10 ticks not fed with 90% retention per tick
        for _ in 0..10 {
            streak.ticks = (streak.ticks as f32 * 0.90) as u32;
        }

        // Should retain ~35% of original (0.9^10 ≈ 0.35)
        // 200 * 0.35 ≈ 70 ticks
        assert!(streak.ticks >= 65 && streak.ticks <= 75,
                "Expected ~70 ticks, got {}", streak.ticks);
    }

    #[test]
    fn test_well_fed_streak_brief_interruption() {
        let mut streak = WellFedStreak { ticks: 300 };

        // Simulate 5 ticks not fed (brief drink)
        for _ in 0..5 {
            streak.ticks = (streak.ticks as f32 * 0.90) as u32;
        }

        // Should retain ~60% (0.9^5 ≈ 0.59)
        // 300 * 0.59 ≈ 177 ticks
        assert!(streak.ticks >= 170,
                "Expected >=170 ticks after brief interruption, got {}", streak.ticks);
    }

    #[test]
    fn test_well_fed_streak_complete_decay() {
        let mut streak = WellFedStreak { ticks: 100 };

        // Simulate 50 ticks not fed
        for _ in 0..50 {
            streak.ticks = (streak.ticks as f32 * 0.90) as u32;
        }

        // Should decay significantly but not to zero (0.9^50 ≈ 0.005)
        // 100 * 0.005 ≈ 0-1 ticks
        assert!(streak.ticks <= 5,
                "Expected near-zero ticks after long decay, got {}", streak.ticks);
    }

    #[test]
    fn test_well_fed_streak_growth_still_works() {
        let mut streak = WellFedStreak { ticks: 100 };

        // Simulate growth (this tests that our change doesn't break increment logic)
        streak.ticks = streak.ticks.saturating_add(1);
        assert_eq!(streak.ticks, 101, "Growth should still work");

        streak.ticks = streak.ticks.saturating_add(10);
        assert_eq!(streak.ticks, 111, "Multi-tick growth should work");
    }
}
