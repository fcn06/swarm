#[allow(unused_imports)]
//use agent_core::graph::graph_definition::{Edge, Graph, Node};
use agent_models::graph::graph_definition::{Edge, Graph, Node};

use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

#[derive(Clone, Eq, PartialEq)]
struct State {
    cost: usize,
    position: String,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn a_star(graph: &Graph, start: &str, goal: &str) -> Option<Vec<String>> {
    let mut dist: HashMap<String, usize> = graph.nodes.keys().map(|id| (id.clone(), usize::MAX)).collect();
    let mut came_from: HashMap<String, String> = HashMap::new();
    let mut pq = BinaryHeap::new();

    dist.insert(start.to_string(), 0);
    pq.push(State { cost: 0, position: start.to_string() });

    while let Some(State { cost, position }) = pq.pop() {
        if position == goal {
            let mut path = Vec::new();
            let mut curr = goal.to_string();
            while let Some(prev) = came_from.get(&curr) {
                path.push(curr);
                curr = prev.clone();
            }
            path.push(start.to_string());
            path.reverse();
            return Some(path);
        }

        if cost > dist[&position] {
            continue;
        }

        for edge in &graph.edges {
            if edge.source == position {
                let neighbor = edge.target.clone();
                let new_cost = cost + 1; // Assuming edge weight is 1
                if new_cost < dist[&neighbor] {
                    dist.insert(neighbor.clone(), new_cost);
                    came_from.insert(neighbor.clone(), position.clone());
                    pq.push(State { cost: new_cost, position: neighbor });
                }
            }
        }
    }

    None
}
