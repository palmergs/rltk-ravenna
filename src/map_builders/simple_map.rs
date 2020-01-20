use rltk::{ RandomNumberGenerator };

use super::{
    Map,
    MapBuilder,
    TileType,
    Rect,
    Position,
    apply_room_to_map,
    apply_horizontal_tunnel, 
    apply_vertical_tunnel };

use specs::prelude::*;

pub struct SimpleMapBuilder {}

impl MapBuilder for SimpleMapBuilder {
    fn build(depth: i32) -> (Map, Position) {
        let mut map = Map::new(depth);
        let player_pos = SimpleMapBuilder::rooms_and_corridors(&mut map);
        (map, player_pos)
    }
}

impl SimpleMapBuilder {
    pub fn new(depth: i32) -> SimpleMapBuilder {
        SimpleMapBuilder {}
    }

    fn rooms_and_corridors(map: &mut Map) -> Position {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.range(1, map.width - w);
            let y = rng.range(1, map.height - h);
            let room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other in map.rooms.iter() { if room.intersect(other) { ok = false; } }

            if ok {
                apply_room_to_map(map, &room);
                if !map.rooms.is_empty() {
                    let (nx, ny) = room.center();
                    let (px, py) = map.rooms[map.rooms.len()-1].center();
                    if rng.range(0, 2) == 1 {
                        apply_horizontal_tunnel(map, px, nx, ny);
                        apply_vertical_tunnel(map, nx, py, ny);
                    } else {
                        apply_vertical_tunnel(map, px, py, ny);
                        apply_horizontal_tunnel(map, px, nx, ny);
                    }
                }

                map.rooms.push(room);
            }
        }

        let stairs = map.rooms[map.rooms.len()-1].center();
        let stairs_idx = Map::xy_idx(stairs.0, stairs.1);
        map.tiles[stairs_idx] = TileType::DownStairs;

        let start_pos = map.rooms[0].center();
        Position { x: start_pos.0, y: start_pos.1 }
    }
}
