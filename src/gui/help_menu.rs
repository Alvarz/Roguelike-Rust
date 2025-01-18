use super::{menu_box, menu_option};
use crate::State;
use bracket_lib::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum HelpMenuResult {
    NoResponse,
    Cancel,
}

pub fn show_help_menu(_gs: &mut State, ctx: &mut BTerm) -> HelpMenuResult {
    let mut draw_batch = DrawBatch::new();
    let count = 10;
    let mut y = (25 - (count / 2)) as i32;
    menu_box(&mut draw_batch, 15, y, (count + 3) as i32, "Help!");
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
        bracket_lib::prelude::to_cp437('*'),
        "Movement: To move on any direction or to attack, use numpad ( try Numlock off and on ) or vi keys",
    );
    y += 1;
    menu_option(
        &mut draw_batch,
        17,
        y,
        bracket_lib::prelude::to_cp437('*'),
        "Use 'I' no open the inventory",
    );
    y += 1;
    menu_option(
        &mut draw_batch,
        17,
        y,
        bracket_lib::prelude::to_cp437('*'),
        "Use 'G' to grab items",
    );

    y += 1;
    menu_option(
        &mut draw_batch,
        17,
        y,
        bracket_lib::prelude::to_cp437('*'),
        "Use 'D' to drop items",
    );
    y += 1;
    menu_option(
        &mut draw_batch,
        17,
        y,
        bracket_lib::prelude::to_cp437('*'),
        "Use 'R' to remove items",
    );
    y += 1;
    menu_option(
        &mut draw_batch,
        17,
        y,
        bracket_lib::prelude::to_cp437('*'),
        "'Shift + <number>' to use consumable from hot keys",
    );

    y += 1;
    menu_option(
        &mut draw_batch,
        17,
        y,
        bracket_lib::prelude::to_cp437('*'),
        "'Control + <number>' to use skills from hot keys",
    );

    y += 1;
    menu_option(
        &mut draw_batch,
        17,
        y,
        bracket_lib::prelude::to_cp437('*'),
        "List mobs current map",
    );

    y += 1;
    menu_option(
        &mut draw_batch,
        17,
        y,
        bracket_lib::prelude::to_cp437('*'),
        "5 or Space to skip a turn",
    );
    y += 1;
    menu_option(
        &mut draw_batch,
        17,
        y,
        bracket_lib::prelude::to_cp437('*'),
        "Escape to open save menu",
    );

    let _ = draw_batch.submit(6000);

    match ctx.key {
        None => HelpMenuResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::Escape => HelpMenuResult::Cancel,
            _ => HelpMenuResult::NoResponse,
        },
    }
}
