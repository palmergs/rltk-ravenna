use rltk::{ RGB, Rltk, Console, RandomNumberGenerator };
use super::{Rect};
use std::cmp::{max, min};

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor
}

/// Converts coordinates into a linear index
pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}

/// Makes a map with solid walls and 400 randomly placed
/// squares.
pub fn new_map_random() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; 80*50];
    for x in 0..80 {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, 49)] = TileType::Wall;
    }

    for y in 0..50 {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(79, y)] = TileType::Wall;
    }

    let mut rng = rltk::RandomNumberGenerator::new();
    for _i in 0..400 {
        let x = rng.roll_dice(1, 79);
        let y = rng.roll_dice(1, 49);
        let idx = xy_idx(x, y);
        if idx != xy_idx(40, 25) {
            map[idx] = TileType::Wall;
        }
    }

    map
}

pub fn new_map_rooms_and_corridors() -> (Vec<Rect>, Vec<TileType>) {
    let mut map = vec![TileType::Wall; 80*50];
    let mut rooms : Vec<Rect> = Vec::new();
    const MAX_ROOMS : i32 = 30;
    const MIN_SIZE : i32 = 4;
    const MAX_SIZE : i32 = 10;

    let mut rng = rltk::RandomNumberGenerator::new();
    for i in 0..MAX_ROOMS {
        let w = rng.range(MIN_SIZE, MAX_SIZE);
        let h = rng.range(MIN_SIZE, MAX_SIZE);
        let x = rng.range(1, 79 - w);
        let y = rng.range(1, 49 - h);


        let room = Rect::new(x, y, w, h);
        let mut ok = true;
        for other in rooms.iter() {
            if room.intersect(other) {
                ok = false;
            }
        }

        if ok {
            sub_room_from_map(&room, &mut map);
            if !rooms.is_empty() {
                let (new_x, new_y) = room.center();
                let (pre_x, pre_y) = rooms[rooms.len() - 1].center();
                if rng.range(0, 2) == 1 {
                    sub_horizontal_tunnel(pre_x, new_x, pre_y, &mut map);
                    sub_vertical_tunnel(new_x, pre_y, new_y, &mut map);
                } else {
                    sub_vertical_tunnel(pre_x, pre_y, new_y, &mut map);
                    sub_horizontal_tunnel(pre_x, new_x, new_y, &mut map);
                }
            }
            rooms.push(room);
        }
    }

    (rooms, map)
}

fn sub_room_from_map(room: &Rect, map: &mut [TileType]) {
    for x in room.x1 ..= room.x2 {
        for y in room.y1 ..= room.y2 {
            map[xy_idx(x, y)] = TileType::Floor;
        }
    }
}

fn sub_horizontal_tunnel(x1: i32, x2: i32, y: i32, map: &mut [TileType]) {
    for x in min(x1, x2)..max(x1, x2)+1 {
        let idx = xy_idx(x, y);
        map[idx] = TileType::Floor;
    }
}

fn sub_vertical_tunnel(x: i32, y1: i32, y2: i32, map: &mut [TileType]) {
    for y in min(y1, y2)..max(y1, y2)+1 {
        let idx = xy_idx(x, y);
        map[idx] = TileType::Floor;
    }
}
