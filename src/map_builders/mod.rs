use super::{ 
    Map,
    World,
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
    fn spawn(map: &mut Map, ecs: &mut World, depth: i32);
}

pub fn build_random_map(depth: i32) -> (Map, Position) {
    SimpleMapBuilder::build(depth)
}

pub fn spawn(map: &mut Map, ecs: &mut World, depth: i32) {
    SimpleMapBuilder::spawn(map, ecs, depth);
}
