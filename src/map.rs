extern crate rltk;
use rltk::{ RGB, Rltk, Console, BaseMap, Algorithm2D, Point };

use super::{Rect};

use std::cmp::{max, min};

extern crate specs;
use specs::prelude::*;

extern crate serde;
use serde::{ Serialize, Deserialize };

pub const MAPWIDTH : usize = 80;
pub const MAPHEIGHT : usize = 50;
pub const MAPCOUNT : usize = MAPHEIGHT * MAPWIDTH;

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles : Vec<TileType>,
    pub revealed : Vec<bool>,
    pub visible : Vec<bool>,
    pub blocked : Vec<bool>,
    pub rooms : Vec<Rect>,
    pub width : i32,
    pub height : i32,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub contents : Vec<Vec<Entity>>,
}

impl Map {
    /// Converts coordinates into a linear index
    pub fn xy_idx(x: i32, y: i32) -> usize {
        (y as usize * MAPWIDTH) + x as usize
    }

    /// Makes a map with solid walls and 400 randomly placed
    /// squares.
    pub fn new_map_random() -> Map {
        let mut map = Map {
            tiles : vec![TileType::Wall; MAPCOUNT],
            revealed : vec![false; MAPCOUNT],
            visible : vec![false; MAPCOUNT],
            blocked: vec![false; MAPCOUNT],
            rooms : Vec::new(),
            width : MAPWIDTH as i32,
            height : MAPHEIGHT as i32,
            contents : vec![Vec::new(); MAPCOUNT],
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
            tiles : vec![TileType::Wall; MAPCOUNT],
            revealed : vec![false; MAPCOUNT],
            visible : vec![false; MAPCOUNT],
            blocked : vec![false; MAPCOUNT],
            rooms : Vec::new(),
            width : MAPWIDTH as i32,
            height : MAPHEIGHT as i32,
            contents : vec![Vec::new(); MAPCOUNT],
        };

        const MAX_ROOMS : i32 = 30;
        const MIN_SIZE : i32 = 4;
        const MAX_SIZE : i32 = 10;

        let mut rng = rltk::RandomNumberGenerator::new();
        for _i in 0..MAX_ROOMS {
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

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 { 
            return false 
        }

        let idx = Map::xy_idx(x, y);
        !self.blocked[idx]
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    pub fn clear_contents(&mut self) {
        for content in self.contents.iter_mut() {
            content.clear();
        }
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: i32) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }

    fn get_available_exits(&self, idx: i32) -> Vec<(i32, f32)> {
        let mut exits : Vec<(i32, f32)> = Vec::new();
        let x = idx % self.width;
        let y = idx / self.width;

        if self.is_exit_valid(x-1, y) { exits.push((idx-1, 1.0)) };
        if self.is_exit_valid(x+1, y) { exits.push((idx+1, 1.0)) };
        if self.is_exit_valid(x, y-1) { exits.push((idx-self.width, 1.0)) };
        if self.is_exit_valid(x, y+1) { exits.push((idx+self.width, 1.0)) };

        if self.is_exit_valid(x-1, y-1) { exits.push(((idx-self.width)-1, 1.45)) };
        if self.is_exit_valid(x+1, y-1) { exits.push(((idx-self.width)+1, 1.45)) };
        if self.is_exit_valid(x-1, y+1) { exits.push(((idx+self.width)-1, 1.45)) };
        if self.is_exit_valid(x+1, y+1) { exits.push(((idx+self.width)+1, 1.45)) };

        exits
    }

    fn get_pathing_distance(&self, idx1: i32, idx2: i32) -> f32 {
        let p1 = Point::new(idx1 % self.width, idx1 / self.width);
        let p2 = Point::new(idx2 % self.width, idx2 / self.width);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
    fn in_bounds(&self, pt : Point) -> bool {
        pt.x > 0 && pt.x < self.width - 1 && pt.y > 0 && pt.y < self.height - 1
    }

    fn point2d_to_index(&self, pt : Point) -> i32 {
        (pt.y * self.width) + pt.x
    }

    fn index_to_point2d(&self, idx : i32) -> Point {
        Point { x: idx % self.width, y: idx / self.width }
    }
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();

    let mut x = 0;
    let mut y = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {
        if map.revealed[idx] {
            let glyph;
            let mut fg;
            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('.');
                    fg = RGB::from_f32(0.0, 0.4, 0.4);
                },
                TileType::Wall => {
                    glyph = rltk::to_cp437('#');
                    fg = RGB::from_f32(0.4, 0.4, 0.0);
                }
            }
            if !map.visible[idx] { fg = fg.to_greyscale(); }
            ctx.set(x, y, fg, RGB::from_f32(0.0, 0.0, 0.0), glyph);
        }

        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}
