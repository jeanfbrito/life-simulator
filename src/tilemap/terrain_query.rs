use super::{Chunk, ChunkCoordinate, ChunkManager, TerrainType};
use bevy::prelude::*;

#[derive(Component)]
pub struct PositionComponent {
    pub x: f32,
    pub y: f32,
}

impl PositionComponent {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &PositionComponent) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn to_chunk_coordinate(&self, tile_size: f32) -> ChunkCoordinate {
        ChunkCoordinate::from_world_position(self.x, self.y, tile_size)
    }

    pub fn to_tile_position(&self, tile_size: f32) -> (i32, i32) {
        let tile_x = (self.x / tile_size).floor() as i32;
        let tile_y = (self.y / tile_size).floor() as i32;
        (tile_x, tile_y)
    }
}

#[derive(Debug, Clone)]
pub struct TerrainQuery {
    pub position: (i32, i32),
    pub terrain_type: Option<TerrainType>,
    pub is_walkable: bool,
    pub movement_cost: f32,
    pub fertility: f32,
    pub resource_potential: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PathfindingNode {
    pub position: (i32, i32),
    pub g_cost: f32,
    pub h_cost: f32,
    pub f_cost: f32,
    pub parent: Option<(i32, i32)>,
}

impl PathfindingNode {
    pub fn new(position: (i32, i32), g_cost: f32, h_cost: f32) -> Self {
        let f_cost = g_cost + h_cost;
        Self {
            position,
            g_cost,
            h_cost,
            f_cost,
            parent: None,
        }
    }
}

pub struct TerrainQuerySystem;

impl TerrainQuerySystem {
    pub fn get_terrain_at_position(
        chunk_manager: &ChunkManager,
        tile_x: i32,
        tile_y: i32,
        chunks_query: &Query<&Chunk>,
    ) -> Option<TerrainQuery> {
        let chunk_coord = ChunkCoordinate::from_tile_position(tile_x, tile_y);

        if let Some(chunk_entity) = chunk_manager.get_chunk_entity(&chunk_coord) {
            if let Ok(chunk) = chunks_query.get(chunk_entity) {
                if let Some(terrain_type) = chunk.get_tile_at_world_position(tile_x, tile_y) {
                    return Some(TerrainQuery {
                        position: (tile_x, tile_y),
                        terrain_type: Some(terrain_type),
                        is_walkable: terrain_type.is_walkable(),
                        movement_cost: terrain_type.movement_cost(),
                        fertility: terrain_type.fertility(),
                        resource_potential: terrain_type
                            .resource_potential()
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect(),
                    });
                }
            }
        }

        None
    }

    pub fn get_terrain_in_radius(
        chunk_manager: &ChunkManager,
        center_x: i32,
        center_y: i32,
        radius: i32,
        chunks_query: &Query<&Chunk>,
    ) -> Vec<TerrainQuery> {
        let mut results = Vec::new();

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let x = center_x + dx;
                let y = center_y + dy;

                if let Some(terrain) =
                    Self::get_terrain_at_position(chunk_manager, x, y, chunks_query)
                {
                    results.push(terrain);
                }
            }
        }

        results
    }

    pub fn find_walkable_area(
        chunk_manager: &ChunkManager,
        start_x: i32,
        start_y: i32,
        max_radius: i32,
        chunks_query: &Query<&Chunk>,
    ) -> Vec<(i32, i32)> {
        let mut walkable_positions = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut to_visit = vec![(start_x, start_y)];

        while let Some((x, y)) = to_visit.pop() {
            if visited.contains(&(x, y)) {
                continue;
            }

            visited.insert((x, y));

            let distance_from_start = ((x - start_x).abs() + (y - start_y).abs()) as i32;
            if distance_from_start > max_radius {
                continue;
            }

            if let Some(terrain) = Self::get_terrain_at_position(chunk_manager, x, y, chunks_query)
            {
                if terrain.is_walkable {
                    walkable_positions.push((x, y));

                    // Add neighbors for further exploration
                    for (dx, dy) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        let nx = x + dx;
                        let ny = y + dy;
                        if !visited.contains(&(nx, ny)) {
                            to_visit.push((nx, ny));
                        }
                    }
                }
            }
        }

        walkable_positions
    }

    pub fn find_path(
        chunk_manager: &ChunkManager,
        start: (i32, i32),
        goal: (i32, i32),
        chunks_query: &Query<&Chunk>,
    ) -> Option<Vec<(i32, i32)>> {
        let mut open_set = std::collections::BinaryHeap::new();
        let mut closed_set = std::collections::HashSet::new();
        let mut nodes = std::collections::HashMap::new();

        let start_node = PathfindingNode::new(start, 0.0, Self::heuristic(start, goal));
        open_set.push(std::cmp::Reverse((start_node.f_cost.to_bits(), start)));
        nodes.insert(start, start_node);

        while let Some(std::cmp::Reverse((_, current_pos))) = open_set.pop() {
            if current_pos == goal {
                return Self::reconstruct_path(&nodes, current_pos);
            }

            closed_set.insert(current_pos);

            let current_node = nodes[&current_pos].clone();

            // Check neighbors
            for (dx, dy) in &[
                (-1, 0),
                (1, 0),
                (0, -1),
                (0, 1),
                (-1, -1),
                (-1, 1),
                (1, -1),
                (1, 1),
            ] {
                let neighbor_pos = (current_pos.0 + dx, current_pos.1 + dy);

                if closed_set.contains(&neighbor_pos) {
                    continue;
                }

                if let Some(terrain) = Self::get_terrain_at_position(
                    chunk_manager,
                    neighbor_pos.0,
                    neighbor_pos.1,
                    chunks_query,
                ) {
                    if !terrain.is_walkable {
                        continue;
                    }

                    let is_diagonal = dx.abs() + dy.abs() > 1;
                    let move_cost = if is_diagonal {
                        terrain.movement_cost * 1.414 // sqrt(2)
                    } else {
                        terrain.movement_cost
                    };

                    let tentative_g_cost = current_node.g_cost + move_cost;

                    let neighbor_node = nodes.entry(neighbor_pos).or_insert_with(|| {
                        PathfindingNode::new(
                            neighbor_pos,
                            f32::INFINITY,
                            Self::heuristic(neighbor_pos, goal),
                        )
                    });

                    if tentative_g_cost < neighbor_node.g_cost {
                        neighbor_node.g_cost = tentative_g_cost;
                        neighbor_node.f_cost = neighbor_node.g_cost + neighbor_node.h_cost;
                        neighbor_node.parent = Some(current_pos);

                        open_set.push(std::cmp::Reverse((
                            neighbor_node.f_cost.to_bits(),
                            neighbor_pos,
                        )));
                    }
                }
            }
        }

        None // No path found
    }

    fn heuristic(from: (i32, i32), to: (i32, i32)) -> f32 {
        let dx = (from.0 - to.0).abs() as f32;
        let dy = (from.1 - to.1).abs() as f32;
        dx + dy // Manhattan distance
    }

    fn reconstruct_path(
        nodes: &std::collections::HashMap<(i32, i32), PathfindingNode>,
        current: (i32, i32),
    ) -> Option<Vec<(i32, i32)>> {
        let mut path = Vec::new();
        let mut current_pos = current;

        while let Some(node) = nodes.get(&current_pos) {
            path.push(current_pos);
            if let Some(parent) = node.parent {
                current_pos = parent;
            } else {
                break;
            }
        }

        path.reverse();
        Some(path)
    }

    pub fn analyze_area(
        chunk_manager: &ChunkManager,
        center_x: i32,
        center_y: i32,
        radius: i32,
        chunks_query: &Query<&Chunk>,
    ) -> AreaAnalysis {
        let mut terrain_counts = std::collections::HashMap::new();
        let mut walkable_count = 0;
        let mut total_tiles = 0;
        let mut total_fertility = 0.0;
        let mut resource_potentials = std::collections::HashMap::new();

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let x = center_x + dx;
                let y = center_y + dy;

                if let Some(terrain) =
                    Self::get_terrain_at_position(chunk_manager, x, y, chunks_query)
                {
                    if let Some(terrain_type) = terrain.terrain_type {
                        *terrain_counts.entry(terrain_type).or_insert(0) += 1;
                    }

                    if terrain.is_walkable {
                        walkable_count += 1;
                    }

                    total_tiles += 1;
                    total_fertility += terrain.fertility;

                    for resource in terrain.resource_potential {
                        *resource_potentials.entry(resource).or_insert(0) += 1;
                    }
                }
            }
        }

        AreaAnalysis {
            center: (center_x, center_y),
            radius,
            total_tiles,
            walkable_tiles: walkable_count,
            walkable_percentage: if total_tiles > 0 {
                walkable_count as f32 / total_tiles as f32
            } else {
                0.0
            },
            average_fertility: if total_tiles > 0 {
                total_fertility / total_tiles as f32
            } else {
                0.0
            },
            terrain_distribution: terrain_counts,
            resource_potentials,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AreaAnalysis {
    pub center: (i32, i32),
    pub radius: i32,
    pub total_tiles: i32,
    pub walkable_tiles: i32,
    pub walkable_percentage: f32,
    pub average_fertility: f32,
    pub terrain_distribution: std::collections::HashMap<TerrainType, i32>,
    pub resource_potentials: std::collections::HashMap<String, i32>,
}

pub fn terrain_query_api_system(_chunk_manager: Res<ChunkManager>, _chunks_query: Query<&Chunk>) {
    // This system can be used to handle terrain queries from other systems
    // or external requests via WebSocket
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_component() {
        let pos1 = PositionComponent::new(10.0, 20.0);
        let pos2 = PositionComponent::new(15.0, 25.0);

        assert_eq!(pos1.distance_to(&pos2), 7.071068);

        let chunk_coord = pos1.to_chunk_coordinate(10.0);
        assert_eq!(chunk_coord.x, 0);
        assert_eq!(chunk_coord.y, 0);

        let tile_pos = pos1.to_tile_position(10.0);
        assert_eq!(tile_pos, (1, 2));
    }

    #[test]
    fn test_pathfinding_node() {
        let node = PathfindingNode::new((0, 0), 5.0, 10.0);
        assert_eq!(node.g_cost, 5.0);
        assert_eq!(node.h_cost, 10.0);
        assert_eq!(node.f_cost, 15.0);
        assert_eq!(node.position, (0, 0));
        assert!(node.parent.is_none());
    }

    #[test]
    fn test_heuristic() {
        let h1 = TerrainQuerySystem::heuristic((0, 0), (3, 4));
        assert_eq!(h1, 7.0); // Manhattan distance

        let h2 = TerrainQuerySystem::heuristic((10, 10), (7, 8));
        assert_eq!(h2, 5.0); // |10-7| + |10-8| = 3 + 2 = 5
    }

    #[test]
    fn test_path_reconstruction() {
        let mut nodes = std::collections::HashMap::new();

        nodes.insert((0, 0), PathfindingNode::new((0, 0), 0.0, 0.0));
        nodes.insert((1, 0), PathfindingNode::new((1, 0), 1.0, 0.0));
        nodes.insert((2, 0), PathfindingNode::new((2, 0), 2.0, 0.0));

        nodes.get_mut(&(1, 0)).unwrap().parent = Some((0, 0));
        nodes.get_mut(&(2, 0)).unwrap().parent = Some((1, 0));

        let path = TerrainQuerySystem::reconstruct_path(&nodes, (2, 0));
        assert_eq!(path, Some(vec![(0, 0), (1, 0), (2, 0)]));
    }
}
