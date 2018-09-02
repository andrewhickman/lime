use std::fs::File;
use std::path::{Path, PathBuf};

use image::png::{PNGDecoder, PNGEncoder};
use image::{ColorType, DecodingResult, ImageDecoder};
use render::{self, ImageTarget};
use shrev::EventChannel;
use specs::prelude::*;
use {env_logger, winit};

pub fn test<P, D3, D2>(dir: P, d3: D3, d2: D2, dimensions: [u32; 2])
where
    P: AsRef<Path>,
    D3: for<'a> System<'a> + Send,
    D2: for<'a> System<'a> + Send,
{
    env_logger::try_init().ok();

    let mut world = World::new();
    let mut dispatcher = DispatcherBuilder::new()
        .with(d3, "D3", &[])
        .with(d2, "D2", &[]);
    world.add_resource(EventChannel::<winit::Event>::new());
    render::init::<ImageTarget>(&mut world, &mut dispatcher, dimensions, "D3", "D2");
    let mut dispatcher = dispatcher.build();

    dispatcher.run_now(&mut world.res);
    world.maintain();

    let out_path = test_data_path(dir.as_ref()).join("actual.png");
    let in_path = test_data_path(dir.as_ref()).join("expected.png");

    world
        .write_resource::<ImageTarget>()
        .read(|data, [width, height]| {
            write_png(out_path, data, (width, height));

            let (exp_data, exp_dims) = read_png(in_path);
            assert_eq!(data, &*exp_data);
            assert_eq!((width, height), exp_dims);
            Ok(())
        })
        .unwrap();
}

fn read_png(path: impl AsRef<Path>) -> (Vec<u8>, (u32, u32)) {
    let mut decoder = PNGDecoder::new(File::open(path).unwrap());
    let dims = decoder.dimensions().unwrap();
    match decoder.read_image().unwrap() {
        DecodingResult::U8(img) => (img, dims),
        _ => unreachable!(),
    }
}

fn write_png(path: impl AsRef<Path>, img: &[u8], (width, height): (u32, u32)) {
    PNGEncoder::new(File::create(path).unwrap())
        .encode(img, width, height, ColorType::RGBA(8))
        .unwrap()
}

fn test_data_path(dir: impl AsRef<Path>) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(dir)
}
