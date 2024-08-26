use std::collections::{HashMap, HashSet};
use std::hash::Hash;

mod vacuume_world;
pub use vacuume_world::vacuume_world;

mod square_less_one;
pub use square_less_one::square_less_one;

pub trait StateActionsMap {
    type State;
    type Action;
    fn actions(&self, current: &Self::State) -> HashSet<Self::Action>;
}
//
// Note, this can be more efficient. Ideally we don't clone the inner
// action set to convert it to a HashMap.
impl<S, I, A> StateActionsMap for HashMap<S, I>
where
    S: Eq + Hash,
    I: IntoIterator<Item = A> + Clone,
    A: Clone + Eq + Hash,
{
    type State = S;
    type Action = A;

    fn actions(&self, current: &Self::State) -> HashSet<Self::Action> {
        let mut action_set = HashSet::new();
        if let Some(actions) = self.get(current) {
            let it = actions.clone().into_iter();
            for action in it {
                action_set.insert(action.clone());
            }
        }
        action_set
    }
}

pub type StateActionsFn<S, A> = fn(&S) -> HashSet<A>;

impl<S, A> StateActionsMap for StateActionsFn<S, A> {
    type State = S;
    type Action = A;
    fn actions(&self, current: &Self::State) -> HashSet<Self::Action> {
        self(current)
    }
}

pub trait Transition {
    type State;
    type Action;
    fn transition(&self, current: &Self::State, action: &Self::Action) -> Self::State;
}

pub type TransitionFn<S, A> = fn(current: &S, action: &A) -> S;

impl<S, A> Transition for TransitionFn<S, A> {
    type State = S;
    type Action = A;

    fn transition(&self, current: &Self::State, action: &Self::Action) -> Self::State {
        self(current, action)
    }
}

pub trait GoalTest {
    type State;
    fn is_goal(&self, s: &Self::State) -> bool;
}

pub type GoalTestFn<S> = fn(s: &S) -> bool;

impl<S> GoalTest for GoalTestFn<S> {
    type State = S;
    fn is_goal(&self, s: &Self::State) -> bool {
        self(s)
    }
}

pub struct Problem<A, T, G>
where
    A: StateActionsMap,
    T: Transition<State = A::State, Action = A::Action>,
    G: GoalTest<State = A::State>,
{
    pub initial_state: A::State,
    pub actions: A,
    pub transition_model: T,
    pub goal: G,
}
