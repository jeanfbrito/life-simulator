# TileSetGenerator.gd - Generates isometric tile textures and TileSet
# Creates colored diamond shapes for terrain rendering

extends Node

# Generate a simple white diamond texture for isometric tiles
static func generate_isometric_tile(size: Vector2i = Vector2i(128, 64)) -> ImageTexture:
	var image = Image.create(size.x, size.y, false, Image.FORMAT_RGBA8)
	image.fill(Color.TRANSPARENT)

	# Calculate diamond vertices
	var center_x = size.x / 2
	var center_y = size.y / 2
	var half_width = size.x / 2
	var half_height = size.y / 2

	# Draw filled diamond using scanline algorithm
	for y in range(size.y):
		# Calculate width of diamond at this y coordinate
		var rel_y = y - center_y
		var width_ratio = 1.0 - abs(rel_y) / float(half_height)
		var line_width = int(half_width * width_ratio)

		if line_width > 0:
			var start_x = center_x - line_width
			var end_x = center_x + line_width

			for x in range(start_x, end_x):
				image.set_pixel(x, y, Color.WHITE)

	var texture = ImageTexture.create_from_image(image)
	return texture

# Create terrain TileSet with colored materials
static func create_terrain_tileset() -> TileSet:
	var tileset = TileSet.new()

	# Set isometric properties (using correct enum values for Godot 4.5)
	tileset.tile_shape = 1  # TileSet.TileShape.ISOMETRIC
	tileset.tile_layout = 1  # TileSet.TileLayout.STACKED
	tileset.tile_size = Vector2i(128, 64)
	tileset.uv_clipping = false

	# Generate base tile texture
	var tile_texture = generate_isometric_tile()

	# Create source 0 with all terrain alternatives
	var source_id = 0
	var source = TileSetAtlasSource.new()
	tileset.add_source(source, source_id)

	# Set texture coordinates
	source.texture = tile_texture
	source.texture_region_size = Vector2i(128, 64)
	source.use_texture_padding = false

	# Get terrain colors from Config
	var terrain_colors = Config.terrain_colors

	# Create terrain set with all terrain types
	var terrain_set = 0
	tileset.add_terrain_set(terrain_set)

	# Add each terrain type to the terrain set
	var terrain_types = {
		"Grass": 0,
		"Forest": 1,
		"Sand": 2,
		"Water": 3,
		"Dirt": 4,
		"Snow": 5,
		"Mountain": 6,
		"Stone": 7,
		"Swamp": 8,
		"Desert": 9,
		"DeepWater": 10,
		"ShallowWater": 11
	}

	for terrain_name in terrain_types:
		var terrain_index = terrain_types[terrain_name]
		var color = terrain_colors.get(terrain_name, Color.WHITE)

		# Add terrain to set
		tileset.add_terrain(terrain_set, terrain_index)
		tileset.set_terrain_name(terrain_set, terrain_index, terrain_name)
		tileset.set_terrain_color(terrain_set, terrain_index, color)

	# Set up tile at coords (0,0) with terrain peering
	source.create_tile(Vector2i(0, 0))

	# Set all corners to be paintable with terrain (using simplified neighbor constants)
	for direction in range(8):
		source.set_terrain_peering_bit(Vector2i(0, 0), direction, -1)

	# Add custom data layers for terrain and resource types
	var terrain_data_layer = tileset.add_custom_data_layer(0)
	tileset.set_custom_data_layer_name(terrain_data_layer, "terrain_type")

	var resource_data_layer = tileset.add_custom_data_layer(1)
	tileset.set_custom_data_layer_name(resource_data_layer, "resource_type")

	return tileset

# Save the generated TileSet to a file
static func save_tileset(tileset: TileSet, path: String) -> void:
	ResourceSaver.save(tileset, path)
	print("💾 TileSet saved to: ", path)

# Generate and save the complete terrain TileSet
static func generate_and_save_terrain_tileset() -> TileSet:
	print("🎨 Generating isometric terrain TileSet...")

	var tileset = create_terrain_tileset()
	var save_path = "res://resources/TerrainTileSet.tres"
	save_tileset(tileset, save_path)

	print("✅ Terrain TileSet generation complete")
	return tileset