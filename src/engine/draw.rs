use std::cell::{RefCell, Ref, RefMut};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::OnceLock;

use super::*;
//use super::bsp::NodeType;
use sdl2::gfx::primitives::DrawRenderer;

pub static mut SEG_COLOURS: OnceLock<HashMap<u16, Color>> = OnceLock::new();

pub type Flags<'a> = Ref<'a, HashMap<String, bool>>;
pub type MutFlags<'a> = RefMut<'a, HashMap<String, bool>>;
pub type Colours<'a> = Ref<'a, HashMap<u16, Color>>;
pub type MutColours<'a> = RefMut<'a, HashMap<u16, Color>>;
pub type Layers<Draw> = Vec<Layer<Draw>>;

pub struct Layer<M> where M: Manager {
    pub name: String,
    pub draw_function: Box<dyn for<'m, 'c> Fn(&'c mut Canvas<Window>, &'c Context, &'m M)>,
}

impl<M: Manager> Drawable for Layer<M> {
    type Manager = M;
    fn draw<'c, 'm>(&'m self, canvas: &'c mut Canvas<Window>, context: &'c Context, manager: &'m Self::Manager) {
        (&self.draw_function.as_ref())(canvas, context, manager);
    }

}

pub trait Drawable {
    type Manager;
    fn draw<'c, 'm>(&'m self, canvas: &'c mut Canvas<Window>, context: &'c Context, manager: &'m Self::Manager); 
}

pub trait Manager {
    fn screen_width(&self) -> i16;
    fn screen_height(&self) -> i16;
    fn draw_layers(&self, canvas: &mut Canvas<Window>, context: &Context);

}

trait FlagsData {
    fn mut_meta(&self) -> MutFlags;
    fn meta(&self) -> Flags;
}

trait ColoursStore {
    fn mut_colours(&self) -> MutColours;
    fn colours(&self) -> Colours;
}
pub struct Draw2D  {
    screen_width: i16,
    screen_height: i16,
    layers: Layers<Self>,
    meta: RefCell<HashMap<String, bool>>,
    colours: RefCell<HashMap<u16, Color>>,
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

impl FlagsData for Draw2D {
    fn mut_meta(&self) -> MutFlags{
        self.meta.borrow_mut()
    }

    fn meta(&self) -> Flags {
        self.meta.borrow()
    }
}

impl ColoursStore for Draw2D {
    fn mut_colours(&self) -> MutColours {
        self.colours.borrow_mut()
    }

    fn colours(&self) -> Colours {
        self.colours.borrow()
    }
}

impl Draw2D {
    pub fn new( screen_width: i16, screen_height: i16) -> Self {
        let layers = Layers::new();
        let mut draw_2d = Self {
            screen_width,
            screen_height,
            layers,
            meta: RefCell::new(HashMap::new()),
            colours: RefCell::new(HashMap::new()),
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

fn draw_player<'m, 'c, M: Manager + FlagsData + ColoursStore>(canvas: &'c mut Canvas<Window>,  context: &'c Context, manager: &'m M ) {    
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

}

fn draw_map_bsp<'m, 'c, M: Manager + FlagsData + ColoursStore>(canvas: &'c mut Canvas<Window>,  context: &'c Context, manager: &'m M ) {
    if manager.meta().get("don't_draw_bsp").is_some_and(|v| *v) { return };

    let map = &context.current_map;
    let player = &context.player;
    //let bsp = &context.bsp;

    let bounds = (manager.screen_width(), manager.screen_height());
    let boarder: i16 = 30;

    for node in map.traverse_bsp((player.x, player.y)) {
        let ((fx, fy), (fw, fh)) = get_bounding_box(&node.front_bbox, &map,bounds, boarder);
        let ((bx, by), (bw, bh)) = get_bounding_box(&node.back_bbox, &map,bounds, boarder);

        let (p1x, p1y) = map_utils::scale_xy(node.x_partion, node.y_partion, map.map_bounds(), bounds, boarder);
        let (p2x, p2y) = map_utils::scale_xy(node.dx_partion + node.x_partion, node.dy_partion + node.y_partion, map.map_bounds(), bounds, boarder);

        canvas.rectangle(fx, fy, fw, fh, Color::GREEN).unwrap();
        canvas.rectangle(bx, by, bw, bh, Color::RED).unwrap();

        canvas.thick_line(p1x, p1y, p2x, p2y,3, Color::BLUE).unwrap();
    }
    manager.mut_meta().insert("don't_draw_bsp".to_string(), true);
}

fn get_bounding_box(bbox: &wad::BoundingBox, map: &wad::Map, bounds: (i16, i16), boarder: i16) -> ((i16, i16), (i16, i16)) {
    (
        map_utils::scale_xy(bbox.x, bbox.y,  map.map_bounds(), bounds, boarder),
        map_utils::scale_xy(bbox.w, bbox.h,  map.map_bounds(), bounds, boarder)
    )
}

fn  draw_map_vertexes<'m, 'c, M: Manager + FlagsData + ColoursStore>(canvas: &'c mut Canvas<Window>,  context: &'c Context, manager: &'m M ) {
    if manager.meta().get("don't_draw_vertexes").is_some_and(|v| *v ) { return };
    
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
    manager.mut_meta().insert("don't_draw_vertexes".to_string(), true);
}

fn draw_map_lines<'m, 'c, M: Manager + FlagsData + ColoursStore>(canvas: &'c mut Canvas<Window>,  context: &'c Context, manager: &'m M ) {
    if manager.meta().get("don't_draw_lines").is_some_and(|v| *v ) { return };

    let map = &context.current_map;
    let player = &context.player;
    //let bsp = &context.bsp;

    let max_bounds = (manager.screen_width(), manager.screen_height());
    let boarder: i16 = 30;

    let points = map_utils::scale_map_points(
        map.map_points(),
        map.map_bounds(),
        max_bounds,
        boarder
    );


    let nodes: Vec<wad::Node> = map.traverse_bsp((player.x, player.y)).collect();

    let segs_by_sub_sector = map.segs_from_nodes(&nodes, (player.x, player.y));

    for segs_to_draw in segs_by_sub_sector {
        for segment in segs_to_draw.segments {
            draw2d_utils::draw_seg(&canvas, &segment, segs_to_draw.sub_sector_id, &points, &mut manager.mut_colours());
        }
    }
    manager.mut_meta().insert("don't_draw_lines".to_string(), true);
}

mod draw2d_utils {
    use super::*;
    use rand::{self, Rng};

    pub fn draw_seg(canvas: &Canvas<Window>, seg: &wad::Segment, subsector_id: u16, vertexes: &wad::Points, colours: &mut MutColours) {
        let p1 = vertexes[seg.start_vertext_id as usize];
        let p2 = vertexes[seg.end_verext_id as usize];
        canvas.thick_line(p1.0, p1.1, p2.0, p2.1,3, rand_colour(colours, subsector_id)).unwrap();
    }

    pub fn rand_colour(colours: &mut MutColours, subsector_id: u16) -> sdl2::pixels::Color {
        *colours.entry(subsector_id).or_insert_with(|| { 
            let seed: Vec<u8> = format!("{:032}", subsector_id).chars().into_iter().map(|c| c as u8).collect();
            let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_seed(seed[0..32].try_into().unwrap());
            let mut rand_color = || rng.gen_range::<u8, core::ops::RangeInclusive<u8>>(100..=255);
            Color { r: rand_color(), g: rand_color(), b: rand_color(), a:255 }
        })
    }
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