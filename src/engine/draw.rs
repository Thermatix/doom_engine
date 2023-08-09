use super::*;


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
    fn screen_width(&self) -> i32;
    fn screen_height(&self) -> i32;
    fn draw_layers(&self, canvas: &mut Canvas<Window>, context: &Context);
}

pub struct Draw2D  {
    screen_width: i32,
    screen_height: i32,
    layers: Layers<Self>,
}

impl Manager for Draw2D {
    fn screen_width(&self) -> i32 {
        self.screen_width
    }
    fn screen_height(&self) -> i32 {
        self.screen_height
    }

    fn draw_layers(&self, canvas: &mut Canvas<Window>, context: &Context) {
        for layer in self.layers.iter() {
            layer.draw(canvas, context, self);
        };
        
    }
}



impl Draw2D {
    pub fn new( screen_width: i32, screen_height: i32) -> Self {
        let layers = Layers::new();
        let mut draw_2d = Self {
            screen_width,
            screen_height,
            layers,
        };

        draw_2d.layers.push(
            Layer {
                 name: "map".to_string(),
                 draw_function: Box::new(draw_map),
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

    let scaled_pos = MapUtils::scale_Manager(
        &map.map_bounds(),
        player.x as i32,
        player.y as i32,
        30,
        manager.screen_width(),
         manager.screen_height(),
    );
    canvas.set_draw_color(Color::GREEN);
    canvas.draw_point(scaled_pos);
}

fn  draw_map<M: Manager>(canvas: &mut Canvas<Window>,  context: &Context, manager: &M ) {

    let map = context.current_map.as_ref().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let points = MapUtils::scale_map_points(
        map.map_points(),
        map.map_bounds(),
        (manager.screen_width(), manager.screen_height()),
        30
    );
    let mut draw_map = true;


    if draw_map {
        for (p1, p2) in map.line_defs_to_vertexes(Some(&points)).unwrap()  {
            canvas.set_draw_color(Color::GREY);
            canvas.draw_line(*p1, *p2).unwrap();
            canvas.set_draw_color(Color::YELLOW);
            canvas.draw_point(*p1).unwrap();
            canvas.draw_point(*p2).unwrap();
        }
        draw_map = false;
    }        
}

mod MapUtils {
    use super::*;


    pub fn scale_map_points(map_points: &wad::Points, map_bounds: &wad::P1P2, screen_bounds: wad::Point, boarder: i32) -> wad::Points {
        let ((x_min, x_max),(y_min, y_max)) = map_bounds;
        let (screen_width, screen_height) = screen_bounds;
        map_points.iter().map(|(x, y)| {
            scale_Manager(&map_bounds, *x, *y, boarder, screen_width, screen_height)
        }).collect()
    } 

    #[inline]
    pub fn scale_Manager(map_bounds: &wad::P1P2, x: i32, y: i32, boarder: i32, max_width: i32, max_height: i32) -> wad::Point {
        let ((x_min, x_max),(y_min, y_max)) = map_bounds;
        (
            scale_x(*x_min, *x_max, x, boarder, max_width - boarder),
            scale_y(*y_min, *y_max, y, boarder, max_height - boarder, max_height)
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