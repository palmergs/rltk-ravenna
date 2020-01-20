use super::{ 
    Map,
    Rect,
    TileType,
    Position,
    spawner };

mod simple_map;
use simple_map::SimpleMapBuilder;

mod common;
use common::*;

use specs::prelude::*;

trait MapBuilder {
    fn build(depth: i32) -> (Map, Position);
}

pub fn build_random_map(depth: i32) -> (Map, Position) {
    SimpleMapBuilder::build(depth)
}
