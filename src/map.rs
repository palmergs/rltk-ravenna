use rltk::{ RGB, Rltk, Console, RandomNumberGenerator };
use super::{Rect};
use std::cmp::{max, min};

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor
}

pub struct Map {
    pub tiles : Vec<TileType>,
    pub rooms : Vec<Rect>,
    pub width : i32,
    pub height : i32,
}

impl Map {
    /// Converts coordinates into a linear index
    pub fn xy_idx(x: i32, y: i32) -> usize {
        (y as usize * 80) + x as usize
    }

    /// Makes a map with solid walls and 400 randomly placed
    /// squares.
    pub fn new_map_random() -> Map {
        let mut map = Map {
            tiles : vec![TileType::Wall; 80*50],
            rooms : Vec::new(),
            width : 80,
            height : 50,
        };

        for x in 0..map.width {
            map.tiles[Map::xy_idx(x, 0)] = TileType::Wall;
            map.tiles[Map::xy_idx(x, map.height - 1)] = TileType::Wall;
        }

        for y in 0..map.height {
            map.tiles[Map::xy_idx(0, y)] = TileType::Wall;
            map.tiles[Map::xy_idx(map.width - 1, y)] = TileType::Wall;
        }

        let mut rng = rltk::RandomNumberGenerator::new();
        for _i in 0..400 {
            let x = rng.roll_dice(1, map.width - 1);
            let y = rng.roll_dice(1, map.height - 1);
            let idx = Map::xy_idx(x, y);
            if idx != Map::xy_idx(map.width / 2, map.height / 2) {
                map.tiles[idx] = TileType::Wall;
            }
        }

        map
    }

    pub fn new_map_rooms_and_corridors() -> Map {
        let mut map = Map {
            tiles : vec![TileType::Wall; 80*50],
            rooms : Vec::new(),
            width : 80,
            height : 50,
        };

        const MAX_ROOMS : i32 = 30;
        const MIN_SIZE : i32 = 4;
        const MAX_SIZE : i32 = 10;

        let mut rng = rltk::RandomNumberGenerator::new();
        for i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.range(1, map.width - 1 - w);
            let y = rng.range(1, map.height - 1 - h);


            let room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other in map.rooms.iter() {
                if room.intersect(other) {
                    ok = false;
                }
            }

            if ok {
                Map::sub_room_from_map(&room, &mut map);
                if !map.rooms.is_empty() {
                    let (new_x, new_y) = room.center();
                    let (pre_x, pre_y) = map.rooms[map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        Map::sub_horizontal_tunnel(pre_x, new_x, pre_y, &mut map);
                        Map::sub_vertical_tunnel(new_x, pre_y, new_y, &mut map);
                    } else {
                        Map::sub_vertical_tunnel(pre_x, pre_y, new_y, &mut map);
                        Map::sub_horizontal_tunnel(pre_x, new_x, new_y, &mut map);
                    }
                }
                map.rooms.push(room);
            }
        }

        map
    }

    fn sub_room_from_map(room: &Rect, map: &mut Map) {
        for x in room.x1 ..= room.x2 {
            for y in room.y1 ..= room.y2 {
                map.tiles[Map::xy_idx(x, y)] = TileType::Floor;
            }
        }
    }

    fn sub_horizontal_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
        for x in min(x1, x2)..max(x1, x2)+1 {
            let idx = Map::xy_idx(x, y);
            map.tiles[idx] = TileType::Floor;
        }
    }

    fn sub_vertical_tunnel(x: i32, y1: i32, y2: i32, map: &mut Map) {
        for y in min(y1, y2)..max(y1, y2)+1 {
            let idx = Map::xy_idx(x, y);
            map.tiles[idx] = TileType::Floor;
        }
    }
}
