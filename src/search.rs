use crate::problem::*;
use std::{
    collections::{HashSet, VecDeque},
    rc::Rc,
};

struct GenerationContext<S, A> {
    parent: Rc<Node<S, A>>,
    action: A,
    path_cost: isize,
}

struct Node<S, A> {
    state: S,
    ctx: Option<GenerationContext<S, A>>,
}

impl<S, A: Clone> Node<S, A> {
    /// Returns the path from the current node back to the root.
    /// If we know this node is a goal node then this is a solution.
    /// Since we are traversing backwards the actions are returned in
    /// a backwards sequence.
    fn solution(&self) -> Vec<A> {
        let mut actions = Vec::new();
        let mut current = self;

        while let Some(ref ctx) = current.ctx {
            actions.push(ctx.action.clone());
            current = ctx.parent.as_ref();
        }

        actions
    }
}

impl<S: PartialEq, A> PartialEq for Node<S, A> {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

impl<S, A> From<S> for Node<S, A> {
    fn from(state: S) -> Self {
        Self { state, ctx: None }
    }
}

fn generate<A, T, G>(
    problem: &Problem<A, T, G>,
    node: &Rc<Node<A::State, A::Action>>,
    action: A::Action,
) -> Node<A::State, A::Action>
where
    A: StateActionsMap,
    T: Transition<State = A::State, Action = A::Action>,
    G: GoalTest<State = A::State>,
{
    let sucessor = problem.transition_model.transition(&node.state, &action);
    Node {
        state: sucessor,
        ctx: Some(GenerationContext {
            parent: node.clone(),
            action,
            path_cost: node.ctx.as_ref().map(|ctx| ctx.path_cost).unwrap_or(0),
        }),
    }
}

use std::hash::Hash;

pub fn breadth_first_search<A, T, G>(problem: Problem<A, T, G>) -> Option<Vec<A::Action>>
where
    A: StateActionsMap,
    A::State: Hash + Eq + Clone,
    A::Action: Clone,
    T: Transition<State = A::State, Action = A::Action>,
    G: GoalTest<State = A::State>,
{
    let root = Rc::new(Node::from(problem.initial_state.clone()));

    let mut frontier = VecDeque::new();
    frontier.push_back(root.clone());

    let mut explored = HashSet::new();

    while let Some(ref unexplored) = frontier.pop_front() {
        explored.insert(unexplored.state.clone());

        let actions = problem.actions.actions(&unexplored.state);
        for act in actions {
            let new_node = Rc::new(generate(&problem, &unexplored, act));
            if !(explored.contains(&new_node.state) || frontier.contains(&new_node)) {
                if problem.goal.is_goal(&new_node.state) {
                    return Some(new_node.solution());
                }
                frontier.push_back(new_node);
            }
        }
    }
    None
}
