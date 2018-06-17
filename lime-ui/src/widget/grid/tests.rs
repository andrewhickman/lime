use std::iter;

use cassowary::strength::*;
use render::d2::Point;
use render::ScreenDimensions;
use serde_json as json;
use shrev::EventChannel;
use specs::prelude::*;
use specs_mirror::StorageMutExt;

use super::*;
use draw::{Visibility, VisibilityState};
use layout::{Constraints, ConstraintsBuilder, Position};
use tests::init_test;
use tree::{Node, Root};

fn create_root_grid(
    world: &mut World,
    cols: impl IntoIterator<Item = Size>,
    rows: impl IntoIterator<Item = Size>,
) -> Entity {
    let root = world.read_resource::<Root>().entity();
    let poss = world.read_storage();
    let mut cons = Constraints::default();
    let grid = Grid::new(poss.get(root).unwrap(), &mut cons, cols, rows);
    world.write_storage().insert(root, grid).unwrap();
    world.write_storage().insert(root, cons).unwrap();
    root
}

fn create_grid(
    world: &mut World,
    cols: impl IntoIterator<Item = Size>,
    rows: impl IntoIterator<Item = Size>,
) -> Entity {
    let root = world.read_resource::<Root>().entity();
    let pos = Position::new();
    let mut cons = Constraints::default();
    let grid = Grid::new(&pos, &mut cons, cols, rows);
    Node::with_parent(world.create_entity(), root)
        .with(pos)
        .with(grid)
        .with(cons)
        .build()
}

fn create_rect(
    world: &mut World,
    parent: Entity,
    col: u32,
    row: u32,
    build: impl FnOnce(ConstraintsBuilder) -> ConstraintsBuilder,
) -> Entity {
    let pos = Position::new();
    let mut cons = build(pos.constraints_builder()).build();
    world
        .read_storage::<Grid>()
        .get(parent)
        .unwrap()
        .insert(col, row, &pos, &mut cons);

    Node::with_parent(world.create_entity(), parent)
        .with(pos)
        .with(cons)
        .build()
}

fn assert_approx_eq(l: Point, r: Point) {
    assert_ulps_eq!(l.0, r.0);
    assert_ulps_eq!(l.1, r.1);
}

#[test]
fn empty() {
    let (mut world, mut dispatcher) = init_test([1000, 750].into());

    create_root_grid(&mut world, iter::empty(), iter::empty());

    dispatcher.dispatch(&world.res);
}

#[test]
fn basic() {
    let (mut world, mut dispatcher) = init_test([1000, 750].into());

    let grid = create_root_grid(
        &mut world,
        iter::repeat(Size::Auto).take(2),
        iter::repeat(Size::Auto).take(3),
    );

    let r1 = create_rect(&mut world, grid, 0, 0, |bld| {
        bld.min_size((100.0, 100.0), STRONG)
    });
    let r2 = create_rect(&mut world, grid, 1, 1, |bld| {
        bld.min_size((100.0, 100.0), STRONG)
    });
    let r3 = create_rect(&mut world, grid, 0, 2, |bld| {
        bld.min_size((100.0, 100.0), STRONG)
    });

    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let p1 = comps.get(r1).unwrap();
        assert_approx_eq(p1.top_left(), Point(0.0, 0.0));
        assert_approx_eq(p1.bottom_right(), Point(100.0, 100.0));
        let p2 = comps.get(r2).unwrap();
        assert_approx_eq(p2.top_left(), Point(100.0, 100.0));
        assert_approx_eq(p2.bottom_right(), Point(200.0, 200.0));
        let p3 = comps.get(r3).unwrap();
        assert_approx_eq(p3.top_left(), Point(0.0, 200.0));
        assert_approx_eq(p3.bottom_right(), Point(100.0, 300.0));
    }

    world
        .write_resource::<EventChannel<ScreenDimensions>>()
        .single_write([1200, 900].into());
    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let p1 = comps.get(r1).unwrap();
        assert_approx_eq(p1.top_left(), Point(0.0, 0.0));
        assert_approx_eq(p1.bottom_right(), Point(100.0, 100.0));
        let p2 = comps.get(r2).unwrap();
        assert_approx_eq(p2.top_left(), Point(100.0, 100.0));
        assert_approx_eq(p2.bottom_right(), Point(200.0, 200.0));
        let p3 = comps.get(r3).unwrap();
        assert_approx_eq(p3.top_left(), Point(0.0, 200.0));
        assert_approx_eq(p3.bottom_right(), Point(100.0, 300.0));
    }
}

#[test]
fn auto() {
    let (mut world, mut dispatcher) = init_test([1000, 750].into());

    let grid = create_root_grid(
        &mut world,
        iter::repeat(Size::Auto).take(2),
        iter::repeat(Size::Auto).take(3),
    );

    let r1 = create_rect(&mut world, grid, 0, 0, |bld| {
        bld.min_size((300.0, 300.0), STRONG)
    });
    let r2 = create_rect(&mut world, grid, 1, 1, |bld| {
        bld.min_size((600.0, 300.0), STRONG)
    });
    let r3 = create_rect(&mut world, grid, 0, 2, |bld| {
        bld.min_size((400.0, 500.0), STRONG)
    });

    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let p1 = comps.get(r1).unwrap();
        assert_approx_eq(p1.top_left(), Point(0.0, 0.0));
        assert_approx_eq(p1.bottom_right(), Point(400.0, 300.0));
        let p2 = comps.get(r2).unwrap();
        assert_approx_eq(p2.top_left(), Point(400.0, 300.0));
        assert_approx_eq(p2.bottom_right(), Point(1000.0, 600.0));
        let p3 = comps.get(r3).unwrap();
        assert_approx_eq(p3.top_left(), Point(0.0, 600.0));
        assert_approx_eq(p3.bottom_right(), Point(400.0, 750.0));
    }

    world
        .write_resource::<EventChannel<ScreenDimensions>>()
        .single_write([1200, 1100].into());
    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let p1 = comps.get(r1).unwrap();
        assert_approx_eq(p1.top_left(), Point(0.0, 0.0));
        assert_approx_eq(p1.bottom_right(), Point(400.0, 300.0));
        let p2 = comps.get(r2).unwrap();
        assert_approx_eq(p2.top_left(), Point(400.0, 300.0));
        assert_approx_eq(p2.bottom_right(), Point(1000.0, 600.0));
        let p3 = comps.get(r3).unwrap();
        assert_approx_eq(p3.top_left(), Point(0.0, 600.0));
        assert_approx_eq(p3.bottom_right(), Point(400.0, 1100.0));
    }
}

#[test]
fn abs() {
    let (mut world, mut dispatcher) = init_test([1000, 750].into());

    let grid = create_root_grid(
        &mut world,
        vec![Size::Abs(500.0), Size::Abs(500.0)],
        vec![Size::Abs(500.0), Size::Abs(500.0)],
    );

    let r1 = create_rect(&mut world, grid, 0, 0, |bld| {
        bld.min_size((0.0, 0.0), STRONG)
    });
    let r2 = create_rect(&mut world, grid, 1, 1, |bld| {
        bld.min_size((0.0, 0.0), STRONG)
    });

    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let p1 = comps.get(r1).unwrap();
        assert_approx_eq(p1.top_left(), Point(0.0, 0.0));
        assert_approx_eq(p1.bottom_right(), Point(500.0, 500.0));
        let p2 = comps.get(r2).unwrap();
        assert_approx_eq(p2.top_left(), Point(500.0, 500.0));
        assert_approx_eq(p2.bottom_right(), Point(1000.0, 750.0));
    }

    world
        .write_resource::<EventChannel<ScreenDimensions>>()
        .single_write([1200, 1100].into());
    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let p1 = comps.get(r1).unwrap();
        assert_approx_eq(p1.top_left(), Point(0.0, 0.0));
        assert_approx_eq(p1.bottom_right(), Point(500.0, 500.0));
        let p2 = comps.get(r2).unwrap();
        assert_approx_eq(p2.top_left(), Point(500.0, 500.0));
        assert_approx_eq(p2.bottom_right(), Point(1000.0, 1000.0));
    }
}

#[test]
fn rel() {
    let (mut world, mut dispatcher) = init_test([1000, 750].into());

    let grid = create_root_grid(
        &mut world,
        vec![Size::Rel(1.0), Size::Rel(1.0), Size::Rel(2.0)],
        vec![Size::Rel(1.0), Size::Rel(2.0)],
    );

    let r1 = create_rect(&mut world, grid, 0, 0, |bld| {
        bld.min_size((0.0, 0.0), STRONG)
    });
    let r2 = create_rect(&mut world, grid, 1, 1, |bld| {
        bld.min_size((0.0, 0.0), STRONG)
    });
    let r3 = create_rect(&mut world, grid, 2, 0, |bld| {
        bld.min_size((0.0, 0.0), STRONG)
    });

    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let p1 = comps.get(r1).unwrap();
        assert_approx_eq(p1.top_left(), Point(0.0, 0.0));
        assert_approx_eq(p1.bottom_right(), Point(250.0, 250.0));
        let p2 = comps.get(r2).unwrap();
        assert_approx_eq(p2.top_left(), Point(250.0, 250.0));
        assert_approx_eq(p2.bottom_right(), Point(500.0, 750.0));
        let p3 = comps.get(r3).unwrap();
        assert_approx_eq(p3.top_left(), Point(500.0, 0.0));
        assert_approx_eq(p3.bottom_right(), Point(1000.0, 250.0));
    }

    world
        .write_resource::<EventChannel<ScreenDimensions>>()
        .single_write([1200, 1200].into());
    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let p1 = comps.get(r1).unwrap();
        assert_approx_eq(p1.top_left(), Point(0.0, 0.0));
        assert_approx_eq(p1.bottom_right(), Point(300.0, 400.0));
        let p2 = comps.get(r2).unwrap();
        assert_approx_eq(p2.top_left(), Point(300.0, 400.0));
        assert_approx_eq(p2.bottom_right(), Point(600.0, 1200.0));
        let p3 = comps.get(r3).unwrap();
        assert_approx_eq(p3.top_left(), Point(600.0, 0.0));
        assert_approx_eq(p3.bottom_right(), Point(1200.0, 400.0));
    }
}

#[test]
fn mix() {
    let (mut world, mut dispatcher) = init_test([1000, 750].into());

    let grid = create_root_grid(
        &mut world,
        vec![
            Size::Abs(100.0),
            Size::Auto,
            Size::Rel(1.0),
            Size::Rel(2.0),
            Size::Abs(150.0),
        ],
        vec![Size::Abs(300.0), Size::Auto, Size::Auto, Size::Abs(250.0)],
    );

    let r1 = create_rect(&mut world, grid, 1, 0, |bld| bld.min_width(100.0, STRONG));
    let r2 = create_rect(&mut world, grid, 1, 1, |bld| {
        bld.size((150.0, 200.0), STRONG)
    });
    let r3 = create_rect(&mut world, grid, 1, 2, |bld| {
        bld.size((150.0, 200.0), STRONG)
    });
    let r4 = create_rect(&mut world, grid, 2, 0, |bld| bld);
    let r5 = create_rect(&mut world, grid, 3, 0, |bld| bld);

    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let p1 = comps.get(r1).unwrap();
        assert_approx_eq(p1.top_left(), Point(100.0, 0.0));
        assert_approx_eq(p1.bottom_right(), Point(250.0, 300.0));
        let p2 = comps.get(r2).unwrap();
        assert_approx_eq(p2.top_left(), Point(100.0, 300.0));
        assert_approx_eq(p2.bottom_right(), Point(250.0, 500.0));
        let p3 = comps.get(r3).unwrap();
        assert_approx_eq(p3.top_left(), Point(100.0, 500.0));
        assert_approx_eq(p3.bottom_right(), Point(250.0, 700.0));
        let p4 = comps.get(r4).unwrap();
        assert_approx_eq(p4.top_left(), Point(250.0, 0.0));
        assert_approx_eq(p4.bottom_right(), Point(450.0, 300.0));
        let p5 = comps.get(r5).unwrap();
        assert_approx_eq(p5.top_left(), Point(450.0, 0.0));
        assert_approx_eq(p5.bottom_right(), Point(850.0, 300.0));
    }

    world
        .write_resource::<EventChannel<ScreenDimensions>>()
        .single_write([1300, 1200].into());
    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let p1 = comps.get(r1).unwrap();
        assert_approx_eq(p1.top_left(), Point(100.0, 0.0));
        assert_approx_eq(p1.bottom_right(), Point(250.0, 300.0));
        let p2 = comps.get(r2).unwrap();
        assert_approx_eq(p2.top_left(), Point(100.0, 300.0));
        assert_approx_eq(p2.bottom_right(), Point(250.0, 500.0));
        let p3 = comps.get(r3).unwrap();
        assert_approx_eq(p3.top_left(), Point(100.0, 500.0));
        assert_approx_eq(p3.bottom_right(), Point(250.0, 700.0));
        let p4 = comps.get(r4).unwrap();
        assert_approx_eq(p4.top_left(), Point(250.0, 0.0));
        assert_approx_eq(p4.bottom_right(), Point(550.0, 300.0));
        let p5 = comps.get(r5).unwrap();
        assert_approx_eq(p5.top_left(), Point(550.0, 0.0));
        assert_approx_eq(p5.bottom_right(), Point(1150.0, 300.0));
    }
}

#[test]
fn size() {
    let (mut world, mut dispatcher) = init_test([1000, 750].into());

    let grid = create_grid(
        &mut world,
        vec![Size::Auto, Size::Rel(1.0), Size::Rel(2.0)],
        vec![Size::Auto, Size::Auto],
    );

    create_rect(&mut world, grid, 0, 0, |bld| {
        bld.size((200.0, 200.0), STRONG)
    });
    create_rect(&mut world, grid, 1, 1, |bld| {
        bld.size((200.0, 300.0), STRONG)
    });

    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let g = comps.get(grid).unwrap();
        assert_ulps_eq!(g.width(), 800.0);
        assert_ulps_eq!(g.height(), 500.0);
    }
}

fn set_visibility(world: &mut World, entity: Entity, state: VisibilityState) {
    let mut storage = world.write_storage::<Visibility>();
    let (vis, chan) = storage.modify(entity).unwrap();
    vis.set(entity, state, chan);
}

#[test]
fn visibility() {
    let (mut world, mut dispatcher) = init_test([1000, 750].into());

    let grid = create_grid(&mut world, vec![Size::Auto], vec![Size::Auto]);

    let pos = Position::new();
    let mut cons = pos.constraints_builder()
        .size((1000.0, 750.0), STRONG)
        .build();
    world
        .read_storage::<Grid>()
        .get(grid)
        .unwrap()
        .insert(0, 0, &pos, &mut cons);

    let node = Node::with_parent(world.create_entity(), grid)
        .with(pos)
        .with(cons)
        .with(Visibility::new())
        .build();

    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let g = comps.get(grid).unwrap();
        assert_ulps_eq!(g.width(), 1000.0);
        assert_ulps_eq!(g.height(), 750.0);
    }

    set_visibility(&mut world, node, VisibilityState::Collapsed);
    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let g = comps.get(grid).unwrap();
        assert_ulps_eq!(g.width(), 0.0);
        assert_ulps_eq!(g.height(), 0.0);
    }

    set_visibility(&mut world, node, VisibilityState::Visible);
    dispatcher.dispatch(&world.res);

    {
        let comps = world.read_storage::<Position>();
        let g = comps.get(grid).unwrap();
        assert_ulps_eq!(g.width(), 1000.0);
        assert_ulps_eq!(g.height(), 750.0);
    }
}

#[test]
fn de() {
    assert_eq!(
        json::from_str::<Size>(r#"{ "type": "auto" }"#).unwrap(),
        Size::Auto
    );
    assert_eq!(
        json::from_str::<Size>(r#"{ "type": "abs", "value": 100 }"#).unwrap(),
        Size::Abs(100.0)
    );
    assert_eq!(
        json::from_str::<Size>(r#"{ "type": "rel", "value": 1 }"#).unwrap(),
        Size::Rel(1.0)
    );
}
