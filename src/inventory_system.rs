extern crate specs;
use specs::prelude::*;

use super::{
    Map,
    WantsToPickupItem,
    WantsToUseItem,
    WantsToDropItem,
    ProvidesHealing,
    InflictsDamage,
    Consumable,
    Name,
    InBackpack,
    Position,
    CombatStats,
    SufferDamage,
    gamelog::GameLog };

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToPickupItem>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>, );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack.
                insert(pickup.item, InBackpack { owner: pickup.collected_by }).
                expect("Unable to insert item into backpack");

            if pickup.collected_by == *player_entity {
                gamelog.entries.insert(0, format!(
                    "You pick up the {}", 
                    names.get(pickup.item).unwrap().name));
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToUseItem>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, Consumable>,
                        ReadStorage<'a, ProvidesHealing>,
                        ReadStorage<'a, InflictsDamage>,
                        WriteStorage<'a, CombatStats>,
                        WriteStorage<'a, SufferDamage>,
                        ReadExpect<'a, Map> );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, 
             mut gamelog, 
             entities, 
             mut wants_use, 
             names, 
             consumables, 
             healers,
             damagers,
             mut combat_stats,
             mut suffer_damage,
             map) = data;

        for(entity, item, stats) in (&entities, &wants_use, &mut combat_stats).join() {
            let item_heals = healers.get(item.item);
            match item_heals {
                None => {},
                Some(healer) => {
                    stats.hp = i32::max(stats.max_hp, stats.hp + healer.heal_amount);
                    if entity == *player_entity {
                        gamelog.entries.insert(0, format!("The {} heals you by {} points", names.get(item.item).unwrap().name, healer.heal_amount));
                    }
                }
            }

            let item_hurts = damagers.get(item.item);
            match item_hurts {
                None => {},
                Some(damage) => {
                    let target_point = item.target.unwrap();
                    let idx = Map::xy_idx(target_point.x, target_point.y);
                    for mob in map.contents[idx].iter() {
                        suffer_damage.insert(*mob, SufferDamage { amount: damage.damage }).expect("Unable to insert");
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(item.item).unwrap();
                            gamelog.entries.insert(0, format!("You use {} on {} and inflict {} damage", item_name.name, mob_name.name, damage.damage));
                        }
                    }
                }
            }

            let consumable = consumables.get(item.item);
            match consumable {
                None => {},
                Some(_) => {
                    entities.delete(item.item).expect("Delete failed");
                }
            }
        }

        wants_use.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToDropItem>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, InBackpack> );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity,
             mut gamelog,
             entities,
             mut wants_drop,
             names,
             mut positions,
             mut backpack) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos : Position = Position{ x: 0, y: 0 };
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions.insert(to_drop.item, Position{ x: dropper_pos.x, y: dropper_pos.y }).expect("Unable to insert position");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.insert(0, format!("You drop the {}", names.get(to_drop.item).unwrap().name));
            }
        }
        wants_drop.clear();
    }
}
