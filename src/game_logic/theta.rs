use crate::game_logic::theta_grid::ThetaGrid;
use bevy::prelude::Component;
use rand::Rng;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;
use crate::game_logic::TILE_SIZE;

#[derive(Default)]
pub enum ThetaCommand {
    #[default]
    Stop,
    Forward,
    Reverse,
    TurnLeft,
    TurnRight,
}

#[derive(Clone)]
pub struct ThetaCheckpoint {
    pub point1: (f32, f32),
    pub point2: (f32, f32),
}

impl ThetaCheckpoint {
    pub fn new(point1: (f32, f32), point2: (f32, f32)) -> Self {
        Self { point1, point2 }
    }
}

#[derive(Component)]
pub struct ThetaCheckpointList {
    pub checkpoints: Vec<ThetaCheckpoint>,
    pub current_checkpoint_index: usize,
    pub cached_path: Vec<(usize, usize)>,
    pub path_index: usize,
    pub target_world_pos: Option<(f32, f32)>,
}

impl ThetaCheckpointList {
    pub fn new(checkpoints: Vec<ThetaCheckpoint>) -> Self {
        ThetaCheckpointList {
            checkpoints,
            current_checkpoint_index: 0,
            cached_path: Vec::new(),
            path_index: 0,
            target_world_pos: None,
        }
    }

    pub fn advance_checkpoint(&mut self) {
        self.current_checkpoint_index =
            (self.current_checkpoint_index + 1) % self.checkpoints.len();
    }
    pub fn load_checkpoint_list(&mut self, map_num: u8) -> ThetaCheckpointList {
        let mut checkpoints: Vec<ThetaCheckpoint> = Vec::new();
        if (map_num == 1) {
            checkpoints.push(ThetaCheckpoint::new((91.0, 18.0), (94.0, 18.0)));
            checkpoints.push(ThetaCheckpoint::new((91.0, 10.0), (94.0, 10.0)));
            checkpoints.push(ThetaCheckpoint::new((85.0, 9.0), (85.0, 5.0)));
            checkpoints.push(ThetaCheckpoint::new((59.0, 5.0), (59.0, 8.0)));
            checkpoints.push(ThetaCheckpoint::new((54.0, 11.0), (57.0, 11.0)));
            checkpoints.push(ThetaCheckpoint::new((54.0, 20.0), (52.0, 18.0)));
            checkpoints.push(ThetaCheckpoint::new((49.0, 30.0), (52.0, 30.0)));
            checkpoints.push(ThetaCheckpoint::new((45.0, 38.0), (45.0, 41.0)));
            checkpoints.push(ThetaCheckpoint::new((31.0, 41.0), (31.0, 38.0)));
            checkpoints.push(ThetaCheckpoint::new((20.0, 46.0), (20.0, 43.0)));
            checkpoints.push(ThetaCheckpoint::new((11.0, 47.0), (11.0, 43.0)));
            checkpoints.push(ThetaCheckpoint::new((5.0, 50.0), (8.0, 50.0)));
            checkpoints.push(ThetaCheckpoint::new((8.0, 56.0), (5.0, 56.0)));
            checkpoints.push(ThetaCheckpoint::new((16.0, 68.0), (18.0, 68.0)));
            checkpoints.push(ThetaCheckpoint::new((16.0, 74.0), (19.0, 74.0)));
            checkpoints.push(ThetaCheckpoint::new((7.0, 84.0), (10.0, 84.0)));
            checkpoints.push(ThetaCheckpoint::new((15.0, 94.0), (15.0, 91.0)));
            checkpoints.push(ThetaCheckpoint::new((33.0, 94.0), (33.0, 91.0)));
            checkpoints.push(ThetaCheckpoint::new((35.0, 89.0), (38.0, 89.0)));
            checkpoints.push(ThetaCheckpoint::new((40.0, 86.0), (40.0, 83.0)));
            checkpoints.push(ThetaCheckpoint::new((53.0, 83.0), (53.0, 86.0)));
            checkpoints.push(ThetaCheckpoint::new((59.0, 89.0), (54.0, 89.0)));
            checkpoints.push(ThetaCheckpoint::new((60.0, 91.0), (60.0, 94.0)));
            checkpoints.push(ThetaCheckpoint::new((89.0, 91.0), (89.0, 94.0)));
            checkpoints.push(ThetaCheckpoint::new((91.0, 89.0), (94.0, 89.0)));
            checkpoints.push(ThetaCheckpoint::new((91.0, 34.0), (93.0, 44.0)));
        } else if (map_num == 2) {
            // No checkpoints implemented yet!
            panic!("Checkpoints not implemented for map 2 yet!");
        } else {
            panic!("Invalid map num: {}", map_num);
        }
        return ThetaCheckpointList::new(checkpoints);
    }
}

pub fn get_next_point(list: &ThetaCheckpointList, grid: &ThetaGrid) -> (f32, f32) {
    let mut rng = rand::thread_rng();

    let curr_checkpoint: ThetaCheckpoint = list.checkpoints[list.current_checkpoint_index].clone();

    let rand_x_tile: f32 = if curr_checkpoint.point1.0 < curr_checkpoint.point2.0 {
        rng.gen_range(curr_checkpoint.point1.0..=curr_checkpoint.point2.0)
    } else {
        rng.gen_range(curr_checkpoint.point2.0..=curr_checkpoint.point1.0)
    };

    let rand_y_tile: f32 = if curr_checkpoint.point1.1 < curr_checkpoint.point2.1 {
        rng.gen_range(curr_checkpoint.point1.1..=curr_checkpoint.point2.1)
    } else {
        rng.gen_range(curr_checkpoint.point2.1..=curr_checkpoint.point1.1)
    };

    // Convert tile coordinates to world coordinates
    let world_x = (rand_x_tile * TILE_SIZE as f32) - (grid.width as f32 * TILE_SIZE as f32 / 2.0) + (TILE_SIZE as f32 / 2.0);
    let world_y = -((rand_y_tile * TILE_SIZE as f32) - (grid.height as f32 * TILE_SIZE as f32 / 2.0) + (TILE_SIZE as f32 / 2.0));


    return (world_x, world_y);
}

// Helper function to calculate steering command toward a target position
fn calculate_steering_command(
    start_pos: (f32, f32),
    target_pos: (f32, f32),
    current_angle: f32,
) -> ThetaCommand {
    let dx = target_pos.0 - start_pos.0;
    let dy = target_pos.1 - start_pos.1;

    // Negate dy because tile Y-axis is flipped (increasing Y goes down in world space)
    let target_angle = dy.atan2(dx);

    // Normalize current angle
    let pi = std::f32::consts::PI;
    let mut current_normalized = current_angle % (2.0 * pi);
    if current_normalized > pi {
        current_normalized -= 2.0 * pi;
    } else if current_normalized < -pi {
        current_normalized += 2.0 * pi;
    }

    let mut angle_diff = target_angle - current_normalized;

    if angle_diff > std::f32::consts::PI {
        angle_diff -= 2.0 * std::f32::consts::PI;
    } else if angle_diff < -std::f32::consts::PI {
        angle_diff += 2.0 * std::f32::consts::PI;
    }

    // Give it some wiggle room so it doesn't oscillate (Greyson's idea)
    let angle_threshold = 0.1; // radians (~5.7 degrees)
    let reverse_threshold = pi * 0.85; // ~153 degrees - only reverse if target is almost directly behind

    if angle_diff.abs() > reverse_threshold {
        ThetaCommand::Reverse
    } else if angle_diff.abs() < angle_threshold {
        ThetaCommand::Forward
    } else if angle_diff > 0.0 {
        ThetaCommand::TurnLeft
    } else {
        ThetaCommand::TurnRight
    }
}

pub fn track_to_position(start_pos: (f32, f32), current_angle: f32, checkpoints: &mut ThetaCheckpointList, grid: &ThetaGrid) -> ThetaCommand {
    if checkpoints.checkpoints.is_empty() {
        return ThetaCommand::Stop;
    }

    //Grab the current checkpoint from the checkpoint list
    let end_pos = get_next_point(&checkpoints, &grid);

    //Calc that distance
    let dx = end_pos.0 - start_pos.0;
    let dy = end_pos.1 - start_pos.1;
    let distance = (dx * dx + dy * dy).sqrt();

    let goal_threshold = 5.0; // pixels
    if distance < goal_threshold {
        println!("ADVANCE");
        checkpoints.advance_checkpoint();
    }

    calculate_steering_command(start_pos, end_pos, current_angle)
}

// Node for priority queue in Theta*
#[derive(Clone, Copy, PartialEq)]
struct Node {
    pos: (usize, usize),
    f_score: f32,
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.f_score.partial_cmp(&self.f_score)
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

// Euclidean distance
fn heuristic(pos: (usize, usize), goal: (usize, usize)) -> f32 {
    let dx = (pos.0 as f32 - goal.0 as f32).abs();
    let dy = (pos.1 as f32 - goal.1 as f32).abs();
    (dx * dx + dy * dy).sqrt()
}

// Compute the cost between two grid positions considering terrain
fn movement_cost(grid: &ThetaGrid, from: (usize, usize), to: (usize, usize)) -> f32 {
    if let Some(node) = grid.get_node(to.0, to.1) {
        let dx = (to.0 as f32 - from.0 as f32).abs();
        let dy = (to.1 as f32 - from.1 as f32).abs();
        let distance = (dx * dx + dy * dy).sqrt();
        distance * node.cost
    } else {
        f32::INFINITY
    }
}

// Theta* pathfinding algorithm
pub fn generate_theta_star_path(
    grid: &ThetaGrid,
    start: (usize, usize),
    goal: (usize, usize),
) -> Option<Vec<(usize, usize)>> {
    let mut open = BinaryHeap::new();
    let mut closed = HashSet::new();
    let mut g_score: HashMap<(usize, usize), f32> = HashMap::new();
    let mut parent: HashMap<(usize, usize), (usize, usize)> = HashMap::new();

    // Initialize start node
    g_score.insert(start, 0.0);
    parent.insert(start, start);
    open.push(Node {
        pos: start,
        f_score: heuristic(start, goal),
    });

    while let Some(current_node) = open.pop() {
        let current = current_node.pos;

        // Goal reached
        if current == goal {
            return Some(reconstruct_path(&parent, current));
        }

        // Skip if already processed
        if closed.contains(&current) {
            continue;
        }
        closed.insert(current);

        // Process neighbors
        let neighbors = grid.get_neighbors(current.0, current.1);
        for neighbor_node in neighbors {
            let neighbor = (neighbor_node.x, neighbor_node.y);

            if closed.contains(&neighbor) {
                continue;
            }

            // Initialize neighbor if not seen before
            g_score.entry(neighbor).or_insert(f32::INFINITY);

            // Update vertex (Theta* logic)
            update_vertex(grid, &mut g_score, &mut parent, &mut open, current, neighbor, goal);
        }
    }

    None // No path found
}

// Update vertex with Theta* logic
fn update_vertex(
    grid: &ThetaGrid,
    g_score: &mut HashMap<(usize, usize), f32>,
    parent: &mut HashMap<(usize, usize), (usize, usize)>,
    open: &mut BinaryHeap<Node>,
    current: (usize, usize),
    neighbor: (usize, usize),
    goal: (usize, usize),
) {
    let g_old = *g_score.get(&neighbor).unwrap_or(&f32::INFINITY);

    // Compute cost using Theta* logic
    compute_cost(grid, g_score, parent, current, neighbor);

    let g_new = *g_score.get(&neighbor).unwrap_or(&f32::INFINITY);

    // If we found a better path, add to open set
    if g_new < g_old {
        let f_score = g_new + heuristic(neighbor, goal);
        open.push(Node {
            pos: neighbor,
            f_score,
        });
    }
}

// Compute cost with line-of-sight optimization
fn compute_cost(
    grid: &ThetaGrid,
    g_score: &mut HashMap<(usize, usize), f32>,
    parent: &mut HashMap<(usize, usize), (usize, usize)>,
    current: (usize, usize),
    neighbor: (usize, usize),
) {
    let current_parent = *parent.get(&current).unwrap_or(&current);

    // Path 2: Try to connect neighbor directly to current's parent (any-angle path)
    if grid.line_of_sight((current_parent.0 as f32, current_parent.1 as f32),
                          (neighbor.0 as f32, neighbor.1 as f32)) {
        let g_parent = *g_score.get(&current_parent).unwrap_or(&f32::INFINITY);
        let cost = movement_cost(grid, current_parent, neighbor);
        let new_g = g_parent + cost;

        if new_g < *g_score.get(&neighbor).unwrap_or(&f32::INFINITY) {
            parent.insert(neighbor, current_parent);
            g_score.insert(neighbor, new_g);
        }
    } else {
        // Path 1: Connect neighbor to current (grid-aligned path)
        let g_current = *g_score.get(&current).unwrap_or(&f32::INFINITY);
        let cost = movement_cost(grid, current, neighbor);
        let new_g = g_current + cost;

        if new_g < *g_score.get(&neighbor).unwrap_or(&f32::INFINITY) {
            parent.insert(neighbor, current);
            g_score.insert(neighbor, new_g);
        }
    }
}

// Reconstruct the path from parent pointers
fn reconstruct_path(
    parent: &HashMap<(usize, usize), (usize, usize)>,
    mut current: (usize, usize),
) -> Vec<(usize, usize)> {
    let mut path = vec![current];
    while let Some(&p) = parent.get(&current) {
        if p == current {
            break; // Reached start
        }
        current = p;
        path.push(current);
    }
    path.reverse();
    path
}

// Wrapper function for Theta*
pub fn theta_star(
    start_pos: (f32, f32),
    current_angle: f32,
    checkpoints: &mut ThetaCheckpointList,
    grid: &ThetaGrid,
) -> ThetaCommand {
    if checkpoints.checkpoints.is_empty() {
        return ThetaCommand::Stop;
    }

    // Get or use cached target position for current checkpoint
    let target_pos = if let Some(cached_target) = checkpoints.target_world_pos {
        cached_target
    } else {
        // Generate new random point for this checkpoint
        let new_target = get_next_point(checkpoints, grid);
        checkpoints.target_world_pos = Some(new_target);
        new_target
    };

    // Check if we need to recompute the path
    let needs_recompute = checkpoints.cached_path.is_empty();

    if needs_recompute {
        // Convert world positions to grid coordinates
        let start_grid = grid.world_to_grid(start_pos.0, start_pos.1);
        let goal_grid = grid.world_to_grid(target_pos.0, target_pos.1);

        println!("Computing path from grid {:?} to grid {:?} (world: {:?} to {:?})",
                 start_grid, goal_grid, start_pos, target_pos);

        // Run Theta* pathfinding
        if let Some(path) = generate_theta_star_path(grid, start_grid, goal_grid) {
            println!("Path found with {} waypoints", path.len());
            checkpoints.cached_path = path;
            checkpoints.path_index = 0;
        } else {
            println!("NO PATH FOUND! Falling back to pure pursuit");
            // No path found, fall back to direct pursuit and check for checkpoint advance
            let dx = target_pos.0 - start_pos.0;
            let dy = target_pos.1 - start_pos.1;
            let distance = (dx * dx + dy * dy).sqrt();
            let goal_threshold = 128.0; // Leeway in pixels (2 tiles)

            if distance < goal_threshold {
                println!("ADVANCE");
                checkpoints.advance_checkpoint();
                checkpoints.cached_path.clear();
                checkpoints.target_world_pos = None;
            }
            return steer_towards(start_pos, current_angle, target_pos, checkpoints);
        }
    }

    // Follow the cached path
    if checkpoints.path_index >= checkpoints.cached_path.len() {
        // Reached end of path, advance checkpoint
        println!("ADVANCE");
        checkpoints.advance_checkpoint();
        checkpoints.cached_path.clear();
        checkpoints.target_world_pos = None;
        return ThetaCommand::Forward;
    }

    // Get the next waypoint in the path
    let next_grid_pos = checkpoints.cached_path[checkpoints.path_index];
    if let Some(node) = grid.get_node(next_grid_pos.0, next_grid_pos.1) {
        let waypoint = (node.world_x, node.world_y);

        // Check if we're close to the current waypoint
        let dx = waypoint.0 - start_pos.0;
        let dy = waypoint.1 - start_pos.1;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < 128.0 { // Leeway in world units (2 tiles)
            // Move to next waypoint
            checkpoints.path_index += 1;
            if checkpoints.path_index >= checkpoints.cached_path.len() {
                // Reached final waypoint, advance checkpoint
                println!("ADVANCE");
                checkpoints.advance_checkpoint();
                checkpoints.cached_path.clear();
                checkpoints.target_world_pos = None;
                return ThetaCommand::Forward;
            }
        }

        // Steer towards current waypoint
        steer_towards(start_pos, current_angle, waypoint, checkpoints)
    } else {
        // Invalid waypoint, recompute
        checkpoints.cached_path.clear();
        steer_towards(start_pos, current_angle, target_pos, checkpoints)
    }
}

// Helper function to steer towards a target position
fn steer_towards(
    start_pos: (f32, f32),
    current_angle: f32,
    target_pos: (f32, f32),
    _checkpoints: &ThetaCheckpointList,
) -> ThetaCommand {
    calculate_steering_command(start_pos, target_pos, current_angle)
}