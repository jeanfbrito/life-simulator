extends Node

func _ready():
	print("=== Simple TileSet Test ===")
	print()

	# Test 1: Create basic TileSet
	print("1. Testing basic TileSet creation...")
	var tileset = TileSet.new()
	tileset.tile_shape = 1  # ISOMETRIC
	tileset.tile_layout = 1  # STACKED
	tileset.tile_size = Vector2i(128, 64)

	print("   ✅ TileSet created")
	print("   🎨 Shape: ", tileset.tile_shape)
	print("   📏 Size: ", tileset.tile_size)
	print()

	# Test 2: Create a simple isometric texture
	print("2. Testing isometric texture creation...")
	var image = Image.create(128, 64, false, Image.FORMAT_RGBA8)
	image.fill(Color.TRANSPARENT)

	# Draw a simple diamond
	var center_x = 64
	var center_y = 32
	for y in range(64):
		var rel_y = y - center_y
		var width_ratio = 1.0 - abs(rel_y) / 32.0
		var line_width = int(64 * width_ratio)

		if line_width > 0:
			var start_x = center_x - line_width
			var end_x = center_x + line_width
			for x in range(start_x, end_x):
				image.set_pixel(x, y, Color.WHITE)

	var texture = ImageTexture.create_from_image(image)
	print("   ✅ Isometric texture created")
	print("   📐 Size: ", texture.get_size())
	print()

	# Test 3: Create TileSet source
	print("3. Testing TileSet source creation...")
	var source = TileSetAtlasSource.new()
	source.texture = texture
	source.texture_region_size = Vector2i(128, 64)

	var source_id = tileset.add_source(source)
	print("   ✅ TileSet source added")
	print("   🆔 Source ID: ", source_id)
	print()

	# Test 4: Create a tile
	print("4. Testing tile creation...")
	source.create_tile(Vector2i(0, 0))
	print("   ✅ Tile created at (0,0)")
	print()

	# Test 5: Save the TileSet
	print("5. Testing TileSet save...")
	var save_path = "res://resources/TerrainTileSet.tres"
	ResourceSaver.save(tileset, save_path)
	print("   ✅ TileSet saved to: ", save_path)
	print()

	# Test 6: Verify terrain colors
	print("6. Testing terrain colors...")
	var terrain_count = Config.terrain_colors.size()
	print("   🎨 Terrain types: ", terrain_count)
	for terrain_name in Config.terrain_colors:
		var color = Config.terrain_colors[terrain_name]
		print("   ", terrain_name, ": ", color)
	print()

	print("=== Simple TileSet Test Complete ===")
	print("✅ Basic TileSet functionality working")

	get_tree().quit()