use super::ItemMenuResult;
use bracket_lib::prelude::*;
use specs::prelude::*;

pub fn menu_box<T: ToString>(draw_batch: &mut DrawBatch, x: i32, y: i32, width: i32, title: T) {
    draw_batch.draw_box(
        Rect::with_size(x, y - 2, 31, width),
        ColorPair::new(
            RGB::named(bracket_lib::terminal::WHITE),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
    );
    draw_batch.print_color(
        Point::new(18, y - 2),
        &title.to_string(),
        ColorPair::new(
            RGB::named(bracket_lib::terminal::MAGENTA),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
    );
}

pub fn menu_option<T: ToString>(
    draw_batch: &mut DrawBatch,
    x: i32,
    y: i32,
    hotkey: bracket_lib::prelude::FontCharType,
    text: T,
) {
    draw_batch.set(
        Point::new(x, y),
        ColorPair::new(
            RGB::named(bracket_lib::terminal::WHITE),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
        bracket_lib::prelude::to_cp437('('),
    );
    draw_batch.set(
        Point::new(x + 1, y),
        ColorPair::new(
            RGB::named(bracket_lib::terminal::YELLOW),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
        hotkey,
    );
    draw_batch.set(
        Point::new(x + 2, y),
        ColorPair::new(
            RGB::named(bracket_lib::terminal::WHITE),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
        bracket_lib::prelude::to_cp437(')'),
    );
    draw_batch.print_color(
        Point::new(x + 5, y),
        &text.to_string(),
        ColorPair::new(
            RGB::named(bracket_lib::terminal::YELLOW),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
    );
}

pub fn item_result_menu<S: ToString>(
    draw_batch: &mut DrawBatch,
    title: S,
    count: usize,
    items: &[(Entity, String)],
    key: Option<VirtualKeyCode>,
) -> (ItemMenuResult, Option<Entity>) {
    let mut y = (25 - (count / 2)) as i32;
    draw_batch.draw_box(
        Rect::with_size(15, y - 2, 31, (count + 3) as i32),
        ColorPair::new(
            RGB::named(bracket_lib::terminal::WHITE),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
    );
    draw_batch.print_color(
        Point::new(18, y - 2),
        &title.to_string(),
        ColorPair::new(
            RGB::named(bracket_lib::terminal::YELLOW),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
    );
    draw_batch.print_color(
        Point::new(18, y + count as i32 + 1),
        "ESCAPE to cancel",
        ColorPair::new(
            RGB::named(bracket_lib::terminal::YELLOW),
            RGB::named(bracket_lib::terminal::BLACK),
        ),
    );

    let mut item_list: Vec<Entity> = Vec::new();
    let mut j = 0;
    for item in items {
        menu_option(
            draw_batch,
            17,
            y,
            97 + j as bracket_lib::prelude::FontCharType,
            &item.1,
        );
        item_list.push(item.0);
        y += 1;
        j += 1;
    }

    match key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = bracket_lib::prelude::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        ItemMenuResult::Selected,
                        Some(item_list[selection as usize]),
                    );
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}
