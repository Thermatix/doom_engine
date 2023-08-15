
use super::*;

pub static NUMBERS: OnceLock<RwLock<HashMap<String, usize>>> = OnceLock::new();

/// A store for numbers between drawing iterations
pub fn numbers() -> RwLockWriteGuard<'static, std::collections::HashMap<std::string::String, usize>> {
    NUMBERS.get_or_init(|| RwLock::new(HashMap::new())).write().unwrap()
}



pub fn scale_map_points(map_points: &wad::Points, map_bounds: &wad::P1P2, max_bounds: wad::Point, boarder: i16) -> wad::Points {
    map_points.iter().map(|(x, y)| {
        scale_xy(*x, *y, &map_bounds, max_bounds, boarder)
    }).collect()
}

#[inline]
pub fn scale_xy(x: i16, y: i16,  map_bounds: &wad::P1P2, max_bounds: wad::Point, boarder: i16) -> wad::Point {
    let (max_width, max_height) = max_bounds;
    let ((x_min, x_max),(y_min, y_max)) = map_bounds;
    (
        scale_x(*x_min as i32, *x_max as i32, x as i32, boarder as i32, (max_width - boarder) as i32) as i16,
        scale_y(*y_min as i32, *y_max as i32, y as i32, boarder as i32, (max_height - boarder) as i32 , max_height  as i32) as i16
    )
}
#[inline]
pub fn scale_x(x_min: i32, x_max: i32, n: i32, out_min: i32, out_max: i32) -> i32 {
    (x_min.max(x_max.min(n)) - x_min) * (out_max - out_min) / (x_max - x_min) + out_min
}

#[inline]
pub fn scale_y(y_min: i32, y_max: i32, n: i32, out_min: i32, out_max: i32, screen_height: i32) -> i32 {
    screen_height - (y_min.max(y_max.min(n)) - y_min) * (out_max - out_min) / (y_max - y_min) - out_min
}
