use bracket_lib::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum FinishGameResult {
    NoSelection,
    QuitToMenu,
}

pub fn finish_game(ctx: &mut BTerm) -> FinishGameResult {
    let mut draw_batch = DrawBatch::new();
    draw_batch.print_color_centered(
        15,
        "Your journey has ended!",
        ColorPair::new(
            RGB::named(bracket_lib::terminal::YELLOW),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
    );
    draw_batch.print_color_centered(
        17,
        "One day, we'll tell you all about how you did.",
        ColorPair::new(
            RGB::named(bracket_lib::terminal::WHITE),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
    );
    draw_batch.print_color_centered(
        18,
        "That day, sadly, is not in this chapter..",
        ColorPair::new(
            RGB::named(bracket_lib::terminal::WHITE),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
    );

    draw_batch.print_color_centered(
        19,
        &format!(
            "You lived for {} turns.",
            crate::gamelog::get_event_count("Turn")
        ),
        ColorPair::new(
            RGB::named(bracket_lib::terminal::WHITE),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
    );
    draw_batch.print_color_centered(
        20,
        &format!(
            "You suffered {} points of damage.",
            crate::gamelog::get_event_count("Damage Taken")
        ),
        ColorPair::new(
            RGB::named(bracket_lib::terminal::RED),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
    );
    draw_batch.print_color_centered(
        21,
        &format!(
            "You inflicted {} points of damage.",
            crate::gamelog::get_event_count("Damage Inflicted")
        ),
        ColorPair::new(
            RGB::named(bracket_lib::terminal::RED),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
    );

    draw_batch.print_color_centered(
        23,
        "Press any key to return to the menu.",
        ColorPair::new(
            RGB::named(bracket_lib::terminal::MAGENTA),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
    );

    let _ = draw_batch.submit(6000);

    match ctx.key {
        None => FinishGameResult::NoSelection,
        Some(_) => FinishGameResult::QuitToMenu,
    }
}
