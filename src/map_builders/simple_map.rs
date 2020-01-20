use rltk::{ RandomNumberGenerator };

use super::{
    Map,
    MapBuilder,
    TileType,
    Rect,
    Position,
    spawner,
    apply_room_to_map,
    apply_horizontal_tunnel, 
    apply_vertical_tunnel };

use specs::prelude::*;

pub struct SimpleMapBuilder {
    map: Map,
    starting_pos: Position,
    depth: i32
}

impl MapBuilder for SimpleMapBuilder {
    fn build_map(&mut self) {
        self.rooms_and_corridors();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        for room in self.map.rooms.iter().skip(1) {
            spawner::spawn_room(ecs, room, self.depth);
        }
    }

    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_pos(&self) -> Position {
        self.starting_pos.clone()
    }
}

impl SimpleMapBuilder {
    pub fn new(depth: i32) -> SimpleMapBuilder {
        SimpleMapBuilder {
            map: Map::new(depth),
            starting_pos: Position{ x: 0, y: 0 },
            depth: depth
        }
    }

    fn rooms_and_corridors(&mut self) {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.range(1, self.map.width - w);
            let y = rng.range(1, self.map.height - h);
            let room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other in self.map.rooms.iter() { 
                if room.intersect(other) { ok = false; }
            }

            if ok {
                apply_room_to_map(&mut self.map, &room);
                if !self.map.rooms.is_empty() {
                    let (nx, ny) = room.center();
                    let (px, py) = self.map.rooms[self.map.rooms.len()-1].center();
                    if rng.range(0, 2) == 1 {
                        apply_horizontal_tunnel(&mut self.map, px, nx, ny);
                        apply_vertical_tunnel(&mut self.map, nx, py, ny);
                    } else {
                        apply_vertical_tunnel(&mut self.map, px, py, ny);
                        apply_horizontal_tunnel(&mut self.map, px, nx, ny);
                    }
                }

                self.map.rooms.push(room);
            }
        }

        let stairs = self.map.rooms[self.map.rooms.len()-1].center();
        let stairs_idx = Map::xy_idx(stairs.0, stairs.1);
        self.map.tiles[stairs_idx] = TileType::DownStairs;

        let start_pos = self.map.rooms[0].center();
        self.starting_pos = Position { x: start_pos.0, y: start_pos.1 }
    }
}
