use super::*;
use rand::{self, Rng};

pub fn draw_seg(canvas: &Canvas<Window>, seg: &wad::Segment, subsector_id: u16, vertexes: &wad::Points, colours: &mut MutColours) {
    let p1 = vertexes[seg.start_vertext_id as usize];
    let p2 = vertexes[seg.end_verext_id as usize];
    helpers::draw_line(canvas, &p1, &p2, rand_colour(colours, subsector_id))
}

pub fn draw_line(canvas: &Canvas<Window>, p1: &wad::Point, p2: &wad::Point, colour: Color) {
    canvas.thick_line(p1.0, p1.1, p2.0, p2.1,3, colour).unwrap();
}

pub fn rand_colour(colours: &mut MutColours, subsector_id: u16) -> sdl2::pixels::Color {
    *colours.entry(subsector_id).or_insert_with(|| { 
        let seed: Vec<u8> = format!("{:032}", subsector_id).chars().into_iter().map(|c| c as u8).collect();
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_seed(seed.try_into().unwrap());
        let mut rand_color = || rng.gen_range::<u8, core::ops::RangeInclusive<u8>>(100..=255);
        Color { r: rand_color(), g: rand_color(), b: rand_color(), a:255 }
    })
}
