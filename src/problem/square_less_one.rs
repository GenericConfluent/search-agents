use std::collections::HashSet;

use rand::prelude::*;
use rand::seq::SliceRandom;

use crate::problem::*;

/// Inner is the grid. Value 0 represents the space. Postive values are
/// specific tiles. Invariant: All values in inner are unique.
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct GridState {
    inner: Vec<usize>,
    side_len: usize,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum GridAction {
    Up,
    Down,
    Right,
    Left,
}

impl GridState {
    fn new(side_len: usize) -> Self {
        let inner = (0..side_len.pow(2)).collect::<Vec<_>>();
        Self { inner, side_len }
    }

    fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.inner.shuffle(&mut rng);
    }

    fn is_solved(&self) -> bool {
        self.inner.iter().skip(1).is_sorted()
            || self.inner.iter().take(self.side_len.pow(2) - 1).is_sorted()
    }

    fn actions(&self) -> HashSet<GridAction> {
        let mut set = HashSet::with_capacity(4);
        set.insert(GridAction::Up);
        set.insert(GridAction::Down);
        set.insert(GridAction::Right);
        set.insert(GridAction::Left);
        set
    }

    fn transition(&self, action: &GridAction) -> Self {
        let space_pos = self
            .inner
            .iter()
            .position(|x| *x == 0)
            .expect("Every grid should contain an empty cell") as isize;
        let side_len = self.side_len as isize;
        let next_pos = space_pos
            + match action {
                GridAction::Up => -side_len,
                GridAction::Down => side_len,
                GridAction::Left => -1,
                GridAction::Right => 1,
            };

        // Disallow moving off the board vertically
        let vertical_exit = next_pos < 0 || side_len.pow(2) <= next_pos;

        // Disallow wrapping between rows.
        let horizontal_wrap = (*action == GridAction::Left && next_pos % side_len == side_len - 1)
            || (*action == GridAction::Right && next_pos % side_len == 0);

        let mut next = self.clone();
        // Only change the board if rules not broken
        if !(vertical_exit || horizontal_wrap) {
            next.inner.swap(space_pos as usize, next_pos as usize);
        }
        next
    }
}

type SquareLessOne = Problem<
    StateActionsFn<GridState, GridAction>,
    TransitionFn<GridState, GridAction>,
    GoalTestFn<GridState>,
>;

pub fn square_less_one(size: usize) -> SquareLessOne {
    let mut initial_state = GridState::new(size);
    initial_state.shuffle();

    SquareLessOne {
        initial_state,
        actions: GridState::actions,
        transition_model: GridState::transition,
        goal: GridState::is_solved,
    }
}
