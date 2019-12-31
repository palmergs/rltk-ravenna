extern crate rltk;
use rltk::{ VirtualKeyCode, RGB, Rltk, Console };

use super::{
    State,
    RunState,
    MainMenuSelection,
    MainMenuResult, };


pub fn main_menu(gs: &mut State, ctx: &mut Rltk) -> MainMenuResult {
    let runstate = gs.ecs.fetch::<RunState>();
    ctx.print_color_centered(15, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Rust Roguelike Tutorial");

    if let RunState::MainMenu { menu_selection: selection } = *runstate {
        toggle_text(ctx, selection, MainMenuSelection::NewGame, 24, "Begin New Game");
        toggle_text(ctx, selection, MainMenuSelection::LoadGame, 25, "Load Game");
        toggle_text(ctx, selection, MainMenuSelection::Quit, 26, "Quit");

        match ctx.key {
            None => return MainMenuResult::NoSelection { selected: selection },
            Some(key) => {
                match key {
                    VirtualKeyCode::Escape => { return MainMenuResult::NoSelection { selected: MainMenuSelection::Quit } }
                    VirtualKeyCode::Up => {
                        let newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::NewGame,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::LoadGame
                        }
                        return MainMenuResult::NoSelection { selected: newselection }
                    }
                    VirtualKeyCode::Down => {
                        let newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::LoadGame,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame
                        }
                        return MainMenuResult::NoSelection { selected: newselection }
                    }
                    VirtualKeyCode::Return => { return MainMenuResult::Selected { selected: selection } }
                    _ => { return MainMenuResult::NoSelection { selected: selection } }
                }
            }
        }
    }

    MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame }
}

fn toggle_text(ctx: &mut Rltk, selection: MainMenuSelection, current: MainMenuSelection, y: i32, label: &str) {
    if selection == current {
        ctx.print_color_centered(
            y,
            RGB::named(rltk::MAGENTA),
            RGB::named(rltk::BLACK),
            label);
    } else {
        ctx.print_color_centered(
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            label);
    }
}
