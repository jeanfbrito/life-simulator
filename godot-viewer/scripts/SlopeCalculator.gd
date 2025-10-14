class_name SlopeCalculator
extends RefCounted

## Calculates slope indices from height data (OpenRCT2 style)
##
## Slope is determined by comparing a tile's height with its 4 neighbors (N/E/S/W).
## Each raised neighbor sets a bit in the slope value (4 bits = 16+ combinations).
##
## Based on OpenRCT2's Paint.Surface.cpp slope calculation system

# Corner bitmasks
const CORNER_N = 0b0001
const CORNER_E = 0b0010
const CORNER_S = 0b0100
const CORNER_W = 0b1000

# Threshold for considering a height difference significant
const HEIGHT_THRESHOLD = 5  # Minimum height difference to create slope


## Calculate slope index for a tile based on neighbor heights
##
## @param heights: 2D array of int heights (16×16 for chunk)
## @param local_pos: Vector2i position within chunk (0-15, 0-15)
## @param chunk_coord: Vector2i chunk coordinate (for boundary checks)
## @param world_cache: WorldDataCache reference (for neighbor chunk access)
## @returns: int (0-18) slope index
static func calculate_slope_index(
	heights: Array,
	local_pos: Vector2i,
	chunk_coord: Vector2i,
	world_cache: Node
) -> int:
	var current_height = heights[local_pos.y][local_pos.x]

	# Get neighbor heights (handles chunk boundaries)
	var h_n = get_neighbor_height(heights, local_pos, Vector2i(0, -1), chunk_coord, world_cache)
	var h_e = get_neighbor_height(heights, local_pos, Vector2i(1, 0), chunk_coord, world_cache)
	var h_s = get_neighbor_height(heights, local_pos, Vector2i(0, 1), chunk_coord, world_cache)
	var h_w = get_neighbor_height(heights, local_pos, Vector2i(-1, 0), chunk_coord, world_cache)

	# Build slope bitfield (OpenRCT2 style)
	var slope = 0

	if h_n > current_height + HEIGHT_THRESHOLD:
		slope |= CORNER_N
	if h_e > current_height + HEIGHT_THRESHOLD:
		slope |= CORNER_E
	if h_s > current_height + HEIGHT_THRESHOLD:
		slope |= CORNER_S
	if h_w > current_height + HEIGHT_THRESHOLD:
		slope |= CORNER_W

	# Check for special cases (valleys, diagonals, peaks)
	return slope_to_index(slope, h_n, h_e, h_s, h_w, current_height)


## Get height of neighbor tile (handles chunk boundaries)
static func get_neighbor_height(
	heights: Array,
	local_pos: Vector2i,
	offset: Vector2i,
	chunk_coord: Vector2i,
	world_cache: Node
) -> int:
	var neighbor_pos = local_pos + offset

	# Check if neighbor is within current chunk
	if neighbor_pos.x >= 0 and neighbor_pos.x < 16 and \
	   neighbor_pos.y >= 0 and neighbor_pos.y < 16:
		return heights[neighbor_pos.y][neighbor_pos.x]

	# Neighbor is in adjacent chunk - fetch from cache
	var neighbor_chunk_coord = chunk_coord
	var neighbor_local_pos = neighbor_pos

	# Adjust chunk coordinate and local position
	if neighbor_pos.x < 0:
		neighbor_chunk_coord.x -= 1
		neighbor_local_pos.x = 15
	elif neighbor_pos.x >= 16:
		neighbor_chunk_coord.x += 1
		neighbor_local_pos.x = 0

	if neighbor_pos.y < 0:
		neighbor_chunk_coord.y -= 1
		neighbor_local_pos.y = 15
	elif neighbor_pos.y >= 16:
		neighbor_chunk_coord.y += 1
		neighbor_local_pos.y = 0

	# Get neighbor chunk from cache
	var chunk_key = "%d,%d" % [neighbor_chunk_coord.x, neighbor_chunk_coord.y]
	var neighbor_chunk = world_cache.get_chunk(chunk_key)

	if neighbor_chunk == null or not neighbor_chunk.has("heights"):
		# Neighbor chunk not loaded - assume same height (flat transition)
		return heights[local_pos.y][local_pos.x]

	return neighbor_chunk["heights"][neighbor_local_pos.y][neighbor_local_pos.x]


## Convert slope bitfield to atlas index (0-18)
static func slope_to_index(
	slope: int,
	h_n: int, h_e: int, h_s: int, h_w: int,
	current: int
) -> int:
	# Basic 4-bit slopes (0-15)
	match slope:
		0b0000: return 0   # Flat
		0b0001: return 1   # N up
		0b0010: return 2   # E up
		0b0011: return 3   # NE side up
		0b0100: return 4   # S up
		0b0101: return 5   # NS valley
		0b0110: return 6   # SE side up
		0b0111: return 7   # NES corners up
		0b1000: return 8   # W up
		0b1001: return 9   # NW side up
		0b1010: return 10  # EW valley
		0b1011: return 11  # NEW corners up
		0b1100: return 12  # SW side up
		0b1101: return 13  # NWS corners up
		0b1110: return 14  # ESW corners up
		0b1111: return 15  # All corners up (plateau)

	# Check for diagonal slopes (16-18)
	# Diagonal NE-SW: N and E high, S and W low (or vice versa)
	if (h_n > current and h_e > current and h_s < current and h_w < current) or \
	   (h_n < current and h_e < current and h_s > current and h_w > current):
		return 16

	# Diagonal NW-SE: N and W high, S and E low (or vice versa)
	if (h_n > current and h_w > current and h_s < current and h_e < current) or \
	   (h_n < current and h_w < current and h_s > current and h_e > current):
		return 17

	# Center peak: tile higher than all neighbors
	if h_n < current and h_e < current and h_s < current and h_w < current:
		return 18

	# Fallback to flat
	return 0
