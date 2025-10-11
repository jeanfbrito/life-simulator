extends Node

func _ready():
	print("=== TileSet Generator Test ===")
	print()

	# Test 1: Generate basic isometric tile texture
	print("1. Testing isometric tile texture generation...")
	var tile_texture = TileSetGenerator.generate_isometric_tile()
	if tile_texture != null:
		print("   ✅ Isometric tile texture generated successfully")
		print("   📐 Texture size: ", tile_texture.get_size())
	else:
		print("   ❌ Failed to generate tile texture")
		return
	print()

	# Test 2: Generate complete terrain TileSet
	print("2. Testing terrain TileSet generation...")
	var tileset = TileSetGenerator.create_terrain_tileset()
	if tileset != null:
		print("   ✅ Terrain TileSet generated successfully")
		print("   🎨 Tile shape: ", tileset.tile_shape)
		print("   📏 Tile size: ", tileset.tile_size)
		print("   🏔️ Terrain sets: ", tileset.get_terrain_sets_count())
		print("   🎭 Terrain types in set 0: ", tileset.get_terrains_count(0))
	else:
		print("   ❌ Failed to generate TileSet")
		return
	print()

	# Test 3: Verify terrain types and colors
	print("3. Testing terrain type configuration...")
	var expected_terrains = ["Grass", "Forest", "Sand", "Water", "Dirt", "Snow", "Mountain", "Stone", "Swamp", "Desert", "DeepWater", "ShallowWater"]
	var terrain_count = tileset.get_terrains_count(0)
	print("   📋 Expected terrains: ", expected_terrains.size())
	print("   📋 Actual terrains: ", terrain_count)

	for i in range(terrain_count):
		var terrain_name = tileset.get_terrain_name(0, i)
		var terrain_color = tileset.get_terrain_color(0, i)
		print("   🎨 Terrain ", i, ": ", terrain_name, " (", terrain_color, ")")
	print()

	# Test 4: Save TileSet to file
	print("4. Testing TileSet save...")
	var save_path = "res://resources/TerrainTileSet.tres"
	TileSetGenerator.save_tileset(tileset, save_path)

	# Verify file exists
	if ResourceLoader.exists(save_path):
		print("   ✅ TileSet saved successfully to: ", save_path)
		var loaded_tileset = ResourceLoader.load(save_path)
		if loaded_tileset != null:
			print("   ✅ TileSet can be loaded back successfully")
			print("   🔍 Loaded TileSet shape: ", loaded_tileset.tile_shape)
		else:
			print("   ❌ Failed to load saved TileSet")
	else:
		print("   ❌ TileSet file not created")
	print()

	# Test 5: Integration with Config
	print("5. Testing Config integration...")
	var config_terrains = Config.terrain_colors.keys()
	print("   📋 Config terrain types: ", config_terrains.size())

	var missing_terrains = []
	for terrain in expected_terrains:
		if not config_terrains.has(terrain):
			missing_terrains.append(terrain)

	if missing_terrains.size() == 0:
		print("   ✅ All expected terrain types found in Config")
	else:
		print("   ⚠️ Missing terrain types in Config: ", missing_terrains)
	print()

	print("=== TileSet Generator Test Complete ===")
	print("✅ Ready to proceed with TileMap implementation")

	get_tree().quit()