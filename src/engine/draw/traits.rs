use super::*;

pub trait Drawable<M: Manager + ?Sized>: Sized {
    fn draw<'c, 'm>(&'m self, canvas: &'c mut Canvas<Window>, context: &'c Context, manager: &'m M);

    fn depends_on(&self) -> Option<&Vec<String>> {
        None
    }

    fn start_drawing<'c, 'm>(
        &'m self,
        canvas: &'c mut Canvas<Window>,
        context: &'c Context,
        manager: &'m M,
    ) {
        if let Some(layer_names) = self.depends_on() {
            let layers = manager.layers();
            for l_name in layer_names.iter() {
                layers[l_name].draw(canvas, &context, manager);
            }
        }
        self.draw(canvas, &context, manager);
    }
}

/// The basic functionality a Layer manager needs
pub trait Manager {
    fn screen_width(&self) -> i16;
    fn screen_height(&self) -> i16;
    fn draw_layers(&self, canvas: &mut Canvas<Window>, context: &Context);

    type Drawable: Drawable<Self>;

    fn layers(&self) -> &HashMap<String, Self::Drawable>;
}

/// A store for flags between iterations
pub trait FlagsData {
    fn mut_meta(&self) -> MutFlags;
    fn meta(&self) -> Flags;
}

/// A Store for colours so they're not repeatedly regenerated
pub trait ColoursStore {
    fn mut_colours(&self) -> MutColours;
    fn colours(&self) -> Colours;
}