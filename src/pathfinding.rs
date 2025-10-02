/// Core pathfinding module for discrete tick-based life simulation
/// Adapted from bevy_entitiles but simplified for our chunk-based world
use bevy::prelude::*;
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet},
};

use crate::tilemap::{chunk::CHUNK_SIZE, terrain::TerrainType};

// ============================================================================
// CORE DATA STRUCTURES
// ============================================================================

/// A single node in the A* pathfinding grid
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathNode {
    pub index: IVec2,
    pub parent: Option<IVec2>,
    pub g_cost: u32, // Cost from origin to this node
    pub h_cost: u32, // Heuristic (Manhattan distance to destination)
    pub cost_to_pass: u32, // Tile movement cost
}

impl PathNode {
    #[inline]
    pub fn new(index: IVec2, g_cost: u32, dest: IVec2, cost_to_pass: u32) -> Self {
        PathNode {
            index,
            parent: None,
            g_cost,
            h_cost: manhattan_distance(dest, index),
            cost_to_pass,
        }
    }

    #[inline]
    pub fn weight(&self) -> u32 {
        self.g_cost + self.h_cost // A* evaluation: f = g + h
    }
}

// BinaryHeap requires Ord - we want lowest cost first
impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap behavior
        other
            .g_cost
            .cmp(&self.g_cost)
            .then(other.h_cost.cmp(&self.h_cost))
    }
}

/// Computed path from origin to destination
#[derive(Component, Clone, Debug)]
pub struct Path {
    waypoints: Vec<IVec2>, // Path from origin to dest (not reversed!)
    current_index: usize,
}

impl Path {
    pub fn new(waypoints: Vec<IVec2>) -> Self {
        Self {
            waypoints,
            current_index: 0,
        }
    }

    /// Get current target tile (where entity should move next tick)
    pub fn current_target(&self) -> Option<IVec2> {
        self.waypoints.get(self.current_index).copied()
    }

    /// Advance to next waypoint (call this when entity moves)
    pub fn advance(&mut self) {
        if self.current_index < self.waypoints.len() {
            self.current_index += 1;
        }
    }

    /// Check if path is complete
    pub fn is_complete(&self) -> bool {
        self.current_index >= self.waypoints.len()
    }

    /// Get remaining waypoints
    pub fn remaining(&self) -> &[IVec2] {
        &self.waypoints[self.current_index..]
    }

    /// Get all waypoints
    pub fn all_waypoints(&self) -> &[IVec2] {
        &self.waypoints
    }
}

// ============================================================================
// PATHFINDING REQUEST & RESULT
// ============================================================================

/// Component: Entity wants to find a path
#[derive(Component, Debug)]
pub struct PathRequest {
    pub origin: IVec2,
    pub destination: IVec2,
    pub allow_diagonal: bool,
    pub max_steps: Option<u32>, // Prevent infinite loops
}

/// Component: Pathfinding failed for this entity
/// This is added when a path cannot be found
#[derive(Component, Debug)]
pub struct PathfindingFailed {
    pub attempted_destination: IVec2,
}

/// Resource: Grid of movement costs for pathfinding
/// This is built once from the terrain and updated when terrain changes
#[derive(Resource, Default)]
pub struct PathfindingGrid {
    costs: HashMap<IVec2, u32>, // tile_pos -> movement_cost
}

impl PathfindingGrid {
    pub fn new() -> Self {
        Self {
            costs: HashMap::new(),
        }
    }

    /// Set movement cost for a tile
    pub fn set_cost(&mut self, pos: IVec2, cost: u32) {
        self.costs.insert(pos, cost);
    }

    /// Get movement cost (returns u32::MAX for unknown/impassable tiles)
    pub fn get_cost(&self, pos: IVec2) -> u32 {
        self.costs.get(&pos).copied().unwrap_or(u32::MAX)
    }

    /// Check if tile is walkable
    pub fn is_walkable(&self, pos: IVec2) -> bool {
        self.get_cost(pos) < u32::MAX
    }

    /// Remove a tile (makes it impassable)
    pub fn remove(&mut self, pos: IVec2) {
        self.costs.remove(&pos);
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.costs.clear();
    }
}

// ============================================================================
// CORE A* ALGORITHM
// ============================================================================

/// Find a path using A* algorithm
/// Returns waypoints from origin to destination (in order)
pub fn find_path(
    origin: IVec2,
    destination: IVec2,
    grid: &PathfindingGrid,
    allow_diagonal: bool,
    max_steps: Option<u32>,
) -> Option<Path> {
    // Early exit if origin is not walkable
    if !grid.is_walkable(origin) {
        warn!("Pathfinding: Origin {:?} is not walkable! Cost: {}", origin, grid.get_cost(origin));
        return None;
    }
    
    // Early exit if destination is not walkable
    if !grid.is_walkable(destination) {
        warn!("Pathfinding: Destination {:?} is not walkable! Cost: {}", destination, grid.get_cost(destination));
        return None;
    }

    let mut to_explore: BinaryHeap<PathNode> = BinaryHeap::new();
    let mut explored: HashSet<IVec2> = HashSet::new();
    let mut all_nodes: HashMap<IVec2, PathNode> = HashMap::new();

    // Start with origin
    let origin_node = PathNode::new(origin, 0, destination, 0);
    to_explore.push(origin_node);
    all_nodes.insert(origin, origin_node);

    let mut steps = 0;

    // A* main loop
    while let Some(current) = to_explore.pop() {
        // Check step limit
        if let Some(max) = max_steps {
            if steps >= max {
                warn!("Pathfinding: Max steps ({}) reached. Origin: {:?}, Dest: {:?}, Explored: {}",
                    max, origin, destination, explored.len());
                return None; // Failed - too many steps
            }
        }
        steps += 1;

        // Reached destination!
        if current.index == destination {
            debug!("Pathfinding: SUCCESS! Steps: {}, Explored: {}", steps, explored.len());
            return Some(reconstruct_path(&all_nodes, origin, destination));
        }

        // Skip if we've already explored this with a better cost
        if explored.contains(&current.index) {
            continue;
        }
        explored.insert(current.index);

        // Check all neighbors
        let neighbors = get_neighbors(current.index, allow_diagonal);
        for neighbor_pos in neighbors {
            if explored.contains(&neighbor_pos) {
                continue;
            }

            let tile_cost = grid.get_cost(neighbor_pos);
            if tile_cost == u32::MAX {
                continue; // Impassable
            }

            let new_g_cost = current.g_cost + tile_cost;

            // Check if this is a better path to this neighbor
            if let Some(existing) = all_nodes.get(&neighbor_pos) {
                if existing.g_cost <= new_g_cost {
                    continue; // Already have better path
                }
            }

            // Create/update node
            let mut neighbor_node = PathNode::new(neighbor_pos, new_g_cost, destination, tile_cost);
            neighbor_node.parent = Some(current.index);
            all_nodes.insert(neighbor_pos, neighbor_node);
            to_explore.push(neighbor_node);
        }
    }

    warn!("Pathfinding: No path found! Origin: {:?}, Dest: {:?}, Steps: {}, Explored: {}, To explore: {}",
        origin, destination, steps, explored.len(), to_explore.len());
    None // No path found
}

/// Reconstruct path by following parent links (returns waypoints in correct order)
fn reconstruct_path(
    all_nodes: &HashMap<IVec2, PathNode>,
    origin: IVec2,
    destination: IVec2,
) -> Path {
    let mut sparse_waypoints = Vec::new();
    let mut current = destination;

    // Walk backwards from destination to origin (these are the A* nodes)
    while current != origin {
        sparse_waypoints.push(current);
        if let Some(node) = all_nodes.get(&current) {
            if let Some(parent) = node.parent {
                current = parent;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    sparse_waypoints.push(origin);

    // Reverse to get origin -> destination order
    sparse_waypoints.reverse();

    // Fill in all intermediate tiles between A* waypoints
    let mut full_waypoints = Vec::new();
    
    for i in 0..sparse_waypoints.len() - 1 {
        let start = sparse_waypoints[i];
        let end = sparse_waypoints[i + 1];
        
        // Add all tiles from start to end (excluding start, including end)
        let mut intermediate = interpolate_tiles(start, end);
        full_waypoints.append(&mut intermediate);
    }

    Path::new(full_waypoints)
}

/// Generate all tiles between two points (excluding start, including end)
/// Assumes points are connected (A* guarantees this)
fn interpolate_tiles(start: IVec2, end: IVec2) -> Vec<IVec2> {
    let mut tiles = Vec::new();
    let mut current = start;
    
    // Calculate direction
    let delta = end - start;
    let steps = delta.x.abs().max(delta.y.abs());
    
    if steps == 0 {
        return tiles; // Same tile, no interpolation needed
    }
    
    // Determine step direction for each axis
    let step_x = if delta.x != 0 { delta.x.signum() } else { 0 };
    let step_y = if delta.y != 0 { delta.y.signum() } else { 0 };
    
    // Generate all intermediate tiles
    for _ in 0..steps {
        current = IVec2::new(current.x + step_x, current.y + step_y);
        tiles.push(current);
    }
    
    tiles
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Manhattan distance heuristic
#[inline]
fn manhattan_distance(a: IVec2, b: IVec2) -> u32 {
    let d = (a - b).abs();
    (d.x + d.y) as u32
}

/// Get neighbor tiles (4-directional or 8-directional)
fn get_neighbors(pos: IVec2, allow_diagonal: bool) -> Vec<IVec2> {
    let orthogonal = vec![
        pos + IVec2::new(0, 1),  // North
        pos + IVec2::new(1, 0),  // East
        pos + IVec2::new(0, -1), // South
        pos + IVec2::new(-1, 0), // West
    ];

    if !allow_diagonal {
        return orthogonal;
    }

    // Add diagonal neighbors
    let mut neighbors = orthogonal;
    neighbors.extend_from_slice(&[
        pos + IVec2::new(1, 1),   // NE
        pos + IVec2::new(1, -1),  // SE
        pos + IVec2::new(-1, -1), // SW
        pos + IVec2::new(-1, 1),  // NW
    ]);
    neighbors
}

// ============================================================================
// TERRAIN COST MAPPING
// ============================================================================

/// Convert terrain to pathfinding cost (uses existing TerrainType::movement_cost)
pub fn terrain_to_pathfinding_cost(terrain: &TerrainType) -> u32 {
    // Convert f32 to u32, and treat very high costs as impassable
    let cost = terrain.movement_cost();
    if cost >= 1000.0 {
        u32::MAX // Impassable
    } else {
        cost as u32
    }
}

// ============================================================================
// BEVY SYSTEMS (Non-tick systems - run every frame)
// ============================================================================

/// System: Process pathfinding requests (runs async, not synced to ticks)
pub fn process_pathfinding_requests(
    mut commands: Commands,
    requests: Query<(Entity, &PathRequest)>,
    grid: Res<PathfindingGrid>,
) {
    for (entity, request) in requests.iter() {
        // Calculate path
        if let Some(path) = find_path(
            request.origin,
            request.destination,
            &grid,
            request.allow_diagonal,
            request.max_steps,
        ) {
            debug!(
                "Path found for entity {:?}: {} waypoints from {:?} to {:?}",
                entity.index(),
                path.waypoints.len(),
                request.origin,
                request.destination
            );
            // Path found - attach to entity and remove request
            commands.entity(entity).insert(path);
        } else {
            // Path not found - add PathfindingFailed component
            info!(
                "Pathfinding failed for entity {:?}: {:?} -> {:?}",
                entity, request.origin, request.destination
            );
            commands.entity(entity).insert(PathfindingFailed {
                attempted_destination: request.destination,
            });
        }

        // Remove request (processed)
        commands.entity(entity).remove::<PathRequest>();
    }
}

// ============================================================================
// INTEGRATION HELPERS
// ============================================================================

/// Build PathfindingGrid from loaded world chunks
pub fn build_pathfinding_grid_from_world(
    world_loader: &crate::world_loader::WorldLoader,
) -> PathfindingGrid {
    let mut grid = PathfindingGrid::new();

    // Iterate through all chunks in the world loader's internal data
    // Note: WorldLoader stores chunks in a HashMap, so we need to iterate differently
    // For now, return an empty grid - this needs to be implemented when WorldLoader
    // exposes an iterator or we can access its chunks
    
    // TODO: Add iter_chunks() method to WorldLoader
    warn!("build_pathfinding_grid_from_world: Not yet fully implemented");
    warn!("Pathfinding grid will need to be manually populated or WorldLoader needs an iterator");
    
    grid
}

/// Update pathfinding grid when terrain changes
pub fn update_pathfinding_grid_for_tile(
    grid: &mut PathfindingGrid,
    tile_pos: IVec2,
    new_terrain: TerrainType,
) {
    grid.set_cost(tile_pos, terrain_to_pathfinding_cost(&new_terrain));
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_straight_line_path() {
        let mut grid = PathfindingGrid::new();

        // Create a simple 5x5 walkable grid
        for y in 0..5 {
            for x in 0..5 {
                grid.set_cost(IVec2::new(x, y), 1);
            }
        }

        let path = find_path(
            IVec2::new(0, 0),
            IVec2::new(4, 0),
            &grid,
            false,
            None,
        );

        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.waypoints.len(), 4); // 0->1->2->3->4 (origin not included)
    }

    #[test]
    fn test_obstacle_avoidance() {
        let mut grid = PathfindingGrid::new();

        // Create 5x5 grid with wall in middle
        for y in 0..5 {
            for x in 0..5 {
                if x == 2 && y != 4 {
                    // Wall at x=2 except top
                    grid.set_cost(IVec2::new(x, y), u32::MAX);
                } else {
                    grid.set_cost(IVec2::new(x, y), 1);
                }
            }
        }

        let path = find_path(
            IVec2::new(0, 0),
            IVec2::new(4, 0),
            &grid,
            false,
            None,
        );

        assert!(path.is_some());
        let path = path.unwrap();
        // Path should go around obstacle
        assert!(path.waypoints.len() > 4);
    }

    #[test]
    fn test_no_path_exists() {
        let mut grid = PathfindingGrid::new();

        // Create destination surrounded by impassable tiles
        grid.set_cost(IVec2::new(0, 0), 1);
        grid.set_cost(IVec2::new(5, 5), 1);
        // Everything else is impassable

        let path = find_path(
            IVec2::new(0, 0),
            IVec2::new(5, 5),
            &grid,
            false,
            Some(100),
        );

        assert!(path.is_none());
    }

    #[test]
    fn test_manhattan_distance() {
        assert_eq!(manhattan_distance(IVec2::new(0, 0), IVec2::new(3, 4)), 7);
        assert_eq!(manhattan_distance(IVec2::new(5, 5), IVec2::new(2, 1)), 7);
    }
}
