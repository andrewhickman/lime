#[macro_use]
extern crate approx;
extern crate cassowary;
extern crate lime_ui as ui;
extern crate lime_utils as utils;
extern crate lime_render as render;
extern crate specs;
extern crate shrev;

mod common;

use std::iter;

use cassowary::strength::*;
use render::ScreenDimensions;
use render::d2::Point;
use shrev::EventChannel;
use specs::prelude::*;
use ui::{Constraints, Node, Position, Root};
use ui::layout::Grid;
use ui::layout::grid::Size;

use common::init_layout;

fn create_grid(
    world: &mut World,
    cols: impl IntoIterator<Item = Size>,
    rows: impl IntoIterator<Item = Size>,
) -> Entity {
    let root = world.read_resource::<Root>().entity();
    let poss = world.read_storage();
    let (grid, cons) = Grid::new(poss.get(root).unwrap(), cols, rows);
    world.write_storage().insert(root, grid).unwrap();
    world.write_storage().insert(root, cons).unwrap();
    root
}

fn create_rect(
    world: &mut World,
    parent: Entity,
    col: u32,
    row: u32,
    min_sz: (f64, f64),
) -> Entity {
    let pos = Position::new();
    let mut cons = Constraints::new(pos.min_size(min_sz, STRONG).collect());
    world.read_storage::<Grid>().get(parent).unwrap().insert(
        col,
        row,
        &pos,
        &mut cons,
    );

    Node::with_parent(world.create_entity(), parent)
        .with(pos)
        .with(cons)
        .build()
}

fn assert_approx_eq(l: Point, r: Point) {
    println!("{:?} exp {:?}", l, r);
    assert_ulps_eq!(l.0, r.0);
    assert_ulps_eq!(l.1, r.1);
}

#[test]
fn basic() {
    let (mut world, mut dispatcher) = init_layout([1000, 750].into());

    let grid = create_grid(
        &mut world,
        iter::repeat(Size::Auto).take(2),
        iter::repeat(Size::Auto).take(3),
    );

    let r1 = create_rect(&mut world, grid, 0, 0, (100.0, 100.0));
    let r2 = create_rect(&mut world, grid, 1, 1, (100.0, 100.0));
    let r3 = create_rect(&mut world, grid, 0, 2, (100.0, 100.0));

    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let p1 = comps.get(r1).unwrap();
        assert_approx_eq(p1.tl(), Point(0.0, 0.0));
        assert_approx_eq(p1.br(), Point(100.0, 100.0));
        let p2 = comps.get(r2).unwrap();
        assert_approx_eq(p2.tl(), Point(100.0, 100.0));
        assert_approx_eq(p2.br(), Point(200.0, 200.0));
        let p3 = comps.get(r3).unwrap();
        assert_approx_eq(p3.tl(), Point(0.0, 200.0));
        assert_approx_eq(p3.br(), Point(100.0, 300.0));
    }

    world
        .write_resource::<EventChannel<ScreenDimensions>>()
        .single_write([1200, 900].into());
    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let p1 = comps.get(r1).unwrap();
        assert_approx_eq(p1.tl(), Point(0.0, 0.0));
        assert_approx_eq(p1.br(), Point(100.0, 100.0));
        let p2 = comps.get(r2).unwrap();
        assert_approx_eq(p2.tl(), Point(100.0, 100.0));
        assert_approx_eq(p2.br(), Point(200.0, 200.0));
        let p3 = comps.get(r3).unwrap();
        assert_approx_eq(p3.tl(), Point(0.0, 200.0));
        assert_approx_eq(p3.br(), Point(100.0, 300.0));
    }
}

#[test]
fn auto() {
    let (mut world, mut dispatcher) = init_layout([1000, 750].into());

    let grid = create_grid(
        &mut world,
        iter::repeat(Size::Auto).take(2),
        iter::repeat(Size::Auto).take(3),
    );

    let r1 = create_rect(&mut world, grid, 0, 0, (300.0, 300.0));
    let r2 = create_rect(&mut world, grid, 1, 1, (600.0, 300.0));
    let r3 = create_rect(&mut world, grid, 0, 2, (400.0, 500.0));

    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let p1 = comps.get(r1).unwrap();
        assert_approx_eq(p1.tl(), Point(0.0, 0.0));
        assert_approx_eq(p1.br(), Point(400.0, 300.0));
        let p2 = comps.get(r2).unwrap();
        assert_approx_eq(p2.tl(), Point(400.0, 300.0));
        assert_approx_eq(p2.br(), Point(1000.0, 600.0));
        let p3 = comps.get(r3).unwrap();
        assert_approx_eq(p3.tl(), Point(0.0, 600.0));
        assert_approx_eq(p3.br(), Point(400.0, 750.0));
    }

    world
        .write_resource::<EventChannel<ScreenDimensions>>()
        .single_write([1200, 1100].into());
    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let p1 = comps.get(r1).unwrap();
        assert_approx_eq(p1.tl(), Point(0.0, 0.0));
        assert_approx_eq(p1.br(), Point(400.0, 300.0));
        let p2 = comps.get(r2).unwrap();
        assert_approx_eq(p2.tl(), Point(400.0, 300.0));
        assert_approx_eq(p2.br(), Point(1000.0, 600.0));
        let p3 = comps.get(r3).unwrap();
        assert_approx_eq(p3.tl(), Point(0.0, 600.0));
        assert_approx_eq(p3.br(), Point(400.0, 1100.0));
    }
}

#[test]
fn abs() {
    let (mut world, mut dispatcher) = init_layout([1000, 750].into());

    let grid = create_grid(
        &mut world,
        vec![Size::Abs(500.0), Size::Abs(500.0)],
        vec![Size::Abs(500.0), Size::Abs(500.0)],
    );

    let r1 = create_rect(&mut world, grid, 0, 0, (0.0, 0.0));
    let r2 = create_rect(&mut world, grid, 1, 1, (0.0, 0.0));

    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let p1 = comps.get(r1).unwrap();
        assert_approx_eq(p1.tl(), Point(0.0, 0.0));
        assert_approx_eq(p1.br(), Point(500.0, 500.0));
        let p2 = comps.get(r2).unwrap();
        assert_approx_eq(p2.tl(), Point(500.0, 500.0));
        assert_approx_eq(p2.br(), Point(1000.0, 750.0));
    }

    world
        .write_resource::<EventChannel<ScreenDimensions>>()
        .single_write([1200, 1100].into());
    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let p1 = comps.get(r1).unwrap();
        assert_approx_eq(p1.tl(), Point(0.0, 0.0));
        assert_approx_eq(p1.br(), Point(500.0, 500.0));
        let p2 = comps.get(r2).unwrap();
        assert_approx_eq(p2.tl(), Point(500.0, 500.0));
        assert_approx_eq(p2.br(), Point(1000.0, 1000.0));
    }
}

#[test]
fn rel() {
    let (mut world, mut dispatcher) = init_layout([1000, 750].into());

    let grid = create_grid(
        &mut world,
        vec![Size::Rel(1.0), Size::Rel(1.0), Size::Rel(2.0)],
        vec![Size::Rel(1.0), Size::Rel(2.0)],
    );

    let r1 = create_rect(&mut world, grid, 0, 0, (0.0, 0.0));
    let r2 = create_rect(&mut world, grid, 1, 1, (0.0, 0.0));
    let r3 = create_rect(&mut world, grid, 2, 0, (0.0, 0.0));

    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let p1 = comps.get(r1).unwrap();
        assert_approx_eq(p1.tl(), Point(0.0, 0.0));
        assert_approx_eq(p1.br(), Point(250.0, 250.0));
        let p2 = comps.get(r2).unwrap();
        assert_approx_eq(p2.tl(), Point(250.0, 250.0));
        assert_approx_eq(p2.br(), Point(500.0, 750.0));
        let p3 = comps.get(r3).unwrap();
        assert_approx_eq(p3.tl(), Point(500.0, 0.0));
        assert_approx_eq(p3.br(), Point(1000.0, 250.0));
    }

    world
        .write_resource::<EventChannel<ScreenDimensions>>()
        .single_write([1200, 1200].into());
    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let p1 = comps.get(r1).unwrap();
        assert_approx_eq(p1.tl(), Point(0.0, 0.0));
        assert_approx_eq(p1.br(), Point(300.0, 400.0));
        let p2 = comps.get(r2).unwrap();
        assert_approx_eq(p2.tl(), Point(300.0, 400.0));
        assert_approx_eq(p2.br(), Point(600.0, 1200.0));
        let p3 = comps.get(r3).unwrap();
        assert_approx_eq(p3.tl(), Point(600.0, 0.0));
        assert_approx_eq(p3.br(), Point(1200.0, 400.0));
    }
}