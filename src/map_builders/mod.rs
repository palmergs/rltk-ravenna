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

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&self) -> Map;
    fn get_starting_pos(&self) -> Position;
}

pub fn random_builder(depth: i32) -> Box<dyn MapBuilder> {
    Box::new(SimpleMapBuilder::new(depth))
}

