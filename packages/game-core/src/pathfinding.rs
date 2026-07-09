use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::content::ContentRegistry;
use crate::world::WorldGrid;

const SQRT_2: f32 = std::f32::consts::SQRT_2;

const NEIGHBORS: [((i32, i32), f32); 8] = [
    ((1, 0), 1.0),
    ((-1, 0), 1.0),
    ((0, 1), 1.0),
    ((0, -1), 1.0),
    ((1, 1), SQRT_2),
    ((1, -1), SQRT_2),
    ((-1, 1), SQRT_2),
    ((-1, -1), SQRT_2),
];

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Node {
    x: i32,
    y: i32,
}

impl Node {
    fn octile_distance(self, other: Node) -> f32 {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        let d = 1.0_f32;
        d * (dx + dy) as f32 + (SQRT_2 - 2.0 * d) * dx.min(dy) as f32
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
    content: &ContentRegistry,
    start: (i32, i32),
    goal: (i32, i32),
) -> Option<Vec<(i32, i32)>> {
    find_path_avoiding(grid, content, start, goal, &[])
}

pub fn find_path_avoiding(
    grid: &WorldGrid,
    content: &ContentRegistry,
    start: (i32, i32),
    goal: (i32, i32),
    blocked: &[(i32, i32)],
) -> Option<Vec<(i32, i32)>> {
    if start == goal {
        return Some(vec![goal]);
    }
    if !grid.is_walkable(content, goal.0, goal.1) {
        return None;
    }

    let is_blocked = |x: i32, y: i32| {
        if (x, y) == goal {
            return false;
        }
        blocked.iter().any(|&(bx, by)| bx == x && by == y)
    };

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
        f: start_node.octile_distance(goal_node),
        node: start_node,
    });

    while let Some(Scored { node: current, .. }) = open.pop() {
        if current == goal_node {
            return Some(reconstruct(&came_from, current));
        }

        let current_g = *g_score.get(&current).unwrap_or(&f32::INFINITY);

        for ((dx, dy), step_cost) in NEIGHBORS {
            let neighbor = Node {
                x: current.x + dx,
                y: current.y + dy,
            };
            if !can_step(grid, content, current, dx, dy, &is_blocked) {
                continue;
            }

            let tentative = current_g + step_cost;
            let prev = *g_score.get(&neighbor).unwrap_or(&f32::INFINITY);
            if tentative < prev {
                came_from.insert(neighbor, current);
                g_score.insert(neighbor, tentative);
                let f = tentative + neighbor.octile_distance(goal_node);
                open.push(Scored { f, node: neighbor });
            }
        }
    }

    None
}

fn can_step(
    grid: &WorldGrid,
    content: &ContentRegistry,
    from: Node,
    dx: i32,
    dy: i32,
    is_blocked: &impl Fn(i32, i32) -> bool,
) -> bool {
    let nx = from.x + dx;
    let ny = from.y + dy;
    if !grid.is_walkable(content, nx, ny) || is_blocked(nx, ny) {
        return false;
    }
    if dx != 0 && dy != 0 {
        if !grid.is_walkable(content, from.x + dx, from.y) || is_blocked(from.x + dx, from.y) {
            return false;
        }
        if !grid.is_walkable(content, from.x, from.y + dy) || is_blocked(from.x, from.y + dy) {
            return false;
        }
    }
    true
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

const ORTHOGONAL_DIRS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// Returns the orthogonally adjacent walkable cell reachable from `from` with the shortest path.
pub fn best_adjacent_stand(
    grid: &WorldGrid,
    content: &ContentRegistry,
    building: (i32, i32),
    from: (i32, i32),
) -> Option<(i32, i32)> {
    best_adjacent_stand_filtered(grid, content, building, from, |_| true)
}

/// Like `best_adjacent_stand`, but skips stands for which `is_available` returns false.
pub fn best_adjacent_stand_filtered<F>(
    grid: &WorldGrid,
    content: &ContentRegistry,
    building: (i32, i32),
    from: (i32, i32),
    mut is_available: F,
) -> Option<(i32, i32)>
where
    F: FnMut((i32, i32)) -> bool,
{
    let mut best: Option<((i32, i32), usize)> = None;

    for (dx, dy) in ORTHOGONAL_DIRS {
        let stand = (building.0 + dx, building.1 + dy);
        if !grid.is_walkable(content, stand.0, stand.1) {
            continue;
        }
        if !is_available(stand) {
            continue;
        }
        if let Some(path) = find_path(grid, content, from, stand) {
            let len = path.len();
            if best.map(|(_, best_len)| len < best_len).unwrap_or(true) {
                best = Some((stand, len));
            }
        }
    }

    best.map(|(stand, _)| stand)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::base_content;
    use crate::world::WORLD_SIZE;

    fn test_content() -> ContentRegistry {
        base_content()
    }

    fn grass_grid(content: &ContentRegistry) -> WorldGrid {
        let len = (WORLD_SIZE * WORLD_SIZE) as usize;
        WorldGrid {
            terrain: vec![content.grass_terrain; len],
            buildings: vec![None; len],
            seed: 0,
        }
    }

    fn path_has_diagonal_step(path: &[(i32, i32)]) -> bool {
        path.windows(2).any(|w| {
            let (x0, y0) = w[0];
            let (x1, y1) = w[1];
            (x1 - x0).abs() == 1 && (y1 - y0).abs() == 1
        })
    }

    fn path_cost(path: &[(i32, i32)]) -> f32 {
        path.windows(2)
            .map(|w| {
                let dx = (w[1].0 - w[0].0).abs();
                let dy = (w[1].1 - w[0].1).abs();
                if dx == 1 && dy == 1 {
                    SQRT_2
                } else {
                    1.0
                }
            })
            .sum()
    }

    #[test]
    fn open_area_path_prefers_diagonal_shortcut() {
        let content = test_content();
        let grid = grass_grid(&content);

        let path = find_path(&grid, &content, (0, 0), (4, 4)).expect("path should exist");

        assert!(
            path_has_diagonal_step(&path),
            "expected diagonal waypoints, got {:?}",
            path
        );
        assert!(
            path_cost(&path) < 8.0,
            "diagonal route should beat orthogonal-only cost of 8, got {}",
            path_cost(&path)
        );
    }

    #[test]
    fn diagonal_step_blocked_when_corner_bounded_by_walls() {
        let content = test_content();
        let mut grid = grass_grid(&content);

        assert!(grid.place_building(&content, 6, 5, content.wall_building));
        assert!(grid.place_building(&content, 5, 6, content.wall_building));

        let path = find_path(&grid, &content, (5, 5), (6, 6)).expect("path should exist");

        assert!(
            !path.windows(2).any(|w| w[0] == (5, 5) && w[1] == (6, 6)),
            "corner-cutting diagonal should be blocked, got {:?}",
            path
        );
        assert!(path.len() > 2, "path must route around the corner");
    }
}
