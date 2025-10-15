extends Node
class_name EdgeRenderer

## EdgeRenderer - Handles vertical edge/cliff faces between terrain tiles
## Mirrors OpenRCT2's TileSurfaceBoundaryData logic from Paint.Surface.cpp

# Constants for edge detection
const EDGE_THRESHOLD = 8  # Minimum height difference (in tiny-Z) to draw an edge

## Detect if an edge should be drawn between two tiles
## Compares corner heights at the shared edge
static func should_draw_edge(tile_corners: Dictionary, neighbor_corners: Dictionary, edge_direction: String) -> bool:
	"""
	Determine if a vertical edge face is needed.
	
	Args:
		tile_corners: {top, right, bottom, left} heights in tiny-Z
		neighbor_corners: {top, right, bottom, left} heights in tiny-Z
		edge_direction: "north", "east", "south", or "west"
	
	Returns:
		true if height difference exceeds threshold
	"""
	
	# Map edge direction to the two corners that define that edge
	const EDGE_CORNERS = {
		"north": ["top", "right"],      # North edge connects top-right corners
		"east": ["right", "bottom"],    # East edge connects right-bottom corners
		"south": ["bottom", "left"],    # South edge connects bottom-left corners
		"west": ["left", "top"]         # West edge connects left-top corners
	}
	
	if not edge_direction in EDGE_CORNERS:
		push_error("Invalid edge_direction: %s" % edge_direction)
		return false
	
	var corners = EDGE_CORNERS[edge_direction]
	var corner_a = corners[0]
	var corner_b = corners[1]
	
	# Get corresponding corners in neighbor (flipped perspective)
	const NEIGHBOR_MAP = {
		"north": {"top": "bottom", "right": "left"},
		"east": {"right": "left", "bottom": "top"},
		"south": {"bottom": "top", "left": "right"},
		"west": {"left": "right", "top": "bottom"}
	}
	
	var neighbor_corner_a = NEIGHBOR_MAP[edge_direction][corner_a]
	var neighbor_corner_b = NEIGHBOR_MAP[edge_direction][corner_b]
	
	# Check height difference at both corners
	var height_diff_a = abs(tile_corners[corner_a] - neighbor_corners[neighbor_corner_a])
	var height_diff_b = abs(tile_corners[corner_b] - neighbor_corners[neighbor_corner_b])
	
	# Draw edge if either corner has significant height difference
	return height_diff_a >= EDGE_THRESHOLD or height_diff_b >= EDGE_THRESHOLD

## Calculate edge sprite position and height
## Returns Dictionary with position, scale, and texture info
static func compute_edge_render_data(tile_pos: Vector2i, tile_corners: Dictionary, neighbor_corners: Dictionary, edge_direction: String) -> Dictionary:
	"""
	Calculate where and how to draw an edge face.
	
	Returns:
		{
			"position": Vector2,    # Screen position for edge sprite
			"height": float,        # Vertical size of edge in pixels
			"orientation": String   # "vertical" or "diagonal"
		}
	"""
	
	# TODO: Implement edge positioning math
	# Reference: OpenRCT2's viewport_surface_paint_data struct
	# - Get base position from tile_pos
	# - Calculate average corner height difference
	# - Determine sprite offset and scale
	
	return {
		"position": Vector2.ZERO,
		"height": 0.0,
		"orientation": "vertical"
	}

## Draw edge sprites for a tile (to be called after base terrain is painted)
func paint_edge_faces(tile_container: Node2D, tile_pos: Vector2i, tile_corners: Dictionary, neighbors: Dictionary) -> void:
	"""
	Add edge sprites for vertical faces between this tile and its neighbors.
	
	Args:
		tile_container: Parent node for edge sprites
		tile_pos: World position of this tile
		tile_corners: {top, right, bottom, left} heights for this tile
		neighbors: Dictionary of neighbor corner data keyed by direction
	"""
	
	# TODO: Implement edge sprite creation
	# For each direction (north, east, south, west):
	#   - Check if edge is needed via should_draw_edge()
	#   - Get render data via compute_edge_render_data()
	#   - Create Sprite2D with edge texture
	#   - Position and scale sprite appropriately
	#   - Set z_index for proper layering
	
	pass

## Placeholder for edge texture loading
## TODO: Extract vertical cliff sprites from OpenRCT2's g1.dat
static func get_edge_texture(edge_type: String, height_units: int) -> Texture2D:
	"""
	Load appropriate edge/cliff texture.
	
	Args:
		edge_type: Terrain type (e.g., "Grass", "Dirt")
		height_units: Number of height steps (determines sprite variant)
	"""
	
	# TODO: Implement texture loading from assets/tiles/edges/
	# Reference: OpenRCT2's edge sprite indices in g1.dat
	
	return null

# Notes for implementation:
# - OpenRCT2 uses separate edge sprites for different height ranges
# - Edge sprites should render AFTER base terrain but BEFORE entities
# - Z-index management is critical for proper depth sorting
# - Diagonal slopes may require special edge handling
# - Edges interact with water tiles (if water system is implemented)

