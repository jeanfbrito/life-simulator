# TerrainTileMap.gd - Manages isometric terrain rendering
# Handles chunk data conversion to TileMap cells

extends TileMap

@export var tileset_path: String = "res://resources/TerrainTileSet.tres"

# Terrain mapping to tile IDs
var terrain_tile_ids: Dictionary = {}

# Grass texture manager for RCT2 grass textures
var grass_manager = null

func _ready():
	print("🗺️ TerrainTileMap initialized")

	# Initialize RCT2 grass texture manager
	var GrassManager = load("res://scripts/GrassTextureManagerRCT2.gd")
	grass_manager = GrassManager.new()
	add_child(grass_manager)
	print("🌿 RCT2 GrassTextureManager initialized")

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
	tileset.tile_size = Vector2i(64, 64)  # Match RCT2 atlas cell size
	print("   📐 TileSet configured: isometric, 64x64 (RCT2 atlas cell size)")

	# Create a single white diamond texture that we'll color with materials
	var source = TileSetAtlasSource.new()
	var white_texture = create_diamond_texture()
	source.texture = white_texture
	source.texture_region_size = Vector2i(64, 64)  # Match RCT2 atlas cell size
	print("   🖼️ White diamond texture created (64x64)")

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

# Create a diamond texture for isometric tiles (64×64 to match RCT2 atlas cells)
func create_diamond_texture() -> ImageTexture:
	var image = Image.create(64, 64, false, Image.FORMAT_RGBA8)
	image.fill(Color.TRANSPARENT)

	# Draw isometric diamond shape in bottom half of 64x64 cell
	# RCT2 sprites are bottom-aligned in 64x64 cells
	# Diamond is ~64 wide × 32 tall, positioned at bottom
	var y_offset = 32  # Start diamond at vertical center

	for y in range(32):  # Diamond height
		for x in range(64):  # Diamond width
			# Diamond shape calculation
			var center_x = 32.0
			var center_y = 16.0
			var dx = float(abs(x - center_x))
			var dy = float(abs(y - center_y))

			# Isometric diamond: width 64, height 32
			if dx / 32.0 + dy / 16.0 <= 1.01:
				image.set_pixel(x, y + y_offset, Color.WHITE)

	return ImageTexture.create_from_image(image)

# Create a colored diamond texture for specific terrain (64×64 to match RCT2 atlas cells)
func create_colored_diamond_texture(color: Color) -> ImageTexture:
	var image = Image.create(64, 64, false, Image.FORMAT_RGBA8)
	image.fill(Color.TRANSPARENT)

	# Draw isometric diamond shape in bottom half of 64x64 cell
	# RCT2 sprites are bottom-aligned in 64x64 cells
	var y_offset = 32  # Start diamond at vertical center

	for y in range(32):  # Diamond height
		for x in range(64):  # Diamond width
			# Diamond shape calculation
			var center_x = 32.0
			var center_y = 16.0
			var dx = float(abs(x - center_x))
			var dy = float(abs(y - center_y))

			# Isometric diamond: width 64, height 32
			if dx / 32.0 + dy / 16.0 <= 1.01:
				image.set_pixel(x, y + y_offset, color)

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
	source.texture_region_size = Vector2i(64, 64)  # Match RCT2 atlas cell size
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
func paint_chunk(chunk_key: String, terrain_data: Array):
	if terrain_data.size() == 0:
		print("⚠️ No terrain data for chunk ", chunk_key)
		return

	var chunk_origin = WorldDataCache.chunk_key_to_world_origin(chunk_key)
	print("🎨 Painting chunk ", chunk_key, " with origin ", chunk_origin, " and ", terrain_data.size(), " rows")

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

			paint_terrain_tile(world_pos, terrain_type)
			tiles_painted += 1

	print("🎨 Painted ", tiles_painted, " terrain tiles for chunk ", chunk_key)
	print("🎨 Total cells in TileMap after painting: ", get_used_cells(0).size())

	# Debug: Print first few cell positions
	if get_used_cells(0).size() > 0 and get_used_cells(0).size() <= 20:
		print("🎨 First cells: ", get_used_cells(0))

# Paint a single terrain tile (isometric)
func paint_terrain_tile(world_pos: Vector2i, terrain_type: String):
	# world_pos is already in tile coordinates - use it directly!
	# The isometric TileMap will handle the projection automatically

	# Check if this is a grass terrain and we have grass textures loaded
	if _should_use_grass_texture(terrain_type) and grass_manager and grass_manager.has_textures():
		_paint_grass_tile(world_pos, terrain_type)
	else:
		# Use colored diamond for non-grass terrain
		_paint_colored_tile(world_pos, terrain_type)

func _should_use_grass_texture(terrain_type: String) -> bool:
	"""Check if this terrain type should use grass textures."""
	return terrain_type in ["Grass", "Forest"]

func _paint_grass_tile(world_pos: Vector2i, terrain_type: String):
	"""Paint a tile using RCT2 grass texture."""
	# Get the flat RCT2 grass texture
	var grass_texture = grass_manager.get_grass_texture()
	if not grass_texture:
		# Fallback to colored tile if texture loading failed
		_paint_colored_tile(world_pos, terrain_type)
		return

	# Get or create a source for the RCT2 grass texture
	var source_id = _get_or_create_texture_source(grass_texture)

	# Set the cell with the grass texture
	set_cell(0, world_pos, source_id, Vector2i(0, 0))

	# Only print for first few tiles to avoid spam
	if get_used_cells(0).size() <= 10:
		var pixel_pos = map_to_local(world_pos)
		print("🌿 Painted RCT2 grass tile at world ", world_pos, " (pixel: ", pixel_pos, ") as ", terrain_type)

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