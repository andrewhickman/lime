use specs::shred::Resources;
use render::{Color, d2, d3};

pub struct D3;

impl d3::Draw for D3 {
    fn draw(&self, _: &Resources, _: &mut FnMut(&d3::Mesh, Color)) {}
}

pub struct D2;

impl d2::Draw for D2 {
    fn draw(&self, _: &Resources, visitor: &mut FnMut(&[d2::Point], Color)) {
        static VERTICES: [d2::Point; 3] = [
            d2::Point(-0.5, -0.5),
            d2::Point(0.5, -0.5),
            d2::Point(0.0, 0.5),
        ];
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        visitor(&VERTICES, red)
    }
}
