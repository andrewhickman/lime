use cassowary::strength::REQUIRED;
use cassowary::WeightedRelation::*;
use cassowary::{Constraint, Solver, Variable};
use fnv::FnvHashMap;
use shrev::{EventChannel, ReaderId};
use specs::prelude::*;
use specs_mirror::{StorageExt, StorageMutExt};
use utils::throw;
use winit::{self, WindowEvent::Resized};

use layout::cons::{ConstraintUpdate, ConstraintsStorage};
use layout::{Constraints, Position};
use tree::Root;
use {State, StateEvent};

pub struct LayoutSystem {
    solver: Solver,
    changes: FnvHashMap<Variable, f64>,
    events_rx: ReaderId<winit::Event>,
    state_rx: ReaderId<StateEvent>,
    width: Variable,
    height: Variable,
}

impl LayoutSystem {
    pub const NAME: &'static str = "ui::Layout";

    pub(crate) fn add(world: &mut World, dispatcher: &mut DispatcherBuilder<'_, '_>) {
        let root = world.read_resource::<Root>();
        let mut poss = world.write_storage::<Position>();
        let mut events_tx = world.write_resource::<EventChannel<winit::Event>>();
        let state_rx = world.write_storage::<State>().register_reader();

        let mut solver = Solver::new();

        let pos = poss
            .entry(root.entity())
            .unwrap_or_else(throw)
            .or_insert_with(Default::default)
            .clone();

        solver
            .add_constraint(pos.left_var() | EQ(REQUIRED) | 0.0)
            .unwrap();
        solver
            .add_constraint(pos.top_var() | EQ(REQUIRED) | 0.0)
            .unwrap();
        let width = pos.right_var();
        solver.add_edit_variable(width, REQUIRED - 1.0).unwrap();
        let height = pos.bottom_var();
        solver.add_edit_variable(height, REQUIRED - 1.0).unwrap();

        let sys = LayoutSystem {
            solver,
            changes: FnvHashMap::default(),
            events_rx: events_tx.register_reader(),
            state_rx,
            width,
            height,
        };

        dispatcher.add_thread_local(sys);
    }

    fn handle_resize(&mut self, events_tx: &EventChannel<winit::Event>) {
        let resize = events_tx
            .read(&mut self.events_rx)
            .filter_map(|event| match event {
                winit::Event::WindowEvent {
                    event: Resized(size),
                    ..
                } => Some(size),
                _ => None,
            })
            .last();

        if let Some(size) = resize {
            trace!("Resizing to ({}, {}).", size.width, size.height);
            let width = self.width;
            self.resize(width, size.width);
            let height = self.height;
            self.resize(height, size.height);
        }
    }

    fn resize(&mut self, var: Variable, val: f64) {
        use cassowary::SuggestValueError::*;

        match self.solver.suggest_value(var, val) {
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
        ReadExpect<'a, EventChannel<winit::Event>>,
        WriteStorage<'a, Constraints>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, State>,
    );

    fn run(&mut self, (events_tx, mut cons, mut poss, states): Self::SystemData) {
        self.handle_resize(&events_tx);

        for state_ev in states.read_events(&mut self.state_rx) {
            if let Some(needs_layout) = state_ev.needs_layout_changed() {
                if let Some(con) = cons.get_mut(state_ev.entity) {
                    if needs_layout {
                        con.expand();
                    } else {
                        con.collapse();
                    }
                }
            }
        }

        ConstraintsStorage::handle_updates(&mut cons, |update| match update {
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
