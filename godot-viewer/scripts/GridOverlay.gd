# GridOverlay.gd - Draws a grid over the isometric tilemap
extends Node2D

@export var grid_color: Color = Color(1.0, 1.0, 1.0, 0.3)  # White with transparency
@export var grid_thickness: float = 1.0
@export var grid_enabled: bool = false  # Disabled by default with heightmap terrain (grid doesn't follow elevation)

# Reference to the coord_helper tilemap for coordinate conversion
var tilemap: Node2D = null
var camera: Camera2D = null

# Grid bounds (in tile coordinates)
var grid_min: Vector2i = Vector2i(-100, -100)
var grid_max: Vector2i = Vector2i(100, 100)

func _ready():
	# Set high z-index to draw on top of everything
	z_index = 100
	# Ensure GridOverlay is at origin (no position offset)
	position = Vector2(0, 0)
	print("🔲 GridOverlay initialized - Press 'G' to toggle")

func set_tilemap(p_tilemap: Node2D):
	tilemap = p_tilemap

func set_camera(p_camera: Camera2D):
	camera = p_camera

func toggle_grid():
	grid_enabled = not grid_enabled
	queue_redraw()
	print("🔲 Grid ", "enabled" if grid_enabled else "disabled")

func _draw():
	if not grid_enabled or tilemap == null:
		return

	# Get visible tile range based on camera viewport
	var visible_tiles = _get_visible_tile_range()

	# Draw horizontal and vertical grid lines for isometric tiles
	# For isometric tiles, we draw parallelograms
	for tile_x in range(visible_tiles.min_x, visible_tiles.max_x + 1):
		for tile_y in range(visible_tiles.min_y, visible_tiles.max_y + 1):
			_draw_tile_border(Vector2i(tile_x, tile_y))

func _draw_tile_border(tile_pos: Vector2i):
	# Get the isometric tile center position
	var center = tilemap.map_to_local(tile_pos)

	# OpenRCT2 tile constants (64×32 isometric diamond)
	const TILE_WIDTH = 64
	const TILE_HEIGHT = 32

	# For OpenRCT2 isometric tiles (64×32), the corners form a diamond
	# In Godot's isometric layout, map_to_local() returns the TOP corner of the diamond
	# We need to offset by half_height to get the actual center
	var half_width = TILE_WIDTH / 2.0  # 64 / 2 = 32
	var half_height = TILE_HEIGHT / 2.0  # 32 / 2 = 16

	# Adjust center: map_to_local() gives the top point, move to visual center
	var visual_center = center + Vector2(0, half_height)

	# Diamond corners: top, right, bottom, left (relative to visual center)
	var top = visual_center + Vector2(0, -half_height)
	var right = visual_center + Vector2(half_width, 0)
	var bottom = visual_center + Vector2(0, half_height)
	var left = visual_center + Vector2(-half_width, 0)

	# Draw the four edges of the diamond
	draw_line(top, right, grid_color, grid_thickness)
	draw_line(right, bottom, grid_color, grid_thickness)
	draw_line(bottom, left, grid_color, grid_thickness)
	draw_line(left, top, grid_color, grid_thickness)

func _get_visible_tile_range() -> Dictionary:
	if camera == null:
		# Default range if no camera
		return {
			"min_x": -10,
			"max_x": 10,
			"min_y": -10,
			"max_y": 10
		}

	# Get viewport size in world coordinates
	var viewport = get_viewport()
	if viewport == null:
		return {"min_x": -10, "max_x": 10, "min_y": -10, "max_y": 10}

	var viewport_size = viewport.get_visible_rect().size
	var camera_pos = camera.get_screen_center_position()
	var zoom = camera.zoom.x

	# Calculate world space bounds
	var half_width = (viewport_size.x / zoom) / 2.0
	var half_height = (viewport_size.y / zoom) / 2.0

	var top_left = camera_pos + Vector2(-half_width, -half_height)
	var bottom_right = camera_pos + Vector2(half_width, half_height)

	# Convert to tile coordinates
	var min_tile = tilemap.local_to_map(top_left)
	var max_tile = tilemap.local_to_map(bottom_right)

	# Add padding in tile space (extra tiles beyond viewport)
	var padding_tiles = 2

	# Ensure min is less than max and add padding
	var actual_min_x = min(min_tile.x, max_tile.x) - padding_tiles
	var actual_max_x = max(min_tile.x, max_tile.x) + padding_tiles
	var actual_min_y = min(min_tile.y, max_tile.y) - padding_tiles
	var actual_max_y = max(min_tile.y, max_tile.y) + padding_tiles

	# Clamp to reasonable bounds
	return {
		"min_x": clamp(actual_min_x, grid_min.x, grid_max.x),
		"max_x": clamp(actual_max_x, grid_min.x, grid_max.x),
		"min_y": clamp(actual_min_y, grid_min.y, grid_max.y),
		"max_y": clamp(actual_max_y, grid_min.y, grid_max.y)
	}

func _process(_delta):
	# Continuously redraw to update with camera movement
	if grid_enabled:
		queue_redraw()
