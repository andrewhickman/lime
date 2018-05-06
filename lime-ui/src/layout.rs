use std::collections::HashMap;
use std::ops::DerefMut;

use cassowary::{strength::*, Solver, Variable, WeightedRelation::*};
use render::ScreenDimensions;
use shrev::EventChannel;
use specs::prelude::*;

use elem::{self, ElementComponent};

pub trait Layout {
    fn add_constraints(&self, vars: &ReadStorage<LayoutVars>, this: &LayoutVars, parent: &LayoutVars, solver: &mut Solver);
    fn resize(&mut self, vars: &Resize);
}

impl<T: ?Sized> Layout for T
where
    T: DerefMut,
    T::Target: Layout,
{
    fn add_constraints(&self, vars: &ReadStorage<LayoutVars>, this: &LayoutVars, parent: &LayoutVars, solver: &mut Solver) {
        self.deref().add_constraints(vars, this, parent, solver)
    }

    fn resize(&mut self, pos: &Resize) {
        self.deref_mut().resize(pos)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Resize {
    pub left: Option<f64>,
    pub right: Option<f64>,
    pub top: Option<f64>,
    pub bottom: Option<f64>,
}

impl Resize {
    fn is_some(&self) -> bool {
        self.left.is_some() || self.right.is_some() || self.top.is_some() || self.bottom.is_some()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LayoutVars {
    pub left: Variable,
    pub right: Variable,
    pub top: Variable,
    pub bottom: Variable,
}

impl LayoutVars {
    pub fn new() -> Self {
        LayoutVars {
            left: Variable::new(),
            right: Variable::new(),
            top: Variable::new(),
            bottom: Variable::new(),
        }
    }
}

impl Component for LayoutVars {
    type Storage = DenseVecStorage<Self>;
}

pub struct LayoutSystem {
    solver: Solver,
    changes: HashMap<Variable, f64>,
    rx: ReaderId<ScreenDimensions>,
    width: Variable,
    height: Variable,
    zero: Variable,
}

impl<'a> System<'a> for LayoutSystem {
    type SystemData = (
        Read<'a, EventChannel<ScreenDimensions>>,
        WriteStorage<'a, ElementComponent>,
        ReadStorage<'a, LayoutVars>,
    );

    fn run(&mut self, (tx, mut elems, vars): Self::SystemData) {
        if let Some(sz) = tx.read(&mut self.rx).last() {
            self.changes.extend(self.solver.fetch_changes().iter().cloned());
            self.solver
                .suggest_value(self.width, sz.width() as f64)
                .unwrap();
            self.changes.extend(self.solver.fetch_changes().iter().cloned());
            self.solver
                .suggest_value(self.height, sz.height() as f64)
                .unwrap();
            self.changes.extend(self.solver.fetch_changes().iter().cloned());
            println!("changes: {:?}", self.changes);
            for (elem, var) in (&mut elems, &vars).join() {
                let resize = Resize {
                    left: self.changes.get(&var.left).cloned(),
                    right: self.changes.get(&var.right).cloned(),
                    top: Some(self.solver.get_value(var.top)),//self.changes.get(&var.top).cloned(),
                    bottom: self.changes.get(&var.bottom).cloned(),
                };

                if resize.is_some() {
                    elem.resize(&resize);
                }
            }
            self.changes.clear();
        }
    }
}

impl LayoutSystem {
    pub fn new(res: &Resources) -> Self {
        type Data<'a> = (
            ReadExpect<'a, ScreenDimensions>,
            Write<'a, EventChannel<ScreenDimensions>>,
            ReadExpect<'a, elem::Root>,
            ReadStorage<'a, ElementComponent>,
            ReadStorage<'a, LayoutVars>,
        );

        let (dim, mut tx, root, elems, vars) = Data::fetch(res);

        let mut solver = Solver::new();
        let width = Variable::new();
        let height = Variable::new();
        let zero = Variable::new();
        let rx = tx.register_reader();

        solver.add_edit_variable(width, REQUIRED - 1.0).unwrap();
        solver.suggest_value(width, dim.width() as f64).unwrap();
        solver.add_edit_variable(height, REQUIRED - 1.0).unwrap();
        solver.suggest_value(height, dim.height() as f64).unwrap();
        solver.add_constraint(zero | EQ(REQUIRED) | 0.0).unwrap();

        let screen_vars = LayoutVars {
            top: zero,
            bottom: height,
            left: zero,
            right: width,
        };

        for &ent in &root.stack {
            add_constraints(&elems, &vars, ent, &screen_vars, &mut solver);
        }

        let changes = solver.fetch_changes().iter().cloned().collect();
        println!("changes: {:?}", changes);

        LayoutSystem {
            solver,
            width,
            height,
            zero,
            changes,
            rx,
        }
    }
}

fn add_constraints(
    elems: &ReadStorage<ElementComponent>,
    vars: &ReadStorage<LayoutVars>,
    ent: Entity,
    parent_vars: &LayoutVars,
    solver: &mut Solver,
) {
    let elem = elems.get(ent).unwrap();
    let cur_vars = vars.get(ent).unwrap();
    elem.add_constraints(vars, cur_vars, parent_vars, solver);
    for &child in elem.children().iter() {
        add_constraints(elems, vars, child, cur_vars, solver)
    }
}