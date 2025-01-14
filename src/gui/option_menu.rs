use super::{menu_box, menu_option};
use crate::State;
use bracket_lib::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum OptionMenuResult {
    NoResponse,
    Continue,
    SaveAndExit,
    ExitWithoutSaving,
}

pub fn show_option_menu(_gs: &mut State, ctx: &mut BTerm) -> OptionMenuResult {
    let mut draw_batch = DrawBatch::new();
    let count = 4;
    let mut y = (25 - (count / 2)) as i32;
    menu_box(&mut draw_batch, 15, y, (count + 3) as i32, "Options");
    draw_batch.print_color(
        Point::new(18, y + count as i32 + 1),
        "ESCAPE to cancel",
        ColorPair::new(
            RGB::named(bracket_lib::terminal::YELLOW),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
    );

    menu_option(
        &mut draw_batch,
        17,
        y,
        bracket_lib::prelude::to_cp437('C'),
        "Continue",
    );
    y += 1;
    menu_option(
        &mut draw_batch,
        17,
        y,
        bracket_lib::prelude::to_cp437('S'),
        "Save and exit",
    );
    y += 1;
    menu_option(
        &mut draw_batch,
        17,
        y,
        bracket_lib::prelude::to_cp437('E'),
        "Exit without saving",
    );

    let _ = draw_batch.submit(6000);

    match ctx.key {
        None => OptionMenuResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::C => OptionMenuResult::Continue,
            VirtualKeyCode::S => OptionMenuResult::SaveAndExit,
            VirtualKeyCode::E => OptionMenuResult::ExitWithoutSaving,
            VirtualKeyCode::Escape => OptionMenuResult::Continue,
            _ => OptionMenuResult::NoResponse,
        },
    }
}
