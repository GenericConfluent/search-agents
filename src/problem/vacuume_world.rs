use crate::problem::*;
use std::collections::HashSet;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Position {
    Left,
    Right,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Cleanliness {
    Dirty,
    Clean,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct VacuumeState {
    vacuume_position: Position,
    dirt_distribution: [Cleanliness; 2],
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum VacuumeAction {
    Suck,
    Position { position: Position },
}

fn vacuume_transition(state: &VacuumeState, action: &VacuumeAction) -> VacuumeState {
    let mut next = *state;
    match action {
        VacuumeAction::Suck => {
            let idx = if next.vacuume_position == Position::Left {
                0
            } else {
                1
            };

            next.dirt_distribution[idx] = Cleanliness::Clean;
        }
        VacuumeAction::Position { position } => {
            next.vacuume_position = *position;
        }
    }
    next
}

fn vacuume_goal_test(state: &VacuumeState) -> bool {
    state
        .dirt_distribution
        .iter()
        .all(|val| *val == Cleanliness::Clean)
}

fn vacuume_actions(_state: &VacuumeState) -> HashSet<VacuumeAction> {
    let mut actions = HashSet::new();
    actions.insert(VacuumeAction::Suck);
    actions.insert(VacuumeAction::Position {
        position: Position::Left,
    });
    actions.insert(VacuumeAction::Position {
        position: Position::Right,
    });
    actions
}

type VacuumeWorld = Problem<
    StateActionsFn<VacuumeState, VacuumeAction>,
    TransitionFn<VacuumeState, VacuumeAction>,
    GoalTestFn<VacuumeState>,
    UniformPathCost<VacuumeState, VacuumeAction, isize>,
>;

pub fn vacuume_world() -> VacuumeWorld {
    VacuumeWorld {
        initial_state: VacuumeState {
            vacuume_position: Position::Left,
            dirt_distribution: [Cleanliness::Dirty, Cleanliness::Dirty],
        },
        actions: vacuume_actions,
        transition_model: vacuume_transition,
        goal: vacuume_goal_test,
        path_cost: UniformPathCost::default(),
    }
}
