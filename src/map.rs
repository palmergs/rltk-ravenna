extern crate rltk;
use rltk::{ RGB, Rltk, Console, BaseMap, Algorithm2D, Point };

use super::{Rect};

use std::collections::HashSet;

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
    Floor,
    DownStairs,
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
    pub depth: i32,
    pub bloodstains : HashSet<usize>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub contents : Vec<Vec<Entity>>,
}

impl Map {
    /// Generates an empty map consisting entirely of solid walls
    pub fn new(depth: i32) -> Map {
        Map {
            tiles: vec![TileType::Wall; MAPCOUNT],
            rooms: Vec::new(),
            width: MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed: vec![false; MAPCOUNT],
            visible: vec![false; MAPCOUNT],
            blocked: vec![false; MAPCOUNT],
            depth,
            bloodstains: HashSet::new(),
            contents: vec![Vec::new(); MAPCOUNT],
        }
    }

    /// Converts coordinates into a linear index
    pub fn xy_idx(x: i32, y: i32) -> usize {
        (y as usize * MAPWIDTH) + x as usize
    }

    /// Makes a map with solid walls and 400 randomly placed
    /// squares.
    pub fn new_map_random(depth: i32) -> Map {
        let mut map = Map {
            tiles : vec![TileType::Wall; MAPCOUNT],
            revealed : vec![false; MAPCOUNT],
            visible : vec![false; MAPCOUNT],
            blocked: vec![false; MAPCOUNT],
            rooms : Vec::new(),
            width : MAPWIDTH as i32,
            height : MAPHEIGHT as i32,
            depth,
            bloodstains: HashSet::new(),
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
            let mut bg = RGB::from_f32(0.0, 0.0, 0.0);
            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('.');
                    fg = RGB::from_f32(0.0, 0.5, 0.5);
                },
                TileType::Wall => {
                    glyph = wall_glyph(&*map, x, y);
                    fg = RGB::from_f32(0.6, 0.6, 0.0);
                },
                TileType::DownStairs => {
                    glyph = rltk::to_cp437('>');
                    fg = RGB::from_f32(0.0, 1.0, 1.0);
                }
            }
            if map.bloodstains.contains(&idx) {
                bg = RGB::from_f32(0.75, 0.0, 0.0); 
            }

            if !map.visible[idx] {
                fg = fg.to_greyscale();
                bg = RGB::from_f32(0.0, 0.0, 0.0);
            }

            ctx.set(x, y, fg, bg, glyph);
        }

        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}

fn wall_glyph(map: &Map, x: i32, y: i32) -> u8 {
    let mut mask: u8 = 0;

    if is_revealed_and_wall(map, x, y-1) { mask += 1; }
    if is_revealed_and_wall(map, x, y+1) { mask += 2; }
    if is_revealed_and_wall(map, x-1, y) { mask += 4; }
    if is_revealed_and_wall(map, x+1, y) { mask += 8; }
    mask_to_glyph(mask)
}

fn mask_to_glyph(mask: u8) -> u8 {
    match mask {
        0 => 9,
        1 => 186,
        2 => 186,
        3 => 186,
        4 => 205,
        5 => 188,
        6 => 187,
        7 => 185,
        8 => 205,
        9 => 200,
        10 => 201,
        11 => 204,
        12 => 205,
        13 => 202,
        14 => 203,
        _ => 35
    }
}

fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
    let idx = Map::xy_idx(x, y);
    map.tiles[idx] == TileType::Wall && map.revealed[idx]
}
