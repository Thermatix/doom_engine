use super::*;

pub trait Drawable {
    type Manager;
    fn draw<'c, 'm>(&'m self, canvas: &'c mut Canvas<Window>, context: &'c Context, manager: &'m Self::Manager); 
}

/// The basic functionality a Layer manager needs
pub trait Manager {
    fn screen_width(&self) -> i16;
    fn screen_height(&self) -> i16;
    fn draw_layers(&self, canvas: &mut Canvas<Window>, context: &Context);

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