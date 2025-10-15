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
static func compute_edge_render_data(tile_pos: Vector2i, tile_corners: Dictionary, neighbor_corners: Dictionary, edge_direction: String, coord_helper) -> Dictionary:
	"""
	Calculate where and how to draw an edge face.
	
	Returns:
		{
			"position": Vector2,        # Screen position for edge sprite
			"height_pixels": float,     # Vertical size of edge in pixels
			"base_height": int,         # Lower corner height in tiny-Z
			"top_height": int,          # Upper corner height in tiny-Z
			"direction": String         # Edge direction for sprite selection
		}
	"""
	
	# OpenRCT2 constants
	const COORDS_XY_STEP = 32
	const COORDS_Z_STEP = 8
	const COORDS_Z_PER_TINY_Z = 16
	const RENDERING_SCALE = 2.0
	
	# Map edge direction to the two corners that define that edge
	const EDGE_CORNERS = {
		"north": ["top", "right"],      # North edge: top-left to top-right in isometric
		"east": ["right", "bottom"],    # East edge: top-right to bottom-right
		"south": ["bottom", "left"],    # South edge: bottom-right to bottom-left
		"west": ["left", "top"]         # West edge: bottom-left to top-left
	}
	
	const NEIGHBOR_MAP = {
		"north": {"top": "bottom", "right": "left"},
		"east": {"right": "left", "bottom": "top"},
		"south": {"bottom": "top", "left": "right"},
		"west": {"left": "right", "top": "bottom"}
	}
	
	var corners = EDGE_CORNERS[edge_direction]
	var corner_a = corners[0]
	var corner_b = corners[1]
	
	var neighbor_corner_a = NEIGHBOR_MAP[edge_direction][corner_a]
	var neighbor_corner_b = NEIGHBOR_MAP[edge_direction][corner_b]
	
	# Get heights at both corners of the edge
	var tile_height_a = tile_corners[corner_a]
	var tile_height_b = tile_corners[corner_b]
	var neighbor_height_a = neighbor_corners[neighbor_corner_a]
	var neighbor_height_b = neighbor_corners[neighbor_corner_b]
	
	# Calculate height differences
	var height_diff_a = tile_height_a - neighbor_height_a
	var height_diff_b = tile_height_b - neighbor_height_b
	
	# Average height difference for this edge
	var avg_height_diff = (height_diff_a + height_diff_b) / 2.0
	
	# Base height is the minimum of the two sides
	var base_height = min(min(tile_height_a, tile_height_b), min(neighbor_height_a, neighbor_height_b))
	var top_height = base_height + abs(avg_height_diff)
	
	# Convert to screen pixels
	var height_pixels = abs(avg_height_diff * COORDS_Z_STEP) / float(COORDS_Z_PER_TINY_Z) * RENDERING_SCALE
	
	# Calculate edge midpoint position
	# For isometric, edges are positioned between tile centers
	var edge_offset_map = {
		"north": Vector2(0, -COORDS_XY_STEP / 2),   # North edge: up from center
		"east": Vector2(COORDS_XY_STEP, 0),          # East edge: right from center
		"south": Vector2(0, COORDS_XY_STEP / 2),     # South edge: down from center
		"west": Vector2(-COORDS_XY_STEP, 0)          # West edge: left from center
	}
	
	# Base position of this tile
	var pixel_x = float(tile_pos.x - tile_pos.y) * float(COORDS_XY_STEP)
	var pixel_y = float(tile_pos.x + tile_pos.y) * float(COORDS_XY_STEP / 2)
	var base_pos = Vector2(pixel_x, pixel_y)
	
	# Offset to edge midpoint
	var edge_offset = edge_offset_map[edge_direction]
	var edge_pos = base_pos + edge_offset
	
	# Adjust Y for base height
	var base_screen_offset = float(base_height * COORDS_Z_STEP) / float(COORDS_Z_PER_TINY_Z) * RENDERING_SCALE
	edge_pos.y -= base_screen_offset
	
	return {
		"position": edge_pos,
		"height_pixels": height_pixels,
		"base_height": int(base_height),
		"top_height": int(top_height),
		"direction": edge_direction,
		"visible": height_pixels > 1.0  # Only draw if height is significant
	}

## Draw edge sprites for a tile (to be called after base terrain is painted)
func paint_edge_faces(tile_container: Node2D, tile_pos: Vector2i, tile_corners: Dictionary, neighbors: Dictionary, coord_helper = null) -> void:
	"""
	Add edge sprites for vertical faces between this tile and its neighbors.
	
	Args:
		tile_container: Parent node for edge sprites
		tile_pos: World position of this tile
		tile_corners: {top, right, bottom, left} heights for this tile
		neighbors: Dictionary of neighbor corner data keyed by direction
		coord_helper: Optional coordinate helper (unused, kept for signature compatibility)
	"""
	
	const DIRECTIONS = ["north", "east", "south", "west"]
	
	for direction in DIRECTIONS:
		# Skip if no neighbor data for this direction
		if not neighbors.has(direction) or neighbors[direction] == null:
			continue
		
		var neighbor_corners = neighbors[direction]
		
		# Check if edge is needed
		if not should_draw_edge(tile_corners, neighbor_corners, direction):
			continue
		
		# Calculate edge render data
		var edge_data = compute_edge_render_data(tile_pos, tile_corners, neighbor_corners, direction, coord_helper)
		
		if not edge_data.visible:
			continue
		
		# Create edge sprite
		var edge_sprite = Sprite2D.new()
		edge_sprite.name = "Edge_%s_%d_%d" % [direction, tile_pos.x, tile_pos.y]
		edge_sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		
		# Load or create edge texture
		var edge_texture = _load_edge_texture(edge_data.height_pixels, direction)
		if not edge_texture:
			edge_texture = _create_edge_texture(edge_data.height_pixels, direction)
		
		if edge_texture:
			edge_sprite.texture = edge_texture
			edge_sprite.position = edge_data.position
			
			# Set z_index to render edges behind terrain surface
			# Edges should appear "below" the tiles they connect
			edge_sprite.z_index = int(edge_data.position.y) - 1
			
			tile_container.add_child(edge_sprite)

## Load actual OpenRCT2 edge sprite
func _load_edge_texture(height_pixels: float, direction: String) -> Texture2D:
	"""
	Load OpenRCT2 cliff/edge sprite based on height.
	Edge sprites are organized by height increments.
	"""
	
	# Map height to edge sprite variant
	# OpenRCT2 has different sprites for different heights
	# 0-16px: edge_00, 16-32px: edge_01, 32-48px: edge_02, etc.
	var edge_index = min(int(height_pixels / 16.0), 3)  # 0-3 for now
	
	# For now, use grass edges as default
	# TODO: Add terrain-specific edge loading
	var base_path = "res://assets/tiles/edges/grass/edge_%02d.png" % edge_index
	
	if ResourceLoader.exists(base_path):
		var texture = load(base_path)
		if texture:
			return texture
	
	return null

## Create a simple colored edge texture (fallback if OpenRCT2 sprites not found)
func _create_edge_texture(height_pixels: float, direction: String) -> Texture2D:
	"""
	Create a procedural edge texture.
	For now, creates a simple colored quad. 
	TODO: Replace with extracted OpenRCT2 cliff sprites.
	"""
	
	# Edge dimensions
	var width = 64  # Width in pixels (matches tile width)
	var height = max(int(height_pixels), 4)  # Minimum 4 pixels tall
	
	# Create image
	var img = Image.create(width, height, false, Image.FORMAT_RGBA8)
	
	# Dark brown/gray color for cliff face
	var base_color = Color(0.3, 0.25, 0.2, 1.0)  # Dark brownish cliff
	
	# Fill with gradient (darker at bottom, lighter at top)
	for y in range(height):
		var brightness = 0.7 + (float(y) / float(height)) * 0.3  # 0.7 to 1.0
		var pixel_color = base_color * brightness
		
		for x in range(width):
			# Add subtle horizontal variation
			var x_var = 1.0 - abs(float(x - width/2) / float(width/2)) * 0.1
			img.set_pixel(x, y, pixel_color * x_var)
	
	return ImageTexture.create_from_image(img)

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

