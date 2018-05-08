use std::collections::HashMap;
use std::ops::DerefMut;

use cassowary::{strength::*, Constraint, Solver, Variable, WeightedRelation::*};
use render::ScreenDimensions;
use shrev::EventChannel;
use specs::prelude::*;
use specs::world::Index;

use elem::ElementComponent;

pub trait Layout {
    fn layout_vars(&self) -> &LayoutVars;
    fn constraints(&self,
        elems: &WriteStorage<ElementComponent>,
        screen: &LayoutVars,
    ) -> Vec<Constraint>;
    fn resize(&mut self, vars: &Resize);
}

impl<T: ?Sized> Layout for T
where
    T: DerefMut,
    T::Target: Layout,
{
    fn layout_vars(&self) -> &LayoutVars {
        self.deref().layout_vars()
    }

    fn constraints(
        &self,
        elems: &WriteStorage<ElementComponent>,
        screen: &LayoutVars,
    ) -> Vec<Constraint> {
        self.deref().constraints(elems, screen)
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

pub struct LayoutSystem {
    solver: Solver,
    changes: HashMap<Variable, f64>,
    constraints: HashMap<Index, Vec<Constraint>>,
    srx: ReaderId<ScreenDimensions>,
    irx: ReaderId<InsertedFlag>,
    rrx: ReaderId<RemovedFlag>,
    screen_vars: LayoutVars,
}

impl LayoutSystem {
    fn add_constraints(&mut self, idx: Index, constraints: Vec<Constraint>) {
        self.solver.add_constraints(&constraints).unwrap();
        if let Some(old) = self.constraints.insert(idx, constraints) {
            for cn in old {
                self.solver.remove_constraint(&cn).unwrap();
            }
        }
    }

    fn remove_constraints(&mut self, idx: Index) {
        if let Some(old) = self.constraints.remove(&idx) {
            for cn in old {
                self.solver.remove_constraint(&cn).unwrap();
            }
        }
    }
}

impl<'a> System<'a> for LayoutSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, ElementComponent>,
        Read<'a, EventChannel<ScreenDimensions>>,
    );

    fn run(&mut self, (ents, mut elems, stx): Self::SystemData) {
        let mut update = false;
        for ev in elems.removed().read(&mut self.rrx) {
            self.remove_constraints(*ev.as_ref());
            update = true;
        }

        for ev in elems.inserted().read(&mut self.irx) {
            if let Some(elem) = elems.get(ents.entity(*ev.as_ref())) {
                let constraints = elem.constraints(&elems, &self.screen_vars);
                self.add_constraints(*ev.as_ref(), constraints);
            }
            update = true;
        }

        if let Some(sz) = stx.read(&mut self.srx).last() {
            self.solver
                .suggest_value(self.screen_vars.right, f64::from(sz.width()))
                .unwrap();
            self.solver
                .suggest_value(self.screen_vars.bottom, f64::from(sz.height()))
                .unwrap();
            update = true;
        }

        if update {
            self.changes
                .extend(self.solver.fetch_changes().iter().cloned());
            for elem in (&mut elems).join() {
                let resize = {
                    let var = elem.layout_vars();
                    Resize {
                        left: self.changes.get(&var.left).cloned(),
                        right: self.changes.get(&var.right).cloned(),
                        top: self.changes.get(&var.top).cloned(),
                        bottom: self.changes.get(&var.bottom).cloned(),
                    }
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
            WriteStorage<'a, ElementComponent>,
        );

        let (dim, mut stx, mut elems) = Data::fetch(res);

        let mut solver = Solver::new();
        let changes = HashMap::new();
        let constraints = HashMap::new();
        let width = Variable::new();
        let height = Variable::new();
        let zero = Variable::new();
        let srx = stx.register_reader();
        let irx = elems.track_inserted();
        let rrx = elems.track_removed();

        solver.add_edit_variable(width, REQUIRED - 1.0).unwrap();
        solver.suggest_value(width, f64::from(dim.width())).unwrap();
        solver.add_edit_variable(height, REQUIRED - 1.0).unwrap();
        solver.suggest_value(height, f64::from(dim.height())).unwrap();
        solver.add_constraint(zero | EQ(REQUIRED) | 0.0).unwrap();

        let screen_vars = LayoutVars {
            top: zero,
            bottom: height,
            left: zero,
            right: width,
        };

/*
        for &ent in &root.stack {
            add_constraints(&elems, &vars, ent, &screen_vars, &mut solver);
        }
        */

        LayoutSystem {
            solver,
            screen_vars,
            changes,
            constraints,
            srx,
            irx, 
            rrx,
        }
    }
}