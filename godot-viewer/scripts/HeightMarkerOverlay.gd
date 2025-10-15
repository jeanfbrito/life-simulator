# HeightMarkerOverlay.gd - Displays height numbers on tiles (OpenRCT2 style)
extends CanvasLayer
class_name HeightMarkerOverlay

# References
var terrain_tilemap: Node2D = null  # TerrainTileMap (Node2D-based, not TileMap)
var world_data_cache: Node = null
var camera: Camera2D = null

# Label pool for performance
var label_pool: Array[Label] = []
var active_labels: Array[Label] = []

# Update frequency
var update_timer: float = 0.0
var update_interval: float = 0.1  # Update every 100ms

func _ready() -> void:
	print("📏 HeightMarkerOverlay initialized")
	layer = 95  # Above terrain but below UI

func set_terrain_tilemap(tilemap: Node2D) -> void:
	terrain_tilemap = tilemap

func set_world_data_cache(cache: Node) -> void:
	world_data_cache = cache

func set_camera(cam: Camera2D) -> void:
	camera = cam

func _process(delta: float) -> void:
	if not Config.show_height_markers:
		# Hide all labels when disabled
		_clear_labels()
		return

	# Update at fixed interval to reduce performance impact
	update_timer += delta
	if update_timer >= update_interval:
		update_timer = 0.0
		_update_height_markers()

func _update_height_markers() -> void:
	if not terrain_tilemap or not world_data_cache or not camera:
		return

	# Clear previous labels
	_clear_labels()

	# Get visible tiles from camera viewport
	var visible_tiles = _get_visible_tiles()

	# Create labels for each visible tile
	for tile_pos in visible_tiles:
		var height = _get_tile_height(tile_pos)
		if height >= 0:
			_create_height_label(tile_pos, height)

func _get_visible_tiles() -> Array[Vector2i]:
	var visible_tiles: Array[Vector2i] = []

	if not camera:
		return visible_tiles

	# Get camera viewport bounds in pixel coordinates
	var viewport_size = get_viewport().get_visible_rect().size
	var camera_pos = camera.global_position
	var camera_zoom = camera.zoom

	# Calculate visible area in pixels
	var half_width = (viewport_size.x / camera_zoom.x) / 2.0
	var half_height = (viewport_size.y / camera_zoom.y) / 2.0

	var min_pixel = camera_pos - Vector2(half_width, half_height)
	var max_pixel = camera_pos + Vector2(half_width, half_height)

	# Convert pixel bounds to tile bounds (with margin for isometric rendering)
	var min_tile = terrain_tilemap.local_to_map(min_pixel) - Vector2i(2, 2)
	var max_tile = terrain_tilemap.local_to_map(max_pixel) + Vector2i(2, 2)

	# Collect all tiles in visible range
	for tile_y in range(min_tile.y, max_tile.y + 1):
		for tile_x in range(min_tile.x, max_tile.x + 1):
			visible_tiles.append(Vector2i(tile_x, tile_y))

	return visible_tiles

func _get_tile_height(tile_pos: Vector2i) -> int:
	# Get chunk and local position
	var chunk_x = floori(float(tile_pos.x) / 16.0)
	var chunk_y = floori(float(tile_pos.y) / 16.0)
	var chunk_key = "%d,%d" % [chunk_x, chunk_y]

	# Check if chunk is loaded
	var height_chunk = world_data_cache.get_height_chunk(chunk_key)
	if height_chunk.size() == 0:
		return -1  # Chunk not loaded

	# Calculate local position in chunk
	var local_x = ((tile_pos.x % 16) + 16) % 16
	var local_y = ((tile_pos.y % 16) + 16) % 16

	# Get height value
	if local_y < height_chunk.size() and local_x < height_chunk[local_y].size():
		return height_chunk[local_y][local_x]

	return -1  # Out of bounds

func _create_height_label(tile_pos: Vector2i, height: int) -> void:
	# Get or create label from pool
	var label = _get_label_from_pool()

	# Set text - display height divided by 16 (like OpenRCT2)
	# OpenRCT2 formula: displays (height + 3) / 16
	var display_value = int((height + 3) / 16)
	label.text = str(display_value)

	# Style the label BEFORE positioning (so size is calculated correctly)
	label.modulate = Color(1, 1, 1, 0.8)  # Slight transparency
	label.add_theme_color_override("font_color", Color(1, 1, 1))
	label.add_theme_color_override("font_outline_color", Color(0, 0, 0))
	label.add_theme_constant_override("outline_size", 2)
	label.add_theme_font_size_override("font_size", 12)

	# Add shadow for better visibility
	label.add_theme_color_override("font_shadow_color", Color(0, 0, 0, 0.5))
	label.add_theme_constant_override("shadow_offset_x", 1)
	label.add_theme_constant_override("shadow_offset_y", 1)

	# Force label to calculate its size
	label.reset_size()

	# Position label at tile center - MATCH TerrainTileMap.gd sprite positioning!
	# From TerrainTileMap.gd:154-187, sprites are positioned with height offset
	var base_pixel = terrain_tilemap.map_to_local(tile_pos)

	# Calculate height offset EXACTLY like TerrainTileMap does
	# From TerrainTileMap.gd:179-183
	const COORDS_Z_STEP = 8            # OpenRCT2 kCoordsZStep
	const COORDS_Z_PER_TINY_Z = 16     # OpenRCT2 kCoordsZPerTinyZ
	const RENDERING_SCALE = 2.0        # TerrainTileMap RENDERING_SCALE

	var base_offset = (float(height) * COORDS_Z_STEP) / COORDS_Z_PER_TINY_Z
	var height_offset = base_offset * RENDERING_SCALE  # Apply rendering scale!

	# Final position matches sprite position
	var tile_pixel = Vector2(base_pixel.x, base_pixel.y - height_offset)

	# Convert world position to canvas position
	# (CanvasLayer uses screen coordinates, not world coordinates)
	if camera:
		var viewport = get_viewport()
		var screen_pos = (tile_pixel - camera.global_position) * camera.zoom + viewport.get_visible_rect().size / 2.0

		# Center the label by using its actual text size
		# Estimate: ~8 pixels per character width, 14 pixels height for font size 12
		var text_width = len(label.text) * 8.0
		var text_height = 14.0
		label.position = screen_pos - Vector2(text_width / 2.0, text_height / 2.0)

	label.visible = true
	active_labels.append(label)

func _get_label_from_pool() -> Label:
	# Reuse existing label from pool
	if label_pool.size() > 0:
		return label_pool.pop_back()

	# Create new label
	var label = Label.new()
	label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	add_child(label)
	return label

func _clear_labels() -> void:
	# Return all active labels to pool
	for label in active_labels:
		label.visible = false
		label_pool.append(label)
	active_labels.clear()

func toggle_visibility() -> void:
	Config.show_height_markers = not Config.show_height_markers
	print("📏 Height markers: ", "ON" if Config.show_height_markers else "OFF")

func force_update() -> void:
	# Force immediate update
	update_timer = update_interval
