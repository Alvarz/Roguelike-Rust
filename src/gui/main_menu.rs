use crate::{rex_assets::RexAssets, RunState, State, PROJECT_NAME};
use bracket_lib::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

pub fn main_menu(gs: &mut State, ctx: &mut BTerm) -> MainMenuResult {
    let mut draw_batch = DrawBatch::new();
    let save_exists = crate::saveload::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();
    let assets = gs.ecs.fetch::<RexAssets>();
    ctx.render_xp_sprite(&assets.menu, 0, 0);

    draw_batch.draw_double_box(
        Rect::with_size(24, 18, 31, 10),
        ColorPair::new(
            RGB::named(bracket_lib::terminal::WHEAT),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
    );

    draw_batch.print_color_centered(
        20,
        PROJECT_NAME,
        ColorPair::new(
            RGB::named(bracket_lib::terminal::YELLOW),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
    );
    draw_batch.print_color_centered(
        21,
        "by Carlos Alvarez",
        ColorPair::new(
            RGB::named(bracket_lib::terminal::CYAN),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
    );
    draw_batch.print_color_centered(
        22,
        "Use Up/Down Arrows and Enter",
        ColorPair::new(
            RGB::named(bracket_lib::terminal::GRAY),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
    );

    let mut y = 24;
    if let RunState::MainMenu {
        menu_selection: selection,
    } = *runstate
    {
        if selection == MainMenuSelection::NewGame {
            draw_batch.print_color_centered(
                y,
                "Begin New Game",
                ColorPair::new(
                    RGB::named(bracket_lib::terminal::MAGENTA),
                    RGB::named(bracket_lib::terminal::BLACK),
                ),
            );
        } else {
            draw_batch.print_color_centered(
                y,
                "Begin New Game",
                ColorPair::new(
                    RGB::named(bracket_lib::terminal::WHITE),
                    RGB::named(bracket_lib::terminal::BLACK),
                ),
            );
        }
        y += 1;

        if save_exists {
            if selection == MainMenuSelection::LoadGame {
                draw_batch.print_color_centered(
                    y,
                    "Load Game",
                    ColorPair::new(
                        RGB::named(bracket_lib::terminal::MAGENTA),
                        RGB::named(bracket_lib::terminal::BLACK),
                    ),
                );
            } else {
                draw_batch.print_color_centered(
                    y,
                    "Load Game",
                    ColorPair::new(
                        RGB::named(bracket_lib::terminal::WHITE),
                        RGB::named(bracket_lib::terminal::BLACK),
                    ),
                );
            }
            y += 1;
        }

        if selection == MainMenuSelection::Quit {
            draw_batch.print_color_centered(
                y,
                "Quit",
                ColorPair::new(
                    RGB::named(bracket_lib::terminal::MAGENTA),
                    RGB::named(bracket_lib::terminal::BLACK),
                ),
            );
        } else {
            draw_batch.print_color_centered(
                y,
                "Quit",
                ColorPair::new(
                    RGB::named(bracket_lib::terminal::WHITE),
                    RGB::named(bracket_lib::terminal::BLACK),
                ),
            );
        }

        let _ = draw_batch.submit(6000);

        match ctx.key {
            None => {
                return MainMenuResult::NoSelection {
                    selected: selection,
                }
            }
            Some(key) => match key {
                VirtualKeyCode::Escape => {
                    return MainMenuResult::NoSelection {
                        selected: MainMenuSelection::Quit,
                    }
                }
                VirtualKeyCode::Up => {
                    let mut newselection;
                    match selection {
                        MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                        MainMenuSelection::LoadGame => newselection = MainMenuSelection::NewGame,
                        MainMenuSelection::Quit => newselection = MainMenuSelection::LoadGame,
                    }
                    if newselection == MainMenuSelection::LoadGame && !save_exists {
                        newselection = MainMenuSelection::NewGame;
                    }
                    return MainMenuResult::NoSelection {
                        selected: newselection,
                    };
                }
                VirtualKeyCode::Down => {
                    let mut newselection;
                    match selection {
                        MainMenuSelection::NewGame => newselection = MainMenuSelection::LoadGame,
                        MainMenuSelection::LoadGame => newselection = MainMenuSelection::Quit,
                        MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame,
                    }
                    if newselection == MainMenuSelection::LoadGame && !save_exists {
                        newselection = MainMenuSelection::Quit;
                    }
                    return MainMenuResult::NoSelection {
                        selected: newselection,
                    };
                }
                VirtualKeyCode::Return => {
                    return MainMenuResult::Selected {
                        selected: selection,
                    }
                }
                _ => {
                    return MainMenuResult::NoSelection {
                        selected: selection,
                    }
                }
            },
        }
    }

    MainMenuResult::NoSelection {
        selected: MainMenuSelection::NewGame,
    }
}
