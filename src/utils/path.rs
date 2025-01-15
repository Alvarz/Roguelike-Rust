use bracket_lib::prelude::NavigationPath;
use specs::Entity;

use crate::Map;

fn bresenham_search(
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    map: &Map,
    entity: &Entity,
) -> NavigationPath {
    let mut line = bracket_lib::prelude::line2d(
        bracket_lib::prelude::LineAlg::Bresenham,
        bracket_lib::prelude::Point::new(start_x, start_y),
        bracket_lib::prelude::Point::new(end_x, end_y),
    );
    let mut result = NavigationPath::new();

    let end_pt = map.xy_idx(end_x, end_y);

    result.success = true;
    result.destination = end_pt;
    result.steps.push(end_pt);
    let _ = line.remove(0);

    for point in line.iter() {
        let point = map.xy_idx(point.x, point.y);
        if !crate::spatial::is_blocked(point) {
            result.steps.push(point);
            continue;
        }

        let blocking_entities = crate::spatial::get_tile_content_clone(point);
        // bracket_lib::prelude::console::log(format!(
        //     "entities that are blocking {:?}",
        //     blocking_entities
        // ));

        if blocking_entities.len() == 1 {
            let blocking_entity = blocking_entities.first().unwrap();
            if blocking_entity == entity {
                result.steps.push(point);
                continue;
            }
        }

        result.success = false;
        return result;
    }

    result
}

pub fn get_path(
    start_x: i32,
    start_y: i32,
    target_x: i32,
    target_y: i32,
    map: &mut Map,
    entity: &Entity,
) -> NavigationPath {
    let path = bresenham_search(start_x, start_y, target_x, target_y, map, entity);

    if path.success && path.steps.len() > 1 {
        // bracket_lib::prelude::console::log(format!(
        //     "Using bresenham for path - it succeeded? {:?}",
        //     path.success
        // ));
        return path;
    }

    let path = bracket_lib::prelude::a_star_search(
        map.xy_idx(start_x, start_y),
        map.xy_idx(target_x, target_y),
        &mut *map,
    );

    // bracket_lib::prelude::console::log(format!(
    //     "Using A* for path - it succeeded? {:?}",
    //     path.success
    // ));
    path
}
