#![recursion_limit="16"]

extern crate rltk;
rltk::add_wasm_support!();
use rltk::{Console, GameState, Rltk, Point};

extern crate specs;
use specs::prelude::*;
use specs::saveload::{ SimpleMarker, SimpleMarkerAllocator };

extern crate serde;

mod components;
pub use components::*;

mod map;
pub use map::*;

mod gui;
mod menu;
mod saveload_system;

pub mod map_builders;

mod gamelog;
use gamelog::*;

mod player;
use player::*;

mod rect;
pub use rect::Rect;

mod random_table;
mod spawner;

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

mod inventory_system;
use inventory_system::ItemCollectionSystem;
use inventory_system::ItemDropSystem;
use inventory_system::ItemUseSystem;

mod particle_system;

#[macro_use]
extern crate specs_derive;

#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    MainMenu { menu_selection: gui::MainMenuSelection },
    SaveGame,
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range: i32, item: Entity },
    NextLevel,
    GameOver }

pub struct State {
    pub ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        ctx.cls();
        particle_system::cull_dead_particles(&mut self.ecs, ctx);

        match newrunstate {
            RunState::MainMenu {..} => {}

            _ => {
                draw_map(&self.ecs, ctx);

                {
                    let map = self.ecs.fetch::<Map>();
                    let positions = self.ecs.read_storage::<Position>();
                    let renderables = self.ecs.read_storage::<Renderable>();

                    let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
                    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order) );
                    for (pos, render) in data.iter() {
                        let idx = Map::xy_idx(pos.x, pos.y);
                        if map.visible[idx] {
                            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                        }
                    }

                    gui::draw_ui(&self.ecs, ctx);
                }
            }
        }


        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }

            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }

            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::MonsterTurn;
            }

            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }

            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {},
                    gui::ItemMenuResult::Selected => {
                        let item = result.1.unwrap();
                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        let is_item_ranged = is_ranged.get(item);
                        if let Some(is_item_ranged) = is_item_ranged {
                            newrunstate = RunState::ShowTargeting {
                                range: is_item_ranged.range, 
                                item: item
                            };
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem{ item, target: None }).expect("Unable to insert intent");
                            newrunstate = RunState::PlayerTurn;
                        }
                    }
                }
            }

            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {},
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToDropItem{ item: item_entity }).expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }

            RunState::ShowTargeting { range, item } => {
                let target = gui::ranged_target(self, ctx, range);
                match target.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {},
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem{ item, target: target.1 }).expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }

            RunState::MainMenu {..} => {
                let result = menu::main_menu(self, ctx);
                match result {
                    gui::MainMenuResult::NoSelection { selected } => newrunstate = RunState::MainMenu { menu_selection: selected },
                    gui::MainMenuResult::Selected { selected } => {
                        match selected {
                            gui::MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
                            gui::MainMenuSelection::LoadGame => {
                                saveload_system::load_game(&mut self.ecs);
                                newrunstate = RunState::AwaitingInput;
                                saveload_system::delete_save();
                            }
                            gui::MainMenuSelection::Quit => { ::std::process::exit(0); }
                        }
                    }
                }
            }

            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu { menu_selection: gui::MainMenuSelection::LoadGame };
            }

            RunState::NextLevel => {
                self.descend();
                newrunstate = RunState::PreRun;
            }

            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::GameOverResult::NoSelection => {},
                    gui::GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();
                        newrunstate = RunState::MainMenu { menu_selection: gui::MainMenuSelection::NewGame };
                    }
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }


        damage_system::delete_the_dead(&mut self.ecs);
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

        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);

        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);

        let mut pickup = ItemCollectionSystem{};
        pickup.run_now(&self.ecs);

        let mut drop = ItemDropSystem{};
        drop.run_now(&self.ecs);

        let mut consumed = ItemUseSystem{};
        consumed.run_now(&self.ecs);

        let mut particles = particle_system::ParticleSpawnSystem{};
        particles.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let player_entity = self.ecs.fetch::<Entity>();
        let equipment = self.ecs.read_storage::<Equipped>();

        let mut to_delete : Vec<Entity> = Vec::new();
        for entity in entities.join() {
            let mut should_delete = true;
            
            // don't delete the player
            let p = player.get(entity);
            if let Some(_p) = p { should_delete = false; }

            // don't delete the player's equipment
            let bp = backpack.get(entity);
            if let Some(bp) = bp { 
                if bp.owner == *player_entity { should_delete = false; }
            }

            let eq = equipment.get(entity);
            if let Some(eq) = eq {
                if eq.owner == *player_entity { should_delete = false; }
            }

            if should_delete { to_delete.push(entity); }
        }

        to_delete
    }

    fn descend(&mut self) {
        // delete the entities that aren't the player or equipment
        let to_delete = self.entities_to_remove_on_level_change();
        for target in to_delete {
            self.ecs.delete_entity(target).expect("Unable to delete entity");
        }

        // build a new map and place the player
        let mut builder;
        let worldmap;
        {
            let mut worldmap_resource = self.ecs.write_resource::<Map>();
            builder = map_builders::random_builder(worldmap_resource.depth + 1);
            builder.build_map();
            *worldmap_resource = builder.get_map();
            worldmap = worldmap_resource.clone();
        }
        
        // spawn some bad gusys
        builder.spawn_entities(&mut self.ecs);

        // place the player and update resources
        let start = builder.get_starting_pos();
        let mut pos = self.ecs.write_resource::<Point>();
        *pos = Point::new(start.x, start.y);
        let mut positions = self.ecs.write_storage::<Position>();
        let player_entity = self.ecs.fetch::<Entity>();
        let player_pos_comp = positions.get_mut(*player_entity);
        if let Some(player_pos_comp) = player_pos_comp {
            player_pos_comp.x = start.x;
            player_pos_comp.y = start.y;
        }

        // mark the player's visibility as dirty
        let mut viewsheds = self.ecs.write_storage::<Viewshed>();
        let vs = viewsheds.get_mut(*player_entity);
        if let Some(vs) = vs { vs.dirty = true; }

        // notify the player and give some health
        let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
        gamelog.entries.insert(0, "You descend to the next level and take a moment to heal".to_string());
        let mut stats = self.ecs.write_storage::<CombatStats>();
        let player_health = stats.get_mut(*player_entity);
        if let Some(player_health) = player_health {
            player_health.hp = i32::max(player_health.hp, player_health.max_hp / 2);
        }
    }

    fn game_over_cleanup(&mut self) {
        let mut to_delete = Vec::new();
        for e in self.ecs.entities().join() { to_delete.push(e); }
        for d in to_delete.iter() {
            self.ecs.delete_entity(*d).expect("deletion failed");
        }

        let mut builder;
        let worldmap;
        {
            let mut worldmap_resource = self.ecs.write_resource::<Map>();
            builder = map_builders::random_builder(1);
            builder.build_map();
            *worldmap_resource = builder.get_map();
            worldmap = worldmap_resource.clone();
        }

        // Spawn bad guys
        builder.spawn_entities(&mut self.ecs);

        // place the payer and update resources
        let start = builder.get_starting_pos();
        let player = spawner::player(&mut self.ecs, start.x, start.y);
        let mut pos = self.ecs.write_resource::<Point>();
        *pos = Point::new(start.x, start.y);
        let mut positions = self.ecs.write_storage::<Position>();
        let mut player_entity_writer = self.ecs.write_resource::<Entity>();
        *player_entity_writer = player;
        let player_pos_comp = positions.get_mut(player);
        if let Some(player_pos_comp) = player_pos_comp {
            player_pos_comp.x = start.x;
            player_pos_comp.y = start.y;
        }

        // mark the player's visibility as dirty
        let mut viewsheds = self.ecs.write_storage::<Viewshed>();
        let vs = viewsheds.get_mut(player);
        if let Some(vs) = vs { vs.dirty = true; }
    }
}

fn main() {
    let mut context = Rltk::init_simple8x8(80, 50, "Hello Rust World!", "resources");
    context.with_post_scanlines(true);

    let mut gs = State { ecs: World::new() };

    gs.ecs.register::<Player>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<MeleePowerBonus>();
    gs.ecs.register::<DefenseBonus>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<ParticleLifetime>();

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    let mut builder = map_builders::random_builder(1);
    builder.build_map();
    let map = builder.get_map();
    let start_pos = builder.get_starting_pos();
    let (px, py) = (start_pos.x, start_pos.y);
    let player_entity = spawner::player(&mut gs.ecs, px, py);

    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    builder.spawn_entities(&mut gs.ecs);

    gs.ecs.insert(map);
    gs.ecs.insert(player_entity);
    gs.ecs.insert(Point::new(px, py));
    gs.ecs.insert(RunState::MainMenu { menu_selection: gui::MainMenuSelection::NewGame });
    gs.ecs.insert(gamelog::GameLog{ entries: vec!["Welcome to Rusty Roguelike".to_string()] });
    gs.ecs.insert(particle_system::ParticleBuilder::new());

    rltk::main_loop(context, gs);
}
