use rltk::{ RGB };
use specs::prelude::*;

#[derive(Component, Debug)]
pub struct Player {}

#[derive(Component)]
pub struct Monster {}

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Debug)]
pub struct Name {
    pub name: String,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: u8,
    pub fg: RGB,
    pub bg: RGB,
}

#[derive(Component)]
pub struct Viewshed {
    pub tiles : Vec<rltk::Point>,
    pub range : i32,
    pub dirty : bool,
}
