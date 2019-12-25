extern crate specs;
use specs::prelude::*;

use super::{Map, Position, BlocksTile};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, BlocksTile>,
                        Entities<'a> );

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, position, blockers, entities) = data;

        map.populate_blocked();
        map.clear_contents();
        for (entity, position) in (&entities, &position).join() {
            let idx = Map::xy_idx(position.x, position.y);

            // if they block, update the blocking list
            let _p : Option<&BlocksTile> = blockers.get(entity);
            if let Some(_p) = _p {
                map.blocked[idx] = true;
            }

            // push the entity to the appropriate index slot. 
            // it's a copy type, so we don't need to clone it
            map.contents[idx].push(entity);
        }
    }
}
