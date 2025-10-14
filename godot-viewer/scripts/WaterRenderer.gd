extends Node2D
class_name WaterRenderer

## RCT2-style Water Renderer
## Follows OpenRCT2's water rendering system:
## - Separate water_height from terrain_height
## - Water surface follows terrain slope
## - Two-layer rendering (mask + overlay)
##
## Water system based on OpenRCT2's Paint.Surface.cpp lines 1230-1268

# Water sprite mapping (OpenRCT2's Byte97B740 lookup table)
const WATER_SPRITE_LOOKUP = [
	0, 0, 0, 0, 0, 0, 0, 2,  # Slopes 0-7
	0, 0, 0, 3, 0, 1, 4, 0,  # Slopes 8-15
]

# Loaded water textures
var water_mask_textures: Array = []  # 5 base water sprites
var water_overlay_textures: Array = []  # 5 overlay sprites (animated)
var textures_loaded: bool = false

# Animation
var overlay_frame: int = 0
var animation_timer: float = 0.0
const ANIMATION_SPEED: float = 0.2  # Seconds per frame

# References
@onready var terrain_tilemap: TileMap = get_parent()

func _ready():
	print("🌊 WaterRenderer initialized")
	load_water_textures()

func _process(delta):
	# Animate water overlay
	if textures_loaded:
		animation_timer += delta
		if animation_timer >= ANIMATION_SPEED:
			animation_timer = 0.0
			overlay_frame = (overlay_frame + 1) % 5
			# Note: In full implementation, this would update overlay sprites

func load_water_textures():
	"""Load RCT2-style water sprites."""
	print("🌊 Loading water textures...")

	var base_path = "assets/tiles/water/rct2"

	# Load 5 water mask sprites
	for i in range(5):
		var file_path = base_path + "/water_mask_%02d.png" % i
		var image = Image.new()
		var error = image.load(file_path)

		if error == OK:
			var texture = ImageTexture.create_from_image(image)
			if texture:
				water_mask_textures.append(texture)
		else:
			push_warning("🌊 Could not load water mask: " + file_path)

	# Load 5 water overlay sprites
	for i in range(5):
		var file_path = base_path + "/water_overlay_%02d.png" % i
		var image = Image.new()
		var error = image.load(file_path)

		if error == OK:
			var texture = ImageTexture.create_from_image(image)
			if texture:
				water_overlay_textures.append(texture)
		else:
			push_warning("🌊 Could not load water overlay: " + file_path)

	if water_mask_textures.size() == 5:
		textures_loaded = true
		print("✅ Loaded 5 water mask textures and %d overlays" % water_overlay_textures.size())
	else:
		push_error("❌ Failed to load water textures")
		textures_loaded = false

func render_water_for_chunk(chunk_key: String, chunk_data: Dictionary):
	"""Render water tiles for a chunk.

	Args:
		chunk_key: Chunk coordinate key (e.g., "0,0")
		chunk_data: Dictionary containing terrain, heights, and water_heights layers
	"""
	if not textures_loaded:
		return

	if not chunk_data.has("water_heights"):
		return  # No water data for this chunk

	if not chunk_data.has("heights"):
		push_warning("🌊 Chunk %s has water_heights but no terrain heights" % chunk_key)
		return

	var water_heights = chunk_data["water_heights"]
	var terrain_heights = chunk_data["heights"]
	var chunk_origin = WorldDataCache.chunk_key_to_world_origin(chunk_key)

	var water_tiles_rendered = 0

	# Render water for each tile in chunk
	for local_y in range(16):
		for local_x in range(16):
			var water_height = water_heights[local_y][local_x]
			var terrain_height = terrain_heights[local_y][local_x]

			# Skip if no water or water below terrain
			if water_height == 0 or water_height <= terrain_height:
				continue

			# Calculate world tile position
			var world_tile_x = chunk_origin.x + local_x
			var world_tile_y = chunk_origin.y + local_y
			var tile_pos = Vector2i(world_tile_x, world_tile_y)

			# Render water tile
			render_water_tile(tile_pos, water_height, terrain_height, chunk_data)
			water_tiles_rendered += 1

	if water_tiles_rendered > 0:
		print("🌊 Rendered %d water tiles for chunk %s" % [water_tiles_rendered, chunk_key])

func render_water_tile(
	tile_pos: Vector2i,
	water_height: int,
	terrain_height: int,
	chunk_data: Dictionary
):
	"""Render a single water tile following OpenRCT2's system.

	Water rendering follows OpenRCT2's Paint.Surface.cpp:
	1. Determine terrain slope at this position
	2. Map slope to water sprite index via Byte97B740 lookup
	3. Render water mask at water_height
	4. Render animated overlay on top

	Args:
		tile_pos: World tile position (x, y)
		water_height: Water height level (0-255)
		terrain_height: Terrain height level (0-255)
		chunk_data: Full chunk data for neighbor access
	"""
	# Calculate terrain slope index (determines water surface shape)
	var slope_index = _calculate_terrain_slope(tile_pos, chunk_data)

	# Map slope to water sprite index (OpenRCT2's Byte97B740)
	var water_sprite_index = WATER_SPRITE_LOOKUP[slope_index & 0xF]

	# Get water mask texture
	if water_sprite_index >= water_mask_textures.size():
		water_sprite_index = 0  # Fallback to flat water

	var water_texture = water_mask_textures[water_sprite_index]

	# Get or create texture source for this water sprite
	var source_id = _get_or_create_water_source(water_texture)

	# Convert tile position to pixel position for water sprite placement
	var pixel_pos = terrain_tilemap.map_to_local(tile_pos)

	# Create water sprite (using Sprite2D for now, can be optimized to TileMap later)
	var water_sprite = Sprite2D.new()
	water_sprite.texture = water_texture
	water_sprite.position = pixel_pos
	water_sprite.z_index = 10  # Above terrain
	water_sprite.centered = true

	# Store sprite for cleanup
	if not has_meta("water_sprites"):
		set_meta("water_sprites", [])
	var sprites = get_meta("water_sprites")
	sprites.append(water_sprite)

	add_child(water_sprite)

	# Add overlay (optional, for animation)
	if water_overlay_textures.size() > 0:
		var overlay_sprite = Sprite2D.new()
		overlay_sprite.texture = water_overlay_textures[overlay_frame]
		overlay_sprite.modulate = Color(1, 1, 1, 0.5)  # Semi-transparent
		overlay_sprite.position = Vector2(0, 0)  # Relative to parent
		overlay_sprite.z_index = 1  # Above water mask
		overlay_sprite.centered = true
		water_sprite.add_child(overlay_sprite)

func _calculate_terrain_slope(tile_pos: Vector2i, chunk_data: Dictionary) -> int:
	"""Calculate terrain slope index at tile position.

	This determines the shape of the water surface.
	For now, returns 0 (flat) since we don't have height variation yet.

	TODO: Implement full slope calculation when height system is added.
	"""
	# For now, return flat slope
	# In full implementation, this would use SlopeCalculator.gd
	return 0

func _get_or_create_water_source(texture: Texture2D) -> int:
	"""Get or create TileMap source ID for water texture."""
	if not has_meta("water_sources"):
		set_meta("water_sources", {})

	var sources = get_meta("water_sources")
	var texture_key = str(texture.get_rid().get_id())

	if sources.has(texture_key):
		return sources[texture_key]

	# Create new source (placeholder for future TileMap integration)
	var source_id = sources.size()
	sources[texture_key] = source_id

	return source_id

func clear_water_for_chunk(chunk_key: String):
	"""Clear all water sprites for a chunk."""
	if not has_meta("water_sprites"):
		return

	var sprites = get_meta("water_sprites")
	for sprite in sprites:
		if sprite and is_instance_valid(sprite):
			sprite.queue_free()

	set_meta("water_sprites", [])

func has_textures() -> bool:
	"""Check if water textures are loaded."""
	return textures_loaded

func get_texture_count() -> int:
	"""Get number of loaded water textures."""
	return water_mask_textures.size() + water_overlay_textures.size()
