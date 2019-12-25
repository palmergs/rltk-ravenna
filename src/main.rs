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

mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;

mod damage_system;
use damage_system::DamageSystem;

#[macro_use]
extern crate specs_derive;

#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn }

pub struct State {
    pub ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }

            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }

            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }

            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        damage_system::delete_the_dead(&mut self.ecs);
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

        let mut melee_combat = MeleeCombatSystem{};
        melee_combat.run_now(&self.ecs);

        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

fn main() {
    let context = Rltk::init_simple8x8(80, 50, "Hello Rust World!", "resources");
    let mut gs = State { ecs: World::new() };

    gs.ecs.register::<Player>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();

    let map = Map::new_map_rooms_and_corridors();
    let (px, py) = map.rooms[0].center();

    let player_entity = gs.ecs
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
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
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
            .with(Position { x, y })
            .with(Renderable {
                glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed { tiles: Vec::new(), range: 8, dirty: true })
            .with(Monster {})
            .with(Name { name: format!("{} #{}", name, i) })
            .with(CombatStats {
                max_hp: 16,
                hp: 16,
                defense: 1,
                power: 4,
            })
            .with(BlocksTile {})
            .build();
    }

    gs.ecs.insert(map);
    gs.ecs.insert(player_entity);
    gs.ecs.insert(Point::new(px, py));
    gs.ecs.insert(RunState::PreRun);

    rltk::main_loop(context, gs);
}
