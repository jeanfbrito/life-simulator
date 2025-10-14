# TerrainTileMap.gd - Manages isometric terrain rendering
# Handles chunk data conversion to TileMap cells

extends TileMap

const SlopeCalculator = preload("res://scripts/SlopeCalculator.gd")

@export var tileset_path: String = "res://resources/TerrainTileSet.tres"

# Terrain mapping to tile IDs
var terrain_tile_ids: Dictionary = {}

# RCT2 terrain texture manager for all terrain types (grass, sand, dirt, etc.)
var rct2_terrain_manager = null

# Water texture manager for RCT2 water sprites
var water_texture_manager = null

func _ready():
	print("🗺️ TerrainTileMap initialized")

	# Initialize RCT2 terrain texture manager
	var TerrainManager = load("res://scripts/RCT2TerrainTextureManager.gd")
	rct2_terrain_manager = TerrainManager.new()
	add_child(rct2_terrain_manager)
	print("🌍 RCT2TerrainTextureManager initialized")

	# Initialize water texture manager (simple version for flat water)
	water_texture_manager = _load_water_texture()
	if water_texture_manager:
		print("🌊 Water textures loaded")

	# Set texture filtering to NEAREST for pixel art (no blurring)
	texture_filter = TEXTURE_FILTER_NEAREST
	print("🎨 Texture filter set to NEAREST (pixel-perfect rendering)")

	# Disable grid lines and debug visualizations
	navigation_visibility_mode = TileMap.VISIBILITY_MODE_FORCE_HIDE
	rendering_quadrant_size = 16  # Match chunk size for optimal rendering
	print("🔧 Grid visualization disabled")

	# Ensure we have at least one rendering layer
	if get_layers_count() == 0:
		print("⚠️ No layers configured, adding layer 0")
		add_layer(-1)  # Add a new layer at the end

	# Make sure layer 0 is enabled and visible
	set_layer_enabled(0, true)
	set_layer_modulate(0, Color(1, 1, 1, 1))  # Fully visible
	print("📋 TileMap layers count: ", get_layers_count())
	print("📋 Layer 0 enabled: ", is_layer_enabled(0))

	# Load the tileset
	load_tileset()

	# Initialize terrain mapping
	setup_terrain_mapping()

# Load the terrain tileset
func load_tileset():
	print("🔍 Attempting to load TileSet from: ", tileset_path)

	if ResourceLoader.exists(tileset_path):
		print("📁 TileSet file exists, attempting to load...")
		var loaded_tileset = ResourceLoader.load(tileset_path)
		if loaded_tileset != null:
			self.tile_set = loaded_tileset
			print("✅ TileSet loaded successfully: ", tileset_path)
			return
		else:
			print("❌ Failed to load TileSet: ", tileset_path)
	else:
		print("❌ TileSet file not found: ", tileset_path)

	# Always create a basic tileset programmatically as fallback
	print("🔧 Creating programmatically generated TileSet...")
	create_basic_tileset()

# Create a basic tileset programmatically if file loading fails
func create_basic_tileset():
	print("🎨 Creating basic TileSet...")

	var tileset = TileSet.new()
	tileset.tile_shape = 1  # ISOMETRIC
	tileset.tile_layout = 1  # STACKED
	tileset.tile_size = Vector2i(64, 32)  # RCT2 isometric tile footprint (2:1 ratio)
	print("   📐 TileSet configured: isometric, 64x32 (RCT2 2:1 isometric)")

	# Create a single white diamond texture that we'll color with materials
	var source = TileSetAtlasSource.new()
	var white_texture = create_diamond_texture()
	source.texture = white_texture
	source.texture_region_size = Vector2i(64, 32)  # RCT2 isometric tile footprint
	print("   🖼️ White diamond texture created (64x32)")

	# Create just one tile at (0,0)
	source.create_tile(Vector2i(0, 0))
	print("   🎯 Tile created at (0,0)")

	var source_id = tileset.add_source(source)
	print("   🔗 Source added with ID: ", source_id)

	# Set the tileset on this TileMap node
	self.tile_set = tileset
	print("✅ Created basic TileSet with white diamond and assigned to TileMap")

	# All terrain types will use tile (0,0) but with different materials
	var available_terrains = ["Grass", "Forest", "Sand", "Water", "Dirt", "Snow",
							 "Mountain", "Stone", "Swamp", "Desert", "DeepWater", "ShallowWater"]

	for terrain in available_terrains:
		terrain_tile_ids[terrain] = 0  # All use the same tile at (0,0)

	print("🎨 Terrain mapping configured for ", available_terrains.size(), " terrain types")

# Create a diamond texture for isometric tiles (64×32 RCT2 isometric footprint)
func create_diamond_texture() -> ImageTexture:
	var image = Image.create(64, 32, false, Image.FORMAT_RGBA8)
	image.fill(Color.TRANSPARENT)

	# Draw isometric diamond shape (64 wide × 32 tall - 2:1 ratio)
	for y in range(32):
		for x in range(64):
			# Diamond shape calculation
			var center_x = 32.0
			var center_y = 16.0
			var dx = float(abs(x - center_x))
			var dy = float(abs(y - center_y))

			# Isometric diamond: width 64, height 32 (2:1 ratio)
			if dx / 32.0 + dy / 16.0 <= 1.01:
				image.set_pixel(x, y, Color.WHITE)

	return ImageTexture.create_from_image(image)

# Create a colored diamond texture for specific terrain (64×32 RCT2 isometric footprint)
func create_colored_diamond_texture(color: Color) -> ImageTexture:
	var image = Image.create(64, 32, false, Image.FORMAT_RGBA8)
	image.fill(Color.TRANSPARENT)

	# Draw isometric diamond shape (64 wide × 32 tall - 2:1 ratio)
	for y in range(32):
		for x in range(64):
			# Diamond shape calculation
			var center_x = 32.0
			var center_y = 16.0
			var dx = float(abs(x - center_x))
			var dy = float(abs(y - center_y))

			# Isometric diamond: width 64, height 32 (2:1 ratio)
			if dx / 32.0 + dy / 16.0 <= 1.01:
				image.set_pixel(x, y, color)

	return ImageTexture.create_from_image(image)


# Get or create a TileSet source for a specific terrain type
func _get_or_create_terrain_source(terrain_type: String, color: Color) -> int:
	if not has_meta("terrain_sources"):
		set_meta("terrain_sources", {})

	var sources = get_meta("terrain_sources")

	# Return existing source ID if we already have one for this terrain type
	if sources.has(terrain_type):
		return sources[terrain_type]

	# Create new source for this terrain type
	var source = TileSetAtlasSource.new()
	source.texture = create_colored_diamond_texture(color)
	source.texture_region_size = Vector2i(64, 32)  # RCT2 isometric tile footprint
	source.create_tile(Vector2i(0, 0))

	var source_id = self.tile_set.add_source(source)
	sources[terrain_type] = source_id

	print("🔧 Created new terrain source for ", terrain_type, " with ID ", source_id)
	return source_id

# Get or create a TileSet source for a specific texture
func _get_or_create_texture_source(texture: Texture2D) -> int:
	if not texture:
		push_error("❌ Cannot create texture source: texture is null")
		return -1

	if not has_meta("texture_sources"):
		set_meta("texture_sources", {})

	var sources = get_meta("texture_sources")

	# Use texture size as key since resource_path may be empty for runtime textures
	var texture_size = texture.get_size()
	var texture_key = str(texture_size.x) + "x" + str(texture_size.y) + "_" + str(texture.get_rid().get_id())

	# Return existing source ID if we already have one for this texture
	if sources.has(texture_key):
		return sources[texture_key]

	# Validate texture size
	if texture_size.x == 0 or texture_size.y == 0:
		push_error("❌ Cannot create texture source: texture has invalid size " + str(texture_size))
		return -1

	# Create new source for this texture
	var source = TileSetAtlasSource.new()
	source.texture = texture
	# IMPORTANT: Use the actual texture size, not our desired tile size!
	source.texture_region_size = Vector2i(int(texture_size.x), int(texture_size.y))
	source.create_tile(Vector2i(0, 0))

	# Apply texture offset like stone-kingdoms does for grass tiles
	# For 1×1 grass: lOffsetY = 16 - lh + 1 (where lh is texture height)
	# For 30×18 texture on 32×16 tile: Y offset = 16 - 18 + 1 = -1
	var tile_height = int(texture_size.y)
	var offset_y = 16 - tile_height + 1  # Match stone-kingdoms offset calculation
	var offset_x = 0

	# Set texture offset on the tile data
	var tile_data = source.get_tile_data(Vector2i(0, 0), 0)
	if tile_data:
		tile_data.texture_origin = Vector2i(offset_x, offset_y)

	var source_id = self.tile_set.add_source(source)
	sources[texture_key] = source_id

	print("🔧 Created new texture source (size: %dx%d, offset: %d,%d) with ID %d" % [texture_size.x, texture_size.y, offset_x, offset_y, source_id])
	return source_id

# Setup terrain to tile ID mapping
func setup_terrain_mapping():
	# For now, we'll use the same white tile for all terrain
	# In a more complex implementation, each terrain type would have its own tile
	var available_terrains = ["Grass", "Forest", "Sand", "Water", "Dirt", "Snow",
							 "Mountain", "Stone", "Swamp", "Desert", "DeepWater", "ShallowWater"]

	for terrain in available_terrains:
		terrain_tile_ids[terrain] = 0  # All use tile ID 0 for now

	print("🎨 Terrain mapping setup for ", terrain_tile_ids.size(), " terrain types")

# Paint a chunk's terrain on the TileMap
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

			paint_terrain_tile(world_pos, terrain_type, slope_index)
			tiles_painted += 1

	print("🎨 Painted ", tiles_painted, " terrain tiles for chunk ", chunk_key)
	print("🎨 Total cells in TileMap after painting: ", get_used_cells(0).size())

	# Debug: Print first few cell positions
	if get_used_cells(0).size() > 0 and get_used_cells(0).size() <= 20:
		print("🎨 First cells: ", get_used_cells(0))

# Paint a single terrain tile (isometric)
func paint_terrain_tile(world_pos: Vector2i, terrain_type: String, slope_index: int = 0):
	# world_pos is already in tile coordinates - use it directly!
	# The isometric TileMap will handle the projection automatically

	# Check if this terrain type has an RCT2 texture
	if _should_use_rct2_texture(terrain_type) and rct2_terrain_manager and rct2_terrain_manager.has_textures():
		_paint_rct2_tile(world_pos, terrain_type, slope_index)
	else:
		# Use colored diamond for terrain without RCT2 textures
		_paint_colored_tile(world_pos, terrain_type)

func _should_use_rct2_texture(terrain_type: String) -> bool:
	"""Check if this terrain type should use RCT2 textures."""
	return terrain_type in ["Grass", "Forest", "Sand", "Desert", "Dirt", "DeepWater", "ShallowWater"]

func _is_water_terrain(terrain_type: String) -> bool:
	"""Check if this terrain type is water."""
	return terrain_type in ["DeepWater", "ShallowWater"]

func _paint_rct2_tile(world_pos: Vector2i, terrain_type: String, slope_index: int = 0):
	"""Paint a tile using RCT2 terrain or water texture with slope variation."""
	var texture: Texture2D = null

	# Check if this is water terrain
	if _is_water_terrain(terrain_type) and water_texture_manager:
		texture = water_texture_manager
	else:
		# Get the RCT2 terrain texture with slope variation
		texture = rct2_terrain_manager.get_terrain_texture(terrain_type, slope_index)

	if not texture:
		# Fallback to colored tile if texture loading failed
		_paint_colored_tile(world_pos, terrain_type)
		return

	# Get or create a source for the RCT2 texture
	var source_id = _get_or_create_texture_source(texture)

	# Set the cell with the RCT2 texture
	set_cell(0, world_pos, source_id, Vector2i(0, 0))

	# Only print for first few tiles to avoid spam
	if get_used_cells(0).size() <= 10:
		var pixel_pos = map_to_local(world_pos)
		var tile_type = "water" if _is_water_terrain(terrain_type) else "terrain"
		var slope_info = " (slope %d)" % slope_index if slope_index > 0 else ""
		print("🌊 Painted RCT2 %s tile at world %s (pixel: %s) as %s%s" % [tile_type, world_pos, pixel_pos, terrain_type, slope_info])

func _paint_colored_tile(world_pos: Vector2i, terrain_type: String):
	"""Paint a tile using colored diamond (original method)."""
	# Get terrain color from config
	var terrain_color = Config.terrain_colors.get(terrain_type, Color.WHITE)

	# Create/get a colored texture for this terrain type
	var source_id = _get_or_create_terrain_source(terrain_type, terrain_color)

	# Set the cell with the colored tile (world_pos is already in tile coords)
	set_cell(0, world_pos, source_id, Vector2i(0, 0))

	# Only print for first few tiles to avoid spam
	if get_used_cells(0).size() <= 10:
		# Get the actual pixel position of this tile in isometric space
		var pixel_pos = map_to_local(world_pos)
		print("🎨 Painted terrain tile at world ", world_pos, " (pixel: ", pixel_pos, ") as ", terrain_type, " with source ID ", source_id)

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
			# world_pos is already in tile coordinates - use directly
			erase_cell(0, world_pos)

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

# Clear all tiles from the TileMap
func clear_all_tiles():
	clear()

	# Clear all terrain sprites
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

# Debug function to print tilemap info
func debug_print_info():
	print("=== TerrainTileMap Debug Info ===")
	print("TileSet: ", "Loaded" if self.tile_set != null else "Not loaded")
	print("Tile shape: ", self.tile_set.tile_shape if self.tile_set else "N/A")
	print("Tile size: ", self.tile_set.tile_size if self.tile_set else "N/A")
	print("Used cells: ", get_used_cells(0).size())
	print("=== End Debug Info ===")