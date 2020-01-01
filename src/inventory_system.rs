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
    Position,
    CombatStats,
    SufferDamage,
    AreaOfEffect,
    Confusion,
    InBackpack,
    Equippable,
    Equipped,
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
                        ReadStorage<'a, AreaOfEffect>,
                        WriteStorage<'a, Confusion>,
                        ReadStorage<'a, Equippable>,
                        WriteStorage<'a, Equipped>,
                        WriteStorage<'a, InBackpack>,
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
             aoe,
             mut confused,
             equippable,
             mut equipment,
             mut backpack,
             map) = data;

        for(entity, useitem) in (&entities, &wants_use).join() {
            let mut targets : Vec<Entity> = Vec::new();
            match useitem.target {
                None => { targets.push(*player_entity); }
                Some(target) => {
                    let area_effect = aoe.get(useitem.item);
                    match area_effect {
                        None => {
                            let idx = Map::xy_idx(target.x, target.y);
                            for mob in map.contents[idx].iter() { targets.push(*mob); }
                        }
                        Some(area_effect) => {
                            let mut blast_tiles = rltk::field_of_view(target, area_effect.radius, &*map);
                            blast_tiles.retain(|p| p.x > 0 && p.x < map.width-1 && p.y > 0 && p.y < map.height-1);
                            for tile_idx in blast_tiles.iter() {
                                let idx = Map::xy_idx(tile_idx.x, tile_idx.y);
                                for mob in map.contents[idx].iter() { targets.push(*mob); }
                            }
                        }
                    }
                }
            }

            let mut add_confusion = Vec::new();
            {
                let causes_confusion = confused.get(useitem.item);
                match causes_confusion {
                    None => {},
                    Some(confusion) => {
                        for mob in targets.iter() {
                            add_confusion.push((*mob, confusion.turns));
                            if entity == *player_entity {
                                let mob_name = names.get(*mob).unwrap();
                                let item_name = names.get(useitem.item).unwrap();
                                gamelog.entries.insert(0, format!("You use {} on {}, confusing them", item_name.name, mob_name.name));
                            }
                        }
                    }
                }
            }
            for mob in add_confusion.iter() {
                confused.insert(mob.0, Confusion { turns: mob.1 }).expect("Unable to insert confused");
            }

            let item_heals = healers.get(useitem.item);
            match item_heals {
                None => {},
                Some(healer) => {
                    for target in targets.iter() {
                        let stats = combat_stats.get_mut(*target);
                        if let Some(stats) = stats {
                            stats.hp = i32::max(stats.max_hp, stats.hp + healer.heal_amount);
                            if entity == *player_entity {
                                gamelog.entries.insert(0, format!("The {} heals you by {} points", names.get(useitem.item).unwrap().name, healer.heal_amount));
                            }
                        }
                    }
                }
            }

            let item_hurts = damagers.get(useitem.item);
            match item_hurts {
                None => {},
                Some(damage) => {
                    for mob in targets.iter() {
                        suffer_damage.insert(*mob, SufferDamage { amount: damage.damage }).expect("Unable to insert");
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(useitem.item).unwrap();
                            gamelog.entries.insert(0, format!("You use {} on {} and inflict {} damage", item_name.name, mob_name.name, damage.damage));
                        }
                    }
                }
            }

            let item_equippable = equippable.get(useitem.item);
            match item_equippable {
                None => {},
                Some(can_equip) => {
                    let target_slot = can_equip.slot;
                    let target = targets[0];

                    // remove any items the target has in the item's slot
                    let mut to_unequip : Vec<Entity> = Vec::new();
                    for (item_entity, already_equipped, name) in (&entities, &equipment, &names).join() {
                        if already_equipped.owner == target && already_equipped.slot == target_slot {
                            to_unequip.push(item_entity);
                            if target == *player_entity {
                                gamelog.entries.insert(0, format!("You unequip {}", name.name));
                            }
                        }
                    }

                    for item in to_unequip.iter() {
                        equipment.remove(*item);
                        backpack.insert(*item, InBackpack { owner: target }).expect("Unable to put in backpack");
                    }

                    // weild the item
                    equipment.insert(useitem.item, Equipped { owner: target, slot: target_slot }).expect("unable to wield equipment");
                    backpack.remove(useitem.item);
                    if target == *player_entity {
                        gamelog.entries.insert(0, format!("You equip {}", names.get(useitem.item).unwrap().name));
                    }
                }
            }

            let consumable = consumables.get(useitem.item);
            match consumable {
                None => {},
                Some(_) => {
                    entities.delete(useitem.item).expect("Delete failed");
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
