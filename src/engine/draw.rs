use super::*;
use sdl2::gfx::primitives::DrawRenderer;


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
        canvas.clear();
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
                name: "player".to_string(),
                draw_function: Box::new(draw_player),
            }
        );

        draw_2d
    }
}

pub fn draw_player<M: Manager>(canvas: &mut Canvas<Window>,  context: &Context, manager: &M ) {
    let player = context.player.as_ref().unwrap();   
    let map = context.current_map.as_ref().unwrap();

    let scaled_pos = map_utils::scale_xy(
        &map.map_bounds(),
        player.x,
        player.y,
        30,
        manager.screen_width(),
        manager.screen_height(),
    );

    canvas.filled_circle(scaled_pos.0, scaled_pos.1, 5, Color::GREEN).unwrap();
}



fn  draw_map_vertexes<M: Manager>(canvas: &mut Canvas<Window>,  context: &Context, manager: &M ) {

    let map = context.current_map.as_ref().unwrap();

    let points = map_utils::scale_map_points(
        map.map_points(),
        map.map_bounds(),
        (manager.screen_width(), manager.screen_height()),
        30
    );
 
    for (p1, p2) in map.line_defs_to_vertexes(Some(&points)).unwrap()  {
        //canvas.set_draw_color(Color::YELLOW);
        canvas.filled_circle(p1.0, p1.1 , 2, Color::YELLOW).unwrap();
        //canvas.draw_point(*p1).unwrap();
        canvas.filled_circle(p2.0, p2.1 , 2, Color::YELLOW).unwrap();
        //canvas.draw_point(*p2).unwrap();
    }     
}

fn draw_map_lines<M: Manager>(canvas: &mut Canvas<Window>,  context: &Context, manager: &M ) {

    let map = context.current_map.as_ref().unwrap();




    let points = map_utils::scale_map_points(
        map.map_points(),
        map.map_bounds(),
        (manager.screen_width(), manager.screen_height()),
        30
    );

    for (p1, p2) in map.line_defs_to_vertexes(Some(&points)).unwrap()  {
        //canvas.set_draw_color(Color::GREY);
        canvas.thick_line(p1.0, p1.1, p2.0, p2.1,3, Color::GREY).unwrap();
    }
       
}

mod map_utils {
    use super::*;


    pub fn scale_map_points(map_points: &wad::Points, map_bounds: &wad::P1P2, screen_bounds: wad::Point, boarder: i16) -> wad::Points {
        let ((x_min, x_max),(y_min, y_max)) = map_bounds;
        let (screen_width, screen_height) = screen_bounds;
        map_points.iter().map(|(x, y)| {
            scale_xy(&map_bounds, *x, *y, boarder, screen_width, screen_height)
        }).collect()
    } 

    #[inline]
    pub fn scale_xy(map_bounds: &wad::P1P2, x: i16, y: i16, boarder: i16, max_width: i16, max_height: i16) -> wad::Point {
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

    pub fn scale_y(y_min: i32, y_max: i32, n: i32, out_min: i32, out_max: i32, screen_height: i32) -> i32 {
        screen_height - (y_min.max(y_max.min(n)) - y_min) * (out_max - out_min) / (y_max - y_min) - out_min
    }
}