# TerrainTileMap.gd - Manages isometric terrain rendering
# Handles chunk data conversion to individual Sprite2D nodes with height-based positioning

extends Node2D

const SlopeCalculator = preload("res://scripts/SlopeCalculator.gd")

# RCT2 terrain texture manager for all terrain types (grass, sand, dirt, etc.)
var rct2_terrain_manager = null

# Water texture manager for RCT2 water sprites
var water_texture_manager = null

# Tile sprite container (for Y-sorting)
var tile_container: Node2D = null

# Store all tile sprites by world position for updates/cleanup
var tile_sprites: Dictionary = {}  # Key: "x,y", Value: Sprite2D

# OpenRCT2 coordinate constants - EXACT MATCH
# From: src/openrct2/world/Location.hpp
const COORDS_XY_STEP = 32          # kCoordsXYStep - base coordinate step
const COORDS_Z_STEP = 8            # kCoordsZStep - pixels per Z level
const COORDS_Z_PER_TINY_Z = 16     # kCoordsZPerTinyZ - height division factor

# Isometric tile dimensions - OpenRCT2 EXACT MATCH
const TILE_WIDTH = 64   # 2 * COORDS_XY_STEP = 64 (diamond width)
const TILE_HEIGHT = 32  # COORDS_XY_STEP = 32 (diamond height)

# Helper TileMap for coordinate conversion only (not rendered)
var coord_helper: TileMap = null

func _ready():
	print("🗺️ TerrainTileMap initialized (Sprite2D-based rendering)")

	# Create tile container with Y-sorting enabled
	tile_container = Node2D.new()
	tile_container.name = "TileContainer"
	tile_container.y_sort_enabled = true
	add_child(tile_container)
	print("📦 Tile container created with Y-sorting enabled")

	# Create helper TileMap for coordinate conversion (not rendered, just for map_to_local)
	coord_helper = TileMap.new()
	coord_helper.tile_set = TileSet.new()
	coord_helper.tile_set.tile_shape = TileSet.TILE_SHAPE_ISOMETRIC
	coord_helper.tile_set.tile_layout = TileSet.TILE_LAYOUT_STACKED
	coord_helper.tile_set.tile_size = Vector2i(TILE_WIDTH, TILE_HEIGHT)
	coord_helper.visible = false  # Don't render it
	add_child(coord_helper)
	print("🧭 Coordinate helper TileMap created")

	# Initialize RCT2 terrain texture manager
	var TerrainManager = load("res://scripts/RCT2TerrainTextureManager.gd")
	rct2_terrain_manager = TerrainManager.new()
	add_child(rct2_terrain_manager)
	print("🌍 RCT2TerrainTextureManager initialized")

	# Initialize water texture manager (simple version for flat water)
	water_texture_manager = _load_water_texture()
	if water_texture_manager:
		print("🌊 Water textures loaded")

# Helper function to convert tile coordinates to pixel position with height
func map_to_local(tile_pos: Vector2i) -> Vector2:
	"""Convert tile coordinates to isometric pixel coordinates."""
	return coord_helper.map_to_local(tile_pos)

# Helper function for reverse conversion (if needed)
func local_to_map(pixel_pos: Vector2) -> Vector2i:
	"""Convert pixel coordinates to tile coordinates."""
	return coord_helper.local_to_map(pixel_pos)

# Paint a chunk's terrain using individual Sprite2D nodes
func paint_chunk(chunk_key: String, terrain_data: Array, height_data: Array = []):
	if terrain_data.size() == 0:
		print("⚠️ No terrain data for chunk ", chunk_key)
		return

	var chunk_origin = WorldDataCache.chunk_key_to_world_origin(chunk_key)
	var has_heights = height_data.size() > 0
	print("🎨 Painting chunk ", chunk_key, " with origin ", chunk_origin, " (heights: ", has_heights, ")")

	# Get chunk coordinates for slope calculation
	var parts = chunk_key.split(",")
	var chunk_coord = Vector2i(int(parts[0]), int(parts[1]))

	var tiles_painted = 0
	for y in range(terrain_data.size()):
		var row = terrain_data[y]
		if not row is Array:
			continue

		for x in range(row.size()):
			var terrain_type = row[x]
			if terrain_type == "":
				continue

			var world_pos = Vector2i(
				chunk_origin.x + x,
				chunk_origin.y + y
			)

			# Get height value for this tile
			var height = 0
			if has_heights and y < height_data.size() and x < height_data[y].size():
				height = int(height_data[y][x])

			# Calculate slope index if we have height data
			var slope_index = 0
			if has_heights:
				var world_cache = get_node("/root/WorldDataCache")
				slope_index = SlopeCalculator.calculate_slope_index(
					height_data,
					Vector2i(x, y),
					chunk_coord,
					world_cache
				)

			paint_terrain_tile(world_pos, terrain_type, slope_index, height)
			tiles_painted += 1

	print("🎨 Painted ", tiles_painted, " terrain tiles for chunk ", chunk_key)
	print("🎨 Total sprites: ", tile_sprites.size())

# Paint a single terrain tile as Sprite2D with height-based positioning
func paint_terrain_tile(world_pos: Vector2i, terrain_type: String, slope_index: int = 0, height: int = 0):
	# Get texture for this tile
	var texture: Texture2D = null

	if _should_use_rct2_texture(terrain_type) and rct2_terrain_manager and rct2_terrain_manager.has_textures():
		if _is_water_terrain(terrain_type) and water_texture_manager:
			texture = water_texture_manager
		else:
			texture = rct2_terrain_manager.get_terrain_texture(terrain_type, slope_index)

	if not texture:
		print("⚠️ No texture for terrain type: ", terrain_type)
		return

	# Create or get existing sprite for this position
	var tile_key = "%d,%d" % [world_pos.x, world_pos.y]
	var sprite: Sprite2D = null

	if tile_sprites.has(tile_key):
		sprite = tile_sprites[tile_key]
	else:
		sprite = Sprite2D.new()
		sprite.name = "Tile_%d_%d" % [world_pos.x, world_pos.y]
		sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST  # Pixel-perfect
		tile_container.add_child(sprite)
		tile_sprites[tile_key] = sprite

	# Set texture
	sprite.texture = texture

	# Calculate isometric position (without height yet)
	var base_pos = map_to_local(world_pos)

	# Apply OpenRCT2 height formula - EXACT MATCH
	# From: src/openrct2/paint/tile_element/Paint.Surface.cpp
	# Formula: screen_y -= (height * kCoordsZStep) / kCoordsZPerTinyZ
	var height_offset = float(height * COORDS_Z_STEP) / float(COORDS_Z_PER_TINY_Z)
	# Simplifies to: height / 2.0

	var final_pos = Vector2(base_pos.x, base_pos.y - height_offset)

	sprite.position = final_pos

	# Set Z index for Y-sorting based on final Y position
	sprite.z_index = int(final_pos.y)

	# Debug output for first few tiles
	if tile_sprites.size() <= 3:
		var slope_info = " slope=%d" % slope_index if slope_index > 0 else ""
		print("🏔️ OpenRCT2 EXACT: tile %s, height=%d → offset=%.1f px (h*%d/%d)%s → %s" %
		      [world_pos, height, height_offset, COORDS_Z_STEP, COORDS_Z_PER_TINY_Z, slope_info, terrain_type])

func _should_use_rct2_texture(terrain_type: String) -> bool:
	"""Check if this terrain type should use RCT2 textures."""
	# Enable RCT2 textures for ALL terrain types that have RCT2 mappings
	var supported_terrains = ["Grass", "Forest", "Sand", "Desert", "Dirt",
							  "Stone", "Mountain", "Snow", "Ice", "Swamp",
							  "DeepWater", "ShallowWater"]
	return terrain_type in supported_terrains

func _is_water_terrain(terrain_type: String) -> bool:
	"""Check if this terrain type is water."""
	return terrain_type in ["DeepWater", "ShallowWater"]

func _load_water_texture() -> Texture2D:
	"""Load flat RCT2 water texture."""
	var file_path = "assets/tiles/water/rct2/water_mask_00.png"
	var image = Image.new()
	var error = image.load(file_path)

	if error == OK:
		var texture = ImageTexture.create_from_image(image)
		if texture:
			print("✅ Loaded RCT2 water texture: ", file_path)
			return texture

	push_warning("🌊 Could not load water texture: " + file_path)
	return null

# Clear a chunk's tiles from the TileMap
func clear_chunk(chunk_key: String):
	var chunk_origin = WorldDataCache.chunk_key_to_world_origin(chunk_key)

	for y in range(Config.CHUNK_SIZE):
		for x in range(Config.CHUNK_SIZE):
			var world_pos = Vector2i(chunk_origin.x + x, chunk_origin.y + y)
			var tile_key = "%d,%d" % [world_pos.x, world_pos.y]

			# Remove sprite if it exists
			if tile_sprites.has(tile_key):
				var sprite = tile_sprites[tile_key]
				if sprite and is_instance_valid(sprite):
					sprite.queue_free()
				tile_sprites.erase(tile_key)

	# Clear terrain rectangles in this chunk area
	_clear_terrain_rects_in_chunk(chunk_origin)

# Clear terrain rectangles in a specific chunk area
func _clear_terrain_rects_in_chunk(chunk_origin: Vector2i):
	if not has_meta("terrain_rects"):
		return

	var rects = get_meta("terrain_rects")
	var rects_to_keep = []

	for rect in rects:
		if rect and is_instance_valid(rect):
			# Check if rectangle is within chunk bounds (simple approximation)
			# For now, we'll clear all rects since we're doing complete chunk replacement
			rect.queue_free()
		else:
			rects_to_keep.append(rect)

	set_meta("terrain_rects", rects_to_keep)

# Clear terrain sprites in a specific chunk area (legacy)
func _clear_terrain_sprites_in_chunk(chunk_origin: Vector2i):
	if not has_meta("terrain_sprites"):
		return

	var sprites = get_meta("terrain_sprites")
	var sprites_to_keep = []

	for sprite in sprites:
		if sprite and is_instance_valid(sprite):
			sprite.queue_free()

	set_meta("terrain_sprites", sprites_to_keep)

# Clear all tiles (sprites)
func clear_all_tiles():
	# Clear all tile sprites
	for tile_key in tile_sprites.keys():
		var sprite = tile_sprites[tile_key]
		if sprite and is_instance_valid(sprite):
			sprite.queue_free()
	tile_sprites.clear()

	# Clear all terrain sprites (legacy)
	if has_meta("terrain_sprites"):
		var sprites = get_meta("terrain_sprites")
		for sprite in sprites:
			if sprite and is_instance_valid(sprite):
				sprite.queue_free()
		set_meta("terrain_sprites", [])

# Get the terrain color for a tile position
func get_terrain_at_tile(tile_pos: Vector2i) -> String:
	# This would need to be implemented based on our data structure
	# For now, return default terrain
	return Config.DEFAULT_TERRAIN_TYPE

# Update multiple chunks efficiently
func update_chunks(chunk_keys: Array[String]):
	for chunk_key in chunk_keys:
		var terrain_data = WorldDataCache.get_terrain_chunk(chunk_key)
		if terrain_data.size() > 0:
			paint_chunk(chunk_key, terrain_data)

# Debug function to print terrain rendering info
func debug_print_info():
	print("=== TerrainTileMap Debug Info ===")
	print("Rendering mode: Sprite2D-based with height positioning")
	print("Total tile sprites: ", tile_sprites.size())
	print("Tile container children: ", tile_container.get_child_count() if tile_container else 0)
	print("Y-sorting enabled: ", tile_container.y_sort_enabled if tile_container else false)
	print("=== End Debug Info ===")