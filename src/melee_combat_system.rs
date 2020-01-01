extern crate rltk;
use rltk::{ RGB, Rltk };

extern crate specs;
use specs::prelude::*;
use super::{ 
    CombatStats,
    WantsToMelee,
    Name,
    SufferDamage,
    GameLog,
    MeleePowerBonus,
    DefenseBonus,
    Equipped,
    particle_system::ParticleBuilder,
    Position, };

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = ( Entities<'a>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToMelee>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, CombatStats>,
                        WriteStorage<'a, SufferDamage>,
                        ReadStorage<'a, MeleePowerBonus>,
                        ReadStorage<'a, DefenseBonus>,
                        ReadStorage<'a, Equipped>,
                        WriteExpect<'a, ParticleBuilder>,
                        ReadStorage<'a, Position> );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, 
            mut log, 
            mut wants_melee, 
            names, 
            combat_stats, 
            mut inflict_damage,
            melee_bonuses,
            defense_bonuses,
            equipment,
            mut particle_builder, 
            positions ) = data;

        for (entity, wants_melee, name, stats) in (&entities, &wants_melee, &names, &combat_stats).join() {
            if stats.hp > 0 {
                let mut offensive_bonus = 0;
                for (_ie, power_bonus, equipped_by) in (&entities, &melee_bonuses, &equipment).join() {
                    if equipped_by.owner == entity {
                        offensive_bonus += power_bonus.power
                    }
                }

                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_melee.target).unwrap();

                    let mut defensive_bonus = 0;
                    for (_ie, defense_bonus, equipped_by) in (&entities, &defense_bonuses, &equipment).join() {
                        if equipped_by.owner == wants_melee.target {
                            defensive_bonus += defense_bonus.defense;
                        }
                    }


                    let damage = i32::max(0, stats.power + offensive_bonus - target_stats.defense - defensive_bonus);
                    if damage == 0 {
                        log.entries.insert(0, format!("{} is unable to hurt {}", &name.name, &target_name.name));
                    } else {
                        let pos = positions.get(wants_melee.target);
                        if let Some(pos) = pos {
                            particle_builder.requests(
                                pos.x, 
                                pos.y, 
                                RGB::named(rltk::ORANGE),
                                RGB::named(rltk::BLACK),
                                rltk::to_cp437('‼'),
                                200.0);
                        }
                        log.entries.insert(0, format!("{} hits {} for {} damage", &name.name, &target_name.name, damage));
                        inflict_damage.insert(wants_melee.target, SufferDamage { amount: damage }).expect("Unable to do damage");
                    }
                }
            }
        }
        wants_melee.clear();
    }
}
