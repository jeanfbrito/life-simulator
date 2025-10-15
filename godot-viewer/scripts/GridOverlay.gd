# GridOverlay.gd - Draws a slope-following grid over the isometric tilemap (OpenRCT2 style)
extends Node2D

@export var grid_color: Color = Color(1.0, 0.0, 0.0, 0.8)  # Red with high visibility
@export var grid_thickness: float = 1.0
@export var grid_enabled: bool = false

# References
var tilemap: Node2D = null
var camera: Camera2D = null
var world_data_cache: Node = null

# Grid bounds (in tile coordinates)
var grid_min: Vector2i = Vector2i(-100, -100)
var grid_max: Vector2i = Vector2i(100, 100)

# OpenRCT2 constants (match TerrainTileMap and HeightMarkerOverlay)
const COORDS_Z_STEP = 8
const COORDS_Z_PER_TINY_Z = 16
const RENDERING_SCALE = 2.0

# Tile dimensions (must match TerrainTileMap rendering scale)
const TILE_WIDTH = 64   # Base tile width
const TILE_HEIGHT = 32  # Base tile height
const RENDERED_TILE_WIDTH = TILE_WIDTH * RENDERING_SCALE   # 128 pixels
const RENDERED_TILE_HEIGHT = TILE_HEIGHT * RENDERING_SCALE  # 64 pixels

func _ready():
	# Set high z-index to draw on top of everything
	z_index = 1000  # Very high to ensure it's above all terrain
	z_as_relative = false  # Make z-index absolute, not relative to parent
	y_sort_enabled = false  # Disable Y-sorting to prevent depth issues

	# Ensure GridOverlay is at origin (no position offset)
	position = Vector2(0, 0)
	print("📐 GridOverlay initialized - draws in world space with high z-index!")

func set_tilemap(p_tilemap: Node2D):
	tilemap = p_tilemap

func set_camera(p_camera: Camera2D):
	camera = p_camera

func set_world_data_cache(cache: Node):
	world_data_cache = cache
	print("✅ GridOverlay: WorldDataCache reference set")

func toggle_grid():
	grid_enabled = not grid_enabled
	Config.show_grid = grid_enabled
	queue_redraw()  # Always redraw to clear old lines when disabling
	print("📐 Grid ", "enabled" if grid_enabled else "disabled")

func _draw():
	if not grid_enabled or tilemap == null or world_data_cache == null:
		return

	# Get visible tile range based on camera viewport
	var visible_tiles = _get_visible_tile_range()

	# Draw grid for each visible tile with height-aware corners
	for tile_x in range(visible_tiles.min_x, visible_tiles.max_x + 1):
		for tile_y in range(visible_tiles.min_y, visible_tiles.max_y + 1):
			_draw_tile_border_with_height(Vector2i(tile_x, tile_y))

func _draw_tile_border_with_height(tile_pos: Vector2i):
	# Get tile height and neighbor heights
	var tile_height = _get_tile_height(tile_pos)
	if tile_height < 0:
		return  # Tile not loaded

	var north_height = _get_tile_height(Vector2i(tile_pos.x, tile_pos.y - 1))
	var east_height = _get_tile_height(Vector2i(tile_pos.x + 1, tile_pos.y))
	var south_height = _get_tile_height(Vector2i(tile_pos.x, tile_pos.y + 1))
	var west_height = _get_tile_height(Vector2i(tile_pos.x - 1, tile_pos.y))

	# Calculate corner heights (average with neighbors for smooth slopes)
	var ne_height = tile_height
	var se_height = tile_height
	var sw_height = tile_height
	var nw_height = tile_height

	if north_height >= 0 and east_height >= 0:
		ne_height = (tile_height + north_height + east_height) / 3.0
	if south_height >= 0 and east_height >= 0:
		se_height = (tile_height + south_height + east_height) / 3.0
	if south_height >= 0 and west_height >= 0:
		sw_height = (tile_height + south_height + west_height) / 3.0
	if north_height >= 0 and west_height >= 0:
		nw_height = (tile_height + north_height + west_height) / 3.0

	# map_to_local() returns the TOP vertex of the diamond (see TooltipOverlay.gd)
	var top_vertex = tilemap.map_to_local(tile_pos)

	# Convert to diamond centre so horizontal offsets stay consistent with TerrainTileMap
	var half_width = TILE_WIDTH / 2.0
	var half_height = TILE_HEIGHT / 2.0
	var center_pixel = top_vertex + Vector2(0, half_height)

	# Base positions for each corner before height adjustment
	var ne_base = center_pixel + Vector2(0, -half_height)            # Top corner
	var se_base = center_pixel + Vector2(half_width, 0)              # Right corner
	var sw_base = center_pixel + Vector2(0, half_height)             # Bottom corner
	var nw_base = center_pixel + Vector2(-half_width, 0)             # Left corner

	# Apply height offsets to each corner
	var ne_pixel = _apply_height_offset(ne_base, ne_height)
	var se_pixel = _apply_height_offset(se_base, se_height)
	var sw_pixel = _apply_height_offset(sw_base, sw_height)
	var nw_pixel = _apply_height_offset(nw_base, nw_height)

	# Draw the four edges of the diamond with slope-following corners
	# Only draw right and bottom edges to avoid duplicates
	draw_line(ne_pixel, se_pixel, grid_color, grid_thickness)  # Right edge (NE -> SE)
	draw_line(se_pixel, sw_pixel, grid_color, grid_thickness)  # Bottom edge (SE -> SW)

func _apply_height_offset(pixel_pos: Vector2, height: float) -> Vector2:
	# Same height offset calculation as HeightMarkerOverlay and TerrainTileMap
	var base_offset = (height * COORDS_Z_STEP) / COORDS_Z_PER_TINY_Z
	var height_offset = base_offset * RENDERING_SCALE

	return Vector2(pixel_pos.x, pixel_pos.y - height_offset)

func _get_tile_height(tile_pos: Vector2i) -> float:
	# Get chunk and local position
	var chunk_x = floori(float(tile_pos.x) / 16.0)
	var chunk_y = floori(float(tile_pos.y) / 16.0)
	var chunk_key = "%d,%d" % [chunk_x, chunk_y]

	# Check if chunk is loaded
	var height_chunk = world_data_cache.get_height_chunk(chunk_key)
	if height_chunk.size() == 0:
		return -1.0  # Chunk not loaded

	# Calculate local position in chunk
	var local_x = ((tile_pos.x % 16) + 16) % 16
	var local_y = ((tile_pos.y % 16) + 16) % 16

	# Get height value
	if local_y < height_chunk.size() and local_x < height_chunk[local_y].size():
		return float(height_chunk[local_y][local_x])

	return -1.0  # Out of bounds

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
	# Always redraw to update with camera movement and clear when disabled
	queue_redraw()
