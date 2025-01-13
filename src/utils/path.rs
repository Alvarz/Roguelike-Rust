use rltk::NavigationPath;

use crate::Map;

fn bresenham_search(
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    map: &Map,
) -> NavigationPath {
    let mut line = rltk::line2d(
        rltk::LineAlg::Bresenham,
        rltk::Point::new(start_x, start_y),
        rltk::Point::new(end_x, end_y),
    );
    let mut result = NavigationPath::new();

    let end_pt = map.xy_idx(end_x, end_y);

    result.success = true;
    result.destination = end_pt;
    result.steps.push(end_pt);
    let _ = line.remove(0);

    for point in line.iter() {
        if crate::spatial::is_blocked(map.xy_idx(point.x, point.y)) {
            result.success = false;
            return result;
        }
        result.steps.push(map.xy_idx(point.x, point.y));
    }

    result
}

pub fn get_path(
    start_x: i32,
    start_y: i32,
    target_x: i32,
    target_y: i32,
    map: &mut Map,
) -> NavigationPath {
    let path = bresenham_search(start_x, start_y, target_x, target_y, map);

    if path.success && path.steps.len() > 1 {
        // rltk::console::log(format!(
        //     "Using bresenham for path - it succeeded? {:?}",
        //     path.success
        // ));
        return path;
    }

    let path = rltk::a_star_search(
        map.xy_idx(start_x, start_y),
        map.xy_idx(target_x, target_y),
        &mut *map,
    );

    // rltk::console::log(format!(
    //     "Using A* for path - it succeeded? {:?}",
    //     path.success
    // ));
    path
}
