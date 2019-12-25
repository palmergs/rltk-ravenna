extern crate rltk;
use rltk::{ RGB, RandomNumberGenerator };

extern crate specs;
use specs::prelude::*;

use super::{ 
    CombatStats, 
    Player, 
    Renderable, 
    Name, 
    Position, 
    Viewshed, 
    Monster, 
    BlocksTile };

/// Spawns the player and returns their entity object
pub fn player(ecs: &mut World, x: i32, y: i32) -> Entity {
    ecs.create_entity().
        with(Position { x, y }).
        with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        }).
        with(Player {}).
        with(Viewshed { tiles: Vec::new(), range: 8, dirty: true }).
        with(Name { name: "Player".to_string() }).
        with(CombatStats { max_hp: 30, hp: 30, defense: 2, power: 5 }).
        build()
}

/// Spawns a random monster at a given location
pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll : i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => { orc(ecs, x, y) }
        _ => { goblin(ecs, x, y) }
    }
}

fn orc(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('o'), "Orc")
}

fn goblin(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('g'), "Goblin")
}

fn monster<S : ToString>(ecs: &mut World, x: i32, y: i32, glyph: u8, name: S) {
    ecs.create_entity().
        with(Position { x, y }).
        with(Renderable {
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        }).
        with(Viewshed { tiles: Vec::new(), range: 8, dirty: true }).
        with(Monster {}).
        with(Name { name: name.to_string() }).
        with(BlocksTile {}).
        with(CombatStats { max_hp: 16, hp: 16, defense: 1, power: 4 }).
        build();
}

