extern crate lime_render as render;
extern crate winit;

use render::{Color, Renderer, d2, d3};
use winit::{Event, EventsLoop, WindowBuilder, WindowEvent};

struct D3;

impl d3::Draw for D3 {
    fn draw(&self, _: &mut FnMut(&d3::Mesh, Color)) {}
}

struct D2;

impl d2::Draw for D2 {
    fn draw(&self, visitor: &mut FnMut(&[d2::Point], Color)) {
        static VERTICES: [d2::Point; 3] = [
            d2::Point(-0.5, -0.5),
            d2::Point(0.5, -0.5),
            d2::Point(0.0, 0.5),
        ];
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        visitor(&VERTICES, red)
    }
}

#[test]
fn triangle() {
    let mut events_loop = EventsLoop::new();
    let builder = WindowBuilder::new();
    let mut renderer = Renderer::new(&events_loop, builder);

    let mut quit = false;
    let mut dim = renderer.dimensions();
    while !quit {
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::Closed,
                    ..
                } => quit = true,
                _ => (),
            };
        });

        renderer.render(&D3, &D2, &mut dim);
    }
}
