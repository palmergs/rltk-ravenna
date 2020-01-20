use super::{ Map, Rect, TileType };
use std::cmp::{ max, min };

pub fn apply_room_to_map(map: &mut Map, room: &Rect) {
    for x in room.x1 ..= room.x2 {
        for y in room.y1 ..= room.y2 {
            let idx = Map::xy_idx(x, y);
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn apply_horizontal_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2) ..= max(x1, x2) {
        let idx = Map::xy_idx(x, y);
        map.tiles[idx] = TileType::Floor;
    }
}

pub fn apply_vertical_tunnel(map: &mut Map, x: i32, y1: i32, y2: i32) {
    for y in min(y1, y2) ..= max(y1, y2) {
        let idx = Map::xy_idx(x, y);
        map.tiles[idx] = TileType::Floor;
    }
}

