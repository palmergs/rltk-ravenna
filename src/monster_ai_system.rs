extern crate specs;
use specs::prelude::*;

use super::{Position, Map, Viewshed, Monster, Name};

extern crate rltk;
use rltk::{Point, console};

pub struct MonsterAI {}

impl <'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, Point>,
                        ReadStorage<'a, Viewshed>,
                        ReadStorage<'a, Monster>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, Position> );

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, player_pos, mut viewshed, monster, name, mut position) = data;

        for (mut viewshed, _monster, name, mut pos) in (&mut viewshed, &monster, &name, &mut position).join() {
            if viewshed.tiles.contains(&*player_pos) {
                console::log(format!("{} shouts insults!", name.name));

                let path = rltk::a_star_search(
                    Map::xy_idx(pos.x, pos.y) as i32,
                    Map::xy_idx(player_pos.x, player_pos.y) as i32,
                    &mut *map);

                if path.success && path.steps.len() > 1 {
                    pos.x = path.steps[1] % map.width;
                    pos.y = path.steps[1] / map.width;
                    viewshed.dirty = true;
                }
            }
        }
    }
}
