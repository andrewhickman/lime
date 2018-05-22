mod constraints;
mod vars;

pub use self::constraints::{ConstraintUpdate, Constraints};
pub use self::vars::{PositionVars, ScreenVars};

use std::collections::HashMap;
use std::ops::DerefMut;

use cassowary::strength::REQUIRED;
use cassowary::{Solver, Variable};
use chan;
use render::ScreenDimensions;
use shrev::EventChannel;
use specs::prelude::*;
use utils::throw_msg;

use ElementComponent;

pub trait Layout {
    fn resize(&mut self, rsz: &Resize);
}

impl<T: ?Sized> Layout for T
where
    T: DerefMut,
    T::Target: Layout,
{
    fn resize(&mut self, rsz: &Resize) {
        self.deref_mut().resize(rsz)
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
    fn new(map: &HashMap<Variable, f64>, vars: &PositionVars) -> Self {
        Resize {
            left: map.get(&vars.left).cloned(),
            right: map.get(&vars.right).cloned(),
            top: map.get(&vars.top).cloned(),
            bottom: map.get(&vars.bottom).cloned(),
        }
    }

    fn is_some(&self) -> bool {
        self.left.is_some() || self.right.is_some() || self.top.is_some() || self.bottom.is_some()
    }
}

pub struct LayoutSystem {
    solver: Solver,
    changes: HashMap<Variable, f64>,
    con_rx: chan::Receiver<ConstraintUpdate>,
    dims_reader: ReaderId<ScreenDimensions>,
}

impl LayoutSystem {
    pub fn new(res: &mut Resources, con_rx: chan::Receiver<ConstraintUpdate>) -> Self {
        type Data<'a> = (
            ReadExpect<'a, ScreenDimensions>,
            ReadExpect<'a, ScreenVars>,
            WriteExpect<'a, EventChannel<ScreenDimensions>>,
        );

        let (dims, svars, mut dims_chan) = Data::fetch(res);
        let mut sys = LayoutSystem {
            solver: Solver::new(),
            changes: HashMap::new(),
            dims_reader: dims_chan.register_reader(),
            con_rx,
        };

        sys.add_edit_variable(svars.width, REQUIRED - 1.0);
        sys.resize(svars.width, dims.width());
        sys.add_edit_variable(svars.height, REQUIRED - 1.0);
        sys.resize(svars.height, dims.height());

        sys
    }

    fn add_edit_variable(&mut self, var: Variable, strength: f64) {
        use cassowary::AddEditVariableError::*;

        match self.solver.add_edit_variable(var, strength) {
            Ok(()) => (),
            Err(DuplicateEditVariable) => throw_msg("duplicate edit variable"),
            Err(BadRequiredStrength) => throw_msg("bad edit variable strength"),
        }
    }

    fn resize(&mut self, var: Variable, val: u32) {
        use cassowary::SuggestValueError::*;

        match self.solver.suggest_value(var, f64::from(val)) {
            Ok(()) => (),
            Err(UnknownEditVariable) => throw_msg(format!("Unknown edit variable {:?}", var)),
            Err(InternalSolverError(msg)) => throw_msg(msg),
        }
    }
}

impl<'a> System<'a> for LayoutSystem {
    type SystemData = (
        ReadExpect<'a, ScreenVars>,
        ReadExpect<'a, EventChannel<ScreenDimensions>>,
        ReadStorage<'a, PositionVars>,
        WriteStorage<'a, ElementComponent>,
    );

    fn run(&mut self, (svars, dims, pvars, mut elems): Self::SystemData) {
        if let Some(dims) = dims.read(&mut self.dims_reader).last() {
            self.resize(svars.width, dims.width());
            self.resize(svars.height, dims.height());
        }

        loop {
            let con_rx = &self.con_rx;
            chan_select! {
                default => break,
                con_rx.recv() -> update => match update {
                    Some(ConstraintUpdate::Add(con)) => {
                        use cassowary::AddConstraintError::*;
                        match self.solver.add_constraint(con.clone()) {
                            Ok(()) => (),
                            Err(UnsatisfiableConstraint) => warn!("Unsatisfiable layout constraint: {:#?}", con),
                            Err(DuplicateConstraint) => throw_msg("duplicate layout constraint"),
                            Err(InternalSolverError(msg)) => throw_msg(msg),
                        }
                    },
                    Some(ConstraintUpdate::Remove(con)) =>{
                        use cassowary::RemoveConstraintError::*;
                        match self.solver.remove_constraint(&con) {
                            Ok(()) | Err(UnknownConstraint) => (),
                            Err(InternalSolverError(msg)) => throw_msg(msg),
                        }
                    },
                    None => break,
                },
            }
        }

        self.changes
            .extend(self.solver.fetch_changes().iter().cloned());
        for (var, elem) in (&pvars, &mut elems).join() {
            let rsz = Resize::new(&self.changes, var);
            if rsz.is_some() {
                elem.resize(&rsz);
            }
        }
        self.changes.clear();
    }
}
