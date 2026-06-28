use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::world::WorldGrid;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Node {
    x: i32,
    y: i32,
}

impl Node {
    fn manhattan(self, other: Node) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Scored {
    f: f32,
    node: Node,
}

impl Eq for Scored {}

impl Ord for Scored {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f
            .partial_cmp(&self.f)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for Scored {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn find_path(
    grid: &WorldGrid,
    start: (i32, i32),
    goal: (i32, i32),
) -> Option<Vec<(i32, i32)>> {
    if start == goal {
        return Some(vec![goal]);
    }
    if !grid.is_walkable(goal.0, goal.1) {
        return None;
    }

    let start_node = Node {
        x: start.0,
        y: start.1,
    };
    let goal_node = Node {
        x: goal.0,
        y: goal.1,
    };

    let mut open = BinaryHeap::new();
    let mut g_score: HashMap<Node, f32> = HashMap::new();
    let mut came_from: HashMap<Node, Node> = HashMap::new();

    g_score.insert(start_node, 0.0);
    open.push(Scored {
        f: start_node.manhattan(goal_node) as f32,
        node: start_node,
    });

    const DIRS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

    while let Some(Scored { node: current, .. }) = open.pop() {
        if current == goal_node {
            return Some(reconstruct(&came_from, current));
        }

        let current_g = *g_score.get(&current).unwrap_or(&f32::INFINITY);

        for (dx, dy) in DIRS {
            let neighbor = Node {
                x: current.x + dx,
                y: current.y + dy,
            };
            if !grid.is_walkable(neighbor.x, neighbor.y) {
                continue;
            }

            let tentative = current_g + 1.0;
            let prev = *g_score.get(&neighbor).unwrap_or(&f32::INFINITY);
            if tentative < prev {
                came_from.insert(neighbor, current);
                g_score.insert(neighbor, tentative);
                let f = tentative + neighbor.manhattan(goal_node) as f32;
                open.push(Scored { f, node: neighbor });
            }
        }
    }

    None
}

fn reconstruct(came_from: &HashMap<Node, Node>, mut current: Node) -> Vec<(i32, i32)> {
    let mut path = vec![(current.x, current.y)];
    while let Some(&prev) = came_from.get(&current) {
        current = prev;
        path.push((current.x, current.y));
    }
    path.reverse();
    path
}
