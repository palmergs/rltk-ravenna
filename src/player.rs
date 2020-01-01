extern crate rltk;
use rltk::{VirtualKeyCode, Rltk, Point};

extern crate specs;
use specs::prelude::*;

use super::{
    RunState, 
    Position, 
    Player, 
    Map, 
    Monster,
    TileType,
    Item,
    State, 
    Viewshed, 
    CombatStats, 
    WantsToPickupItem,
    GameLog,
    WantsToMelee};
use std::cmp::{min, max};

/// Check the map to see if the player can move into the 
/// given location.
pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let entities = ecs.entities();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.fetch::<Map>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();

    for (entity, _play, pos, viewshed) in (&entities, &players, &mut positions, &mut viewsheds).join() {
        if pos.x + delta_x < 1 || 
            pos.x + delta_x > map.width - 1 || 
            pos.y + delta_y < 1 ||
            pos.y + delta_y > map.height -1 { return; }

        let dest_idx = Map::xy_idx(pos.x + delta_x, pos.y + delta_y);
        for potential_target in map.contents[dest_idx].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                wants_to_melee.
                    insert(entity, WantsToMelee { target: *potential_target }).
                    expect("Add target failed!");
                return;
            }
        }

        if !map.blocked[dest_idx] {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
            
            // update the resource for this user's position
            viewshed.dirty = true;
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => { return RunState::AwaitingInput },
        Some(key) => match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::Numpad4 |
            VirtualKeyCode::H => try_move_player(-1, 0, &mut gs.ecs),

            VirtualKeyCode::Right |
            VirtualKeyCode::Numpad6 |
            VirtualKeyCode::L => try_move_player(1, 0, &mut gs.ecs),

            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 |
            VirtualKeyCode::K => try_move_player(0, -1, &mut gs.ecs),

            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 |
            VirtualKeyCode::J => try_move_player(0, 1, &mut gs.ecs),

            VirtualKeyCode::Numpad9 |
            VirtualKeyCode::Y => try_move_player(1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad7 |
            VirtualKeyCode::U => try_move_player(-1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad3 |
            VirtualKeyCode::N => try_move_player(1, 1, &mut gs.ecs),

            VirtualKeyCode::Numpad1 |
            VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.ecs),

            VirtualKeyCode::G => get_item(&mut gs.ecs),
            VirtualKeyCode::D => return RunState::ShowDropItem,

            VirtualKeyCode::I => return RunState::ShowInventory,

            VirtualKeyCode::Escape => return RunState::SaveGame,

            VirtualKeyCode::Numpad5 => return skip_turn(&mut gs.ecs),
            VirtualKeyCode::Space => return skip_turn(&mut gs.ecs),

            VirtualKeyCode::Period => {
                if try_next_level(&mut gs.ecs) {
                    return RunState::NextLevel;
                }
            },

            _ => { return RunState::AwaitingInput }
        },
    }

    RunState::PlayerTurn
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player = ecs.fetch::<Entity>();
    let viewsheds = ecs.read_storage::<Viewshed>();
    let monsters = ecs.read_storage::<Monster>();
    let worldmap = ecs.fetch::<Map>();

    let mut can_heal = true;
    let viewshed = viewsheds.get(*player).unwrap();
    for tile in viewshed.tiles.iter() {
        let idx = Map::xy_idx(tile.x, tile.y);
        for entity_id in worldmap.contents[idx].iter() {
            let mob = monsters.get(*entity_id);
            match mob {
                None => {}
                Some(_) => { can_heal = false; }
            }
        }
    }

    if can_heal {
        let mut stats = ecs.write_storage::<CombatStats>();
        let player_hp = stats.get_mut(*player).unwrap();
        player_hp.hp = i32::min(player_hp.hp + 1, player_hp.max_hp);
    }

    RunState::PlayerTurn
}

fn try_next_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = Map::xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::DownStairs {
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog.entries.insert(0, "There is no way down from here".to_string());
        false
    }
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();
    let mut target_item : Option<Entity> = None;
    for(item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }
    
    match target_item {
        None => gamelog.entries.insert(0, "There is nothing here to pick up".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup.insert(*player_entity, 
                WantsToPickupItem { collected_by: *player_entity, item }
            ).expect("Unable to insert want to pickup");
        }
    }
}
