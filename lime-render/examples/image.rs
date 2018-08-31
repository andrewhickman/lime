extern crate env_logger;
extern crate image;
extern crate lime_render as render;
extern crate shrev;
extern crate specs;
extern crate winit;

use std::fs::File;

use image::png::PNGEncoder;
use image::ColorType;
use render::{d2, Color, ImageTarget};
use shrev::EventChannel;
use specs::prelude::*;

struct D3;

impl<'a> System<'a> for D3 {
    type SystemData = ();

    fn run(&mut self, (): Self::SystemData) {}
}

struct D2;

impl<'a> System<'a> for D2 {
    type SystemData = WriteExpect<'a, d2::Renderer>;

    fn run(&mut self, mut renderer: Self::SystemData) {
        static VERTICES: [d2::Point; 3] = [
            d2::Point(100.0, 100.0),
            d2::Point(200.0, 100.0),
            d2::Point(100.0, 200.0),
        ];
        renderer.draw_tris(&VERTICES, Color::RED)
    }
}

fn main() {
    env_logger::init();

    let mut world = World::new();
    let mut dispatcher = DispatcherBuilder::new()
        .with(D3, "D3", &[])
        .with(D2, "D2", &[]);
    world.add_resource(EventChannel::<winit::Event>::new());
    render::init::<ImageTarget>(&mut world, &mut dispatcher, [500, 500], "D3", "D2");
    let mut dispatcher = dispatcher.build();

    dispatcher.run_now(&mut world.res);
    world.maintain();

    world
        .write_resource::<ImageTarget>()
        .read(|data, [width, height]| {
            PNGEncoder::new(File::create("triangle.png").unwrap())
                .encode(data, width, height, ColorType::RGBA(8))
                .map_err(From::from)
        })
        .unwrap();
}
