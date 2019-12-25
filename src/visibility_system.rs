extern crate specs;
use specs::prelude::*;
use super::{Viewshed, Position, Map, Player};
use rltk::{field_of_view, Point};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = ( WriteExpect<'a, Map>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Player> );

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player) = data;
        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.tiles.clear();
                viewshed.tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
                viewshed.tiles.retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);
               
                let _p : Option<&Player> = player.get(ent);
                if let Some(_p) = _p {
                    for t in map.visible.iter_mut() { *t = false; }
                    for vis in viewshed.tiles.iter() {
                        let idx = Map::xy_idx(vis.x, vis.y);
                        map.revealed[idx] = true;
                        map.visible[idx] = true;
                    }
                }
            }
        }
    }
}
