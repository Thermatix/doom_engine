use std::collections::HashMap;
use std::hash::Hash;
use std::sync::OnceLock;

use super::*;
use sdl2::gfx::primitives::DrawRenderer;

pub static mut META: OnceLock<HashMap<&'static str, usize>> = OnceLock::new();


pub fn meta() -> &'static HashMap<&'static str, usize> {
    unsafe {
        META.get_or_init(|| HashMap::new())
    }
}


pub fn get_mutmeta() -> &'static mut HashMap<&'static str, usize> {
    unsafe {
        META.get_or_init(|| HashMap::new() );
        META.get_mut().unwrap()
    }
}

pub fn set_meta(k: &'static str, v: usize, ovewrite: bool) {
    let meta = get_mutmeta();

    if ovewrite || meta.get(k) == None {
        meta.insert(k, v);
    }


}



pub type Layers<Draw> = Vec<Layer<Draw>>;

pub struct Layer<M> where M: Manager {
    pub name: String,
    pub draw_function: Box<dyn Fn(&mut Canvas<Window>, &Context, &M)>,
}

impl<M: Manager> Drawable for Layer<M> {
    type Manager = M;
    fn draw(&self, canvas: &mut Canvas<Window>, context: &Context, manager: &Self::Manager) {
        (&self.draw_function.as_ref())(canvas, context, manager);
    }

}


pub trait Drawable {
    type Manager;
    fn draw(&self, canvas: &mut Canvas<Window>, context: &Context, manager: &Self::Manager); 
}


pub trait Manager {
    fn screen_width(&self) -> i16;
    fn screen_height(&self) -> i16;
    fn draw_layers(&self, canvas: &mut Canvas<Window>, context: &Context);
}

pub struct Draw2D  {
    screen_width: i16,
    screen_height: i16,
    layers: Layers<Self>,
}

impl Manager for Draw2D {
    fn screen_width(&self) -> i16 {
        self.screen_width
    }
    fn screen_height(&self) -> i16 {
        self.screen_height
    }

    fn draw_layers(&self, canvas: &mut Canvas<Window>, context: &Context) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        //canvas.clear();
        for layer in self.layers.iter() {
            layer.draw(canvas, context, self);
        };
        
    }
}



impl Draw2D {
    pub fn new( screen_width: i16, screen_height: i16) -> Self {
        let layers = Layers::new();
        let mut draw_2d = Self {
            screen_width,
            screen_height,
            layers,
        };

        draw_2d.layers.push(
            Layer {
                name: "map-lines".to_string(),
                draw_function: Box::new(draw_map_lines),
            }
        );

        draw_2d.layers.push(
            Layer {
                name: "map-vertexes".to_string(),
                draw_function: Box::new(draw_map_vertexes),
            }
        );

        draw_2d.layers.push(
            Layer {
                name: "map-bsp".to_string(),
                draw_function: Box::new(draw_map_bsp),
            }
        );

        draw_2d.layers.push(
            Layer { 
                name: "player".to_string(),
                draw_function: Box::new(draw_player),
            }
        );

        draw_2d
    }
}

pub fn draw_player<M: Manager>(canvas: &mut Canvas<Window>,  context: &Context, manager: &M ) {
    if let Some(v) = meta().get("don't_draw_vertexes") { if *v == 0 { return } } else { return };
    
    let player = &context.player;   
    let map = &context.current_map;

    let scaled_pos = map_utils::scale_xy(
        player.x,
        player.y,
        &map.map_bounds(),
        (manager.screen_width(), manager.screen_height()),
        30,
    );

    canvas.filled_circle(scaled_pos.0, scaled_pos.1, 5, Color::GREEN).unwrap();
    set_meta("player_is_drawn", 1, false);

}

fn draw_map_bsp<M: Manager>(canvas: &mut Canvas<Window>,  context: &Context, manager: &M ) {
    //if let Some(v) = meta().get("player_is_drawn") { if *v == 0 { return } } else { return };

    let map = &context.current_map;
    let player = &context.player;
    let bsp = &context.bsp;

    let bounds = (manager.screen_width(), manager.screen_height());
    let boarder: i16 = 30;

    for node_or_leaf in bsp.traverse((player.x, player.y)) {
        if let bsp::NodeType::Node(node) = node_or_leaf {
            let ((fx, fy), (fw, fh)) = get_bounding_box(&node.front_bbox, &map,bounds, boarder);
            let ((bx, by), (bw, bh)) = get_bounding_box(&node.back_bbox, &map,bounds, boarder);

            let (p1x, p1y) = map_utils::scale_xy(node.x_partion, node.y_partion, map.map_bounds(), bounds, boarder);
            let (p2x, p2y) = map_utils::scale_xy(node.dx_partion + node.x_partion, node.dy_partion + node.y_partion, map.map_bounds(), bounds, boarder);

            canvas.rectangle(fx, fy, fw, fh, Color::GREEN).unwrap();
            canvas.rectangle(bx, by, bw, bh, Color::RED).unwrap();

            canvas.thick_line(p1x, p1y, p2x, p2y,3, Color::BLUE).unwrap();
        }
    }
}

fn get_bounding_box(bbox: &wad::BoundingBox, map: &wad::Map, bounds: (i16, i16), boarder: i16) -> ((i16, i16), ((i16, i16))) {
    (
        map_utils::scale_xy(bbox.x, bbox.y,  map.map_bounds(), bounds, 30),
        map_utils::scale_xy(bbox.w, bbox.h,  map.map_bounds(), bounds, 30)
    )
}

fn  draw_map_vertexes<M: Manager>(canvas: &mut Canvas<Window>,  context: &Context, manager: &M ) {
    set_meta("don't_draw_vertexes", 0, false);
    if meta()["don't_draw_vertexes"] >= 1 { return };
    
    let map = &context.current_map;

    let points = map_utils::scale_map_points(
        map.map_points(),
        map.map_bounds(),
        (manager.screen_width(), manager.screen_height()),
        30
    );

    for (p1, p2) in map.line_defs_to_vertexes(Some(&points)) {
        canvas.filled_circle(p1.0, p1.1 , 2, Color::YELLOW).unwrap();
        canvas.filled_circle(p2.0, p2.1 , 2, Color::YELLOW).unwrap();
    }

    set_meta("don't_draw_vertexes", 1, true);

}

fn draw_map_lines<M: Manager>(canvas: &mut Canvas<Window>,  context: &Context, manager: &M ) {
    set_meta("don't_draw_lines", 0, false);
    if meta()["don't_draw_lines"] >= 1 { return };

    let map = &context.current_map;

    let points = map_utils::scale_map_points(
        map.map_points(),
        map.map_bounds(),
        (manager.screen_width(), manager.screen_height()),
        30
    );

    for (p1, p2) in map.line_defs_to_vertexes(Some(&points)) {
        canvas.thick_line(p1.0, p1.1, p2.0, p2.1,3, Color::GREY).unwrap();
    }

    set_meta("don't_draw_lines", 1, true);
}

mod map_utils {
    use super::*;


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
}