# TerrainTileMap.gd - Manages isometric terrain rendering
# Handles chunk data conversion to TileMap cells

extends TileMap

@export var tileset_path: String = "res://resources/SimpleTerrainTileSet.tres"

# Terrain mapping to tile IDs
var terrain_tile_ids: Dictionary = {}

func _ready():
	print("🗺️ TerrainTileMap initialized")

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
	tileset.tile_size = Vector2i(128, 64)
	print("   📐 TileSet configured: isometric, 128x64")

	# Create a single white diamond texture that we'll color with materials
	var source = TileSetAtlasSource.new()
	var white_texture = create_diamond_texture()
	source.texture = white_texture
	source.texture_region_size = Vector2i(128, 64)
	print("   🖼️ White diamond texture created")

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

# Create a diamond texture for isometric tiles
func create_diamond_texture() -> ImageTexture:
	var image = Image.create(128, 64, false, Image.FORMAT_RGBA8)
	image.fill(Color.TRANSPARENT)

	# Draw filled diamond
	var center_x = 64
	var center_y = 32
	var half_width = 64
	var half_height = 32

	for y in range(64):
		var rel_y = y - center_y
		var width_ratio = 1.0 - abs(rel_y) / float(half_height)
		var line_width = int(half_width * width_ratio)

		if line_width > 0:
			var start_x = center_x - line_width
			var end_x = center_x + line_width
			for x in range(start_x, end_x):
				image.set_pixel(x, y, Color.WHITE)

	return ImageTexture.create_from_image(image)

# Create a colored diamond texture for specific terrain
func create_colored_diamond_texture(color: Color) -> ImageTexture:
	var image = Image.create(128, 64, false, Image.FORMAT_RGBA8)
	image.fill(Color.TRANSPARENT)

	# Draw filled diamond with specified color
	var center_x = 64
	var center_y = 32
	var half_width = 64
	var half_height = 32

	for y in range(64):
		var rel_y = y - center_y
		var width_ratio = 1.0 - abs(rel_y) / float(half_height)
		var line_width = int(half_width * width_ratio)

		if line_width > 0:
			var start_x = center_x - line_width
			var end_x = center_x + line_width
			for x in range(start_x, end_x):
				image.set_pixel(x, y, color)

	return ImageTexture.create_from_image(image)

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
		return

	var chunk_origin = WorldDataCache.chunk_key_to_world_origin(chunk_key)

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

# Paint a single terrain tile
func paint_terrain_tile(world_pos: Vector2i, terrain_type: String):
	# Convert world coordinates to tilemap coordinates
	var tile_pos = local_to_map(Vector2(world_pos))

	# Get terrain color from config
	var terrain_color = Config.terrain_colors.get(terrain_type, Color.WHITE)

	# Set the cell with the base tile
	set_cell(0, tile_pos, 0, Vector2i(0, 0))

	# Apply terrain color modulation using TileMap's built-in features
	# In Godot 4.5, we can use the set_cells_terrain_connect method for terrain
	# But for now, let's use a custom approach with individual tile modulation

	# Create a child Sprite2D for colored terrain overlay
	var terrain_sprite = Sprite2D.new()
	terrain_sprite.texture = create_colored_diamond_texture(terrain_color)
	terrain_sprite.position = map_to_local(tile_pos)
	terrain_sprite.centered = false
	add_child(terrain_sprite)

	# Store reference for cleanup
	if not has_meta("terrain_sprites"):
		set_meta("terrain_sprites", [])
	var sprites = get_meta("terrain_sprites")
	sprites.append(terrain_sprite)

# Clear a chunk's tiles from the TileMap
func clear_chunk(chunk_key: String):
	var chunk_origin = WorldDataCache.chunk_key_to_world_origin(chunk_key)

	for y in range(Config.CHUNK_SIZE):
		for x in range(Config.CHUNK_SIZE):
			var world_pos = Vector2i(chunk_origin.x + x, chunk_origin.y + y)
			var tile_pos = local_to_map(Vector2(world_pos))
			erase_cell(0, tile_pos)

	# Also clear terrain sprites in this chunk area
	_clear_terrain_sprites_in_chunk(chunk_origin)

# Clear terrain sprites in a specific chunk area
func _clear_terrain_sprites_in_chunk(chunk_origin: Vector2i):
	if not has_meta("terrain_sprites"):
		return

	var sprites = get_meta("terrain_sprites")
	var sprites_to_keep = []

	for sprite in sprites:
		if sprite and is_instance_valid(sprite):
			# Check if sprite is within chunk bounds
			var sprite_world_pos = local_to_map(sprite.position)
			var sprite_chunk_x = floor(sprite_world_pos.x / float(Config.CHUNK_SIZE))
			var sprite_chunk_y = floor(sprite_world_pos.y / float(Config.CHUNK_SIZE))

			if sprite_chunk_x != chunk_origin.x / Config.CHUNK_SIZE or sprite_chunk_y != chunk_origin.y / Config.CHUNK_SIZE:
				sprites_to_keep.append(sprite)
			else:
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