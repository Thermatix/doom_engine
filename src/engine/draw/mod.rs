use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;

use super::*;

mod helpers;
mod layers;
mod traits;

pub use traits::*;

pub type Flags<'a> = Ref<'a, HashMap<String, bool>>;
pub type MutFlags<'a> = RefMut<'a, HashMap<String, bool>>;
pub type Colours<'a> = Ref<'a, HashMap<u16, Color>>;
pub type MutColours<'a> = RefMut<'a, HashMap<u16, Color>>;
pub type Layers<Draw> = HashMap<String, Layer<Draw>>;

/// A Drawing layer
// TODO: Provide a way to allow for layers to depend on other layers (and thus can have values shared between them)
pub struct Layer<M> where M: Manager {
    pub draw_function: Box<dyn for<'m, 'c> Fn(&'c mut Canvas<Window>, &'c Context, &'m M)>,
}

impl<M> Drawable for Layer<M> where M: Manager {
    type Manager = M;
    fn draw<'c, 'm>(&'m self, canvas: &'c mut Canvas<Window>, context: &'c Context, manager: &'m Self::Manager) {
        (&self.draw_function.as_ref())(canvas, context, manager);
    }
}

/// A Drawing manager for Drawing the MAP in a top down view
pub struct Draw2D {
    screen_width: i16,
    screen_height: i16,
    layers: Layers<Self>,
    enabled_layers: Vec<String>,
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
        for layer in self.enabled_layers.iter() {
            self.layers[layer].draw(canvas, context, self);
        }
    }

    type Drawable = Layer<Draw2D>;

    fn layers(&self) -> &HashMap<String, Self::Drawable> {
        &self.layers
    }
}

impl FlagsData for Draw2D {
    fn mut_meta(&self) -> MutFlags {
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
    pub fn new(screen_width: i16, screen_height: i16) -> Self {
        let layers = Layers::new();
        let mut draw_2d = Self {
            screen_width,
            screen_height,
            layers,
            enabled_layers: Vec::new(),
            meta: RefCell::new(HashMap::new()),
            colours: RefCell::new(HashMap::new()),
        };

        draw_2d.layers.insert(
            "map-lines_bsp".to_string(),
            Layer {
                draw_function: Box::new(layers::draw_map_lines_bsp),
            },
        );

        draw_2d.layers.insert(
            "map-lines_def".to_string(),
            Layer {
                draw_function: Box::new(layers::draw_map_line_defs),
            },
        );

        draw_2d.layers.insert(
            "map-vertexes".to_string(),
            Layer {
                draw_function: Box::new(layers::draw_map_vertexes),
            },
        );

        draw_2d.layers.insert(
            "map-bsp".to_string(),
            Layer {
                draw_function: Box::new(layers::draw_map_bsp),
            },
        );

        draw_2d.layers.insert(
            "player".to_string(),
            Layer {
                draw_function: Box::new(layers::draw_player),
            },
        );

        draw_2d.enabled_layers.push("map-lines_bsp".to_string());
        draw_2d.enabled_layers.push("map-vertexes".to_string());
        draw_2d.enabled_layers.push("player".to_string());

        draw_2d
    }
}
