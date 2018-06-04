use cassowary::strength::REQUIRED;
use cassowary::WeightedRelation::*;
use cassowary::{Constraint, Solver, Variable};
use fnv::FnvHashMap;
use render::ScreenDimensions;
use shrev::{EventChannel, ReaderId};
use specs::prelude::*;
use utils::throw;

use draw::VisibilityEvent;
use layout::cons::{ConstraintStorage, ConstraintUpdate};
use layout::{Constraints, Position};
use tree::Root;

pub struct LayoutSystem {
    solver: Solver,
    changes: FnvHashMap<Variable, f64>,
    dims_rx: ReaderId<ScreenDimensions>,
    vis_rx: ReaderId<VisibilityEvent>,
    width: Variable,
    height: Variable,
}

impl LayoutSystem {
    pub(crate) fn new(world: &mut World) -> Self {
        let dims = world.read_resource::<ScreenDimensions>();
        let root = world.read_resource::<Root>();
        let mut poss = world.write_storage::<Position>();
        let mut dims_tx = world.write_resource::<EventChannel<ScreenDimensions>>();
        let mut vis_tx = world.write_resource::<EventChannel<VisibilityEvent>>();

        let mut solver = Solver::new();

        let zero = Variable::new();
        solver.add_constraint(zero | EQ(REQUIRED) | 0.0).unwrap();
        let width = Variable::new();
        solver.add_edit_variable(width, REQUIRED - 1.0).unwrap();
        let height = Variable::new();
        solver.add_edit_variable(height, REQUIRED - 1.0).unwrap();

        poss.insert(root.entity(), Position::root(zero, width, height))
            .unwrap_or_else(throw);

        let mut sys = LayoutSystem {
            solver,
            changes: FnvHashMap::default(),
            dims_rx: dims_tx.register_reader(),
            vis_rx: vis_tx.register_reader(),
            width,
            height,
        };

        sys.resize(width, dims.width());
        sys.resize(height, dims.height());
        sys
    }

    fn resize(&mut self, var: Variable, val: u32) {
        use cassowary::SuggestValueError::*;

        match self.solver.suggest_value(var, val.into()) {
            Ok(()) => (),
            Err(UnknownEditVariable) => panic!("Unknown edit variable {:?}", var),
            Err(InternalSolverError(msg)) => panic!(msg),
        }
    }

    fn add_constraint(&mut self, con: Constraint) {
        use cassowary::AddConstraintError::*;

        match self.solver.add_constraint(con.clone()) {
            Ok(()) => (),
            Err(DuplicateConstraint) => error!("Constraint added twice: '{:#?}'.", con),
            Err(UnsatisfiableConstraint) => warn!("Unsatisfiable constraint '{:#?}'.", con),
            Err(InternalSolverError(msg)) => panic!(msg),
        }
    }

    fn remove_constraint(&mut self, con: Constraint) {
        use cassowary::RemoveConstraintError::*;

        match self.solver.remove_constraint(&con) {
            Ok(()) => (),
            Err(UnknownConstraint) => error!("Constraint removed twice: '{:#?}'.", con),
            Err(InternalSolverError(msg)) => panic!(msg),
        }
    }
}

impl<'a> System<'a> for LayoutSystem {
    type SystemData = (
        ReadExpect<'a, EventChannel<ScreenDimensions>>,
        ReadExpect<'a, EventChannel<VisibilityEvent>>,
        WriteStorage<'a, Constraints>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, (dims_tx, vis_tx, mut cons, mut poss): Self::SystemData) {
        if let Some(dims) = dims_tx.read(&mut self.dims_rx).last() {
            trace!("Resizing ui to '({}, {})'.", dims.width(), dims.height());
            let LayoutSystem { width, height, .. } = self;
            self.resize(*width, dims.width());
            self.resize(*height, dims.height());
        }

        for vis_ev in vis_tx.read(&mut self.vis_rx) {
            if let Some(needs_layout) = vis_ev.needs_layout_changed() {
                if let Some(con) = cons.get_mut(vis_ev.entity) {
                    if needs_layout {
                        con.expand();
                    } else {
                        con.collapse();
                    }
                }
            }
        }

        ConstraintStorage::handle_updates(&mut cons, |update| match update {
            ConstraintUpdate::Add(con) => self.add_constraint(con),
            ConstraintUpdate::Remove(con) => self.remove_constraint(con),
        });

        self.changes
            .extend(self.solver.fetch_changes().iter().cloned());
        if !self.changes.is_empty() {
            trace!("Applying {} layout changes.", self.changes.len());
            for pos in (&mut poss).join() {
                pos.update(&self.changes);
            }
            self.changes.clear();
        }
    }
}
