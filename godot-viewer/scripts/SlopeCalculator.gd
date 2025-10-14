extends RefCounted
class_name SlopeCalculator

## SlopeCalculator - Determines slope variation from height map data
## Matches OpenRCT2's 19 slope variations (0-18)

# Corner bit flags (N/E/S/W)
const CORNER_N = 0b0001  # North corner elevated
const CORNER_E = 0b0010  # East corner elevated
const CORNER_S = 0b0100  # South corner elevated
const CORNER_W = 0b1000  # West corner elevated

# Height threshold for corner elevation detection
const HEIGHT_THRESHOLD = 5  # Height difference to consider corner elevated

## Calculate slope index for a tile based on neighbor heights
## Returns slope index 0-18 (0 = flat, 1-18 = various slopes)
static func calculate_slope_index(
	heights: Array,           # 16x16 height data for current chunk
	local_pos: Vector2i,      # Position within chunk (0-15, 0-15)
	chunk_coord: Vector2i,    # Current chunk coordinates
	world_cache: Node         # WorldDataCache singleton for neighbor fetching
) -> int:
	# Get current tile height
	if local_pos.y >= heights.size() or local_pos.x >= heights[local_pos.y].size():
		return 0  # Out of bounds = flat

	var current_height = heights[local_pos.y][local_pos.x]

	# Get neighbor heights (N, E, S, W)
	var h_n = get_neighbor_height(heights, local_pos, Vector2i(0, -1), chunk_coord, world_cache)
	var h_e = get_neighbor_height(heights, local_pos, Vector2i(1, 0), chunk_coord, world_cache)
	var h_s = get_neighbor_height(heights, local_pos, Vector2i(0, 1), chunk_coord, world_cache)
	var h_w = get_neighbor_height(heights, local_pos, Vector2i(-1, 0), chunk_coord, world_cache)

	# Build slope bitfield based on which corners are elevated
	var slope = 0
	if h_n > current_height + HEIGHT_THRESHOLD:
		slope |= CORNER_N
	if h_e > current_height + HEIGHT_THRESHOLD:
		slope |= CORNER_E
	if h_s > current_height + HEIGHT_THRESHOLD:
		slope |= CORNER_S
	if h_w > current_height + HEIGHT_THRESHOLD:
		slope |= CORNER_W

	# Convert bitfield to slope index (0-18)
	return slope_to_index(slope, h_n, h_e, h_s, h_w, current_height)

## Get height of a neighboring tile, handling chunk boundaries
static func get_neighbor_height(
	heights: Array,
	local_pos: Vector2i,
	offset: Vector2i,
	chunk_coord: Vector2i,
	world_cache: Node
) -> int:
	var neighbor_pos = local_pos + offset

	# Check if neighbor is within current chunk
	if neighbor_pos.x >= 0 and neighbor_pos.x < 16 and neighbor_pos.y >= 0 and neighbor_pos.y < 16:
		return heights[neighbor_pos.y][neighbor_pos.x]

	# Neighbor is in different chunk - fetch from cache
	var world_tile_x = chunk_coord.x * 16 + neighbor_pos.x
	var world_tile_y = chunk_coord.y * 16 + neighbor_pos.y

	var neighbor_chunk_x = floori(float(world_tile_x) / 16.0)
	var neighbor_chunk_y = floori(float(world_tile_y) / 16.0)
	var neighbor_chunk_key = "%d,%d" % [neighbor_chunk_x, neighbor_chunk_y]

	var neighbor_heights = world_cache.get_height_chunk(neighbor_chunk_key)
	if neighbor_heights.size() == 0:
		return 0  # Neighbor chunk not loaded, assume flat

	# Calculate local position in neighbor chunk
	var neighbor_local_x = ((world_tile_x % 16) + 16) % 16
	var neighbor_local_y = ((world_tile_y % 16) + 16) % 16

	if neighbor_local_y < neighbor_heights.size() and neighbor_local_x < neighbor_heights[neighbor_local_y].size():
		return neighbor_heights[neighbor_local_y][neighbor_local_x]

	return 0

## Convert slope bitfield to OpenRCT2 slope index (0-18)
static func slope_to_index(slope: int, h_n: int, h_e: int, h_s: int, h_w: int, current: int) -> int:
	# Match OpenRCT2 slope indices
	match slope:
		0b0000:  # All flat
			return 0
		0b0001:  # North corner up
			return 1
		0b0010:  # East corner up
			return 2
		0b0011:  # North-East side up
			return 3
		0b0100:  # South corner up
			return 4
		0b0101:  # North-South valley
			return 5
		0b0110:  # South-East side up
			return 6
		0b0111:  # North, East, South corners up (W down)
			return 7
		0b1000:  # West corner up
			return 8
		0b1001:  # North-West side up
			return 9
		0b1010:  # East-West valley
			return 10
		0b1011:  # North, East, West corners up (S down)
			return 11
		0b1100:  # South-West side up
			return 12
		0b1101:  # North, West, South corners up (E down)
			return 13
		0b1110:  # East, South, West corners up (N down)
			return 14
		0b1111:  # All corners up
			return 15

	# Check for diagonal slopes (indices 16-17) and peak (index 18)
	# These are special cases not covered by simple bitfield

	# Diagonal NE-SW (opposite corners elevated)
	if (h_n > current and h_e > current and h_s < current and h_w < current) or \
	   (h_n < current and h_e < current and h_s > current and h_w > current):
		return 16

	# Diagonal NW-SE (opposite corners elevated)
	if (h_n > current and h_w > current and h_s < current and h_e < current) or \
	   (h_n < current and h_w < current and h_s > current and h_e > current):
		return 17

	# Peak (all neighbors lower than current)
	if h_n < current and h_e < current and h_s < current and h_w < current:
		return 18

	# Default to flat
	return 0
