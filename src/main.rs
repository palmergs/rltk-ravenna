#![recursion_limit="16"]

rltk::add_wasm_support!();
use rltk::{Console, GameState, Rltk, RGB, Point};
use specs::prelude::*;

mod components;
pub use components::*;

mod map;
pub use map::*;

mod player;
use player::*;

mod rect;
pub use rect::Rect;

mod visibility_system;
use visibility_system::VisibilitySystem;

mod monster_ai_system;
use monster_ai_system::MonsterAI;

mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;

#[macro_use]
extern crate specs_derive;

#[derive(PartialEq, Clone, Copy)]
pub enum RunState { Paused, Running }

pub struct State {
    pub ecs: World,
    pub runstate: RunState,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        if self.runstate == RunState::Running {
            self.run_systems();
            self.runstate = RunState::Paused;
        } else {
            self.runstate = player_input(self, ctx);
        }

        draw_map(&self.ecs, ctx);

        let map = self.ecs.fetch::<Map>();
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = Map::xy_idx(pos.x, pos.y);
            if map.visible[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);

        let mut mob = MonsterAI{};
        mob.run_now(&self.ecs);

        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

fn main() {
    let context = Rltk::init_simple8x8(80, 50, "Hello Rust World!", "resources");
    let mut gs = State { 
        ecs: World::new(), 
        runstate: RunState::Running,
    };

    gs.ecs.register::<Player>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();

    let map = Map::new_map_rooms_and_corridors();
    let (px, py) = map.rooms[0].center();

    gs.ecs
        .create_entity()
        .with(Position { x: px, y: py })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Viewshed { tiles: Vec::new(), range: 8, dirty: true })
        .with(Name { name: "Player".to_string() })
        .build();

    let mut rng = rltk::RandomNumberGenerator::new();
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();

        let glyph : u8;
        let name : String;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => { 
                glyph = rltk::to_cp437('g');
                name = "Goblin".to_string();
            },
            _ => { 
                glyph = rltk::to_cp437('o');
                name = "Orc".to_string();
            },
        }

        gs.ecs
            .create_entity()
            .with(Position { x: x, y: y })
            .with(Renderable {
                glyph: glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed { tiles: Vec::new(), range: 8, dirty: true })
            .with(Monster {})
            .with(Name { name: format!("{} #{}", name, i) })
            .with(BlocksTile {})
            .build();
    }

    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(px, py));

    rltk::main_loop(context, gs);
}
