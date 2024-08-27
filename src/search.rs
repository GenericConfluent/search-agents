use crate::problem::*;
use orx_priority_queue::{NodeKeyRef, PriorityQueueDecKey};
use std::{
    collections::{HashSet, VecDeque},
    marker::PhantomData,
    rc::Rc,
};

pub struct GenerationContext<S, A> {
    parent: Rc<Node<S, A>>,
    action: A,
    path_cost: isize,
}

pub struct Node<S, A> {
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

    fn path_cost(&self) -> isize {
        self.ctx.as_ref().map(|ctx| ctx.path_cost).unwrap_or(0)
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

fn generate<A, T, G, C>(
    problem: &Problem<A, T, G, C>,
    node: &Rc<Node<A::State, A::Action>>,
    action: A::Action,
) -> Node<A::State, A::Action>
where
    A: StateActionsMap,
    T: Transition<State = A::State, Action = A::Action>,
    G: GoalTest<State = A::State>,
    C: PathCostSource<State = A::State, Action = A::Action, CostType = isize>,
{
    let sucessor = problem.transition_model.transition(&node.state, &action);
    let cost = problem.path_cost.cost(&node.state, &action);
    Node {
        state: sucessor,
        ctx: Some(GenerationContext {
            parent: node.clone(),
            action,
            path_cost: cost
                + node
                    .ctx
                    .as_ref()
                    .map(|ctx| ctx.path_cost)
                    .unwrap_or_default(),
        }),
    }
}

use std::hash::Hash;

pub fn breadth_first_search<A, T, G, C>(problem: Problem<A, T, G, C>) -> Option<Vec<A::Action>>
where
    A: StateActionsMap,
    A::State: Hash + Eq + Clone,
    A::Action: Clone,
    T: Transition<State = A::State, Action = A::Action>,
    G: GoalTest<State = A::State>,
    C: PathCostSource<State = A::State, Action = A::Action, CostType = isize>,
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

// Some boilerplate to let me easily lookup and replace nodes in the priority queues.
// `orx-priority-queue` isn't clear about it in the documentation. But nodes must be
// `Clone + Eq + Hash`. The bounds aren't on the traits but the specific implementations.
//
// In this impl what matters is that each node corresponds to a single state. So the notion
// of Eq is defined with respect to that requirement.
#[derive(Clone)]
pub struct EqOnNodeState<S, A, N>(N, PhantomData<(S, A)>)
where
    N: AsRef<Node<S, A>> + Clone;

impl<S, A, N: AsRef<Node<S, A>> + Clone> From<N> for EqOnNodeState<S, A, N> {
    fn from(value: N) -> Self {
        EqOnNodeState(value, PhantomData::default())
    }
}

impl<S: PartialEq, A, N: AsRef<Node<S, A>> + Clone> PartialEq for EqOnNodeState<S, A, N> {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref().state.eq(&other.0.as_ref().state)
    }
}

impl<S: Eq, A, N: AsRef<Node<S, A>> + Clone> Eq for EqOnNodeState<S, A, N> {}

impl<S: Hash, A, N: AsRef<Node<S, A>> + Clone> Hash for EqOnNodeState<S, A, N> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ref().state.hash(state);
    }
}

// FIXME: Update optimal solution alongside optimal cost.
// Why is this called `uniform_cost_search` when it is searching for the solution with
// the lowest cost? Looking for the lowest cost among several solutions of equal length
// implies that the costs of each action differs.
pub fn uniform_cost_search<A, T, G, C, Q>(problem: Problem<A, T, G, C>) -> Option<Vec<A::Action>>
where
    A: StateActionsMap,
    A::State: Hash + Eq + Clone,
    A::Action: Clone,
    T: Transition<State = A::State, Action = A::Action>,
    G: GoalTest<State = A::State>,
    C: PathCostSource<State = A::State, Action = A::Action, CostType = isize>,
    Q: PriorityQueueDecKey<
            EqOnNodeState<A::State, A::Action, Rc<Node<A::State, A::Action>>>,
            isize,
        > + Default,
{
    // By default node cost is zero with the current impl.
    let root = Rc::new(Node::from(problem.initial_state.clone()));

    // New priority queue
    let mut frontier = Q::default();
    frontier.push(root.clone().into(), root.path_cost());

    let mut explored = HashSet::new();

    while let Some(lowest_cost_node) = frontier.pop_node() {
        if problem.goal.is_goal(&lowest_cost_node.0.state) {
            return Some(lowest_cost_node.0.solution());
        }
        explored.insert(lowest_cost_node.0.state.clone());
        for act in problem.actions.actions(&lowest_cost_node.0.state) {
            let child_node = generate(&problem, &lowest_cost_node.0, act);
            let child_cost = child_node.path_cost();

            let child_node = EqOnNodeState::from(Rc::new(child_node));

            if !(explored.contains(&child_node.0.state) || frontier.contains(&child_node)) {
                frontier.push(child_node, child_cost);
            } else if frontier.contains(&child_node) {
                // Update the node if we have found a better cost. This currently
                // does not work, because this will update the cost but not the parent
                // of the node which would lead to that better cost.
                //
                // The implementation here should be swapped, so that the entire node
                // is stored as the key and the state is used as the node. That involves
                // switching `EqOnNodeState` to be `OrdOnNodeCost` and implementing
                // `PartialOrd` on the wrapper. For now though this will return suboptimal
                // solutions for problems that don't have uniform costs.
                frontier.try_decrease_key(&child_node, child_cost);
            }
        }
    }

    None
}
