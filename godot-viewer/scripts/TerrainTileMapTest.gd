extends Node2D

@onready var terrain_tilemap: TileMap = $TerrainTileMap

func _ready():
	print("=== TerrainTileMap Integration Test ===")
	print()

	# Test 1: Verify TileMap initialization
	print("1. Testing TileMap initialization...")
	if terrain_tilemap != null:
		print("   ✅ TerrainTileMap node found")
		terrain_tilemap.debug_print_info()
	else:
		print("   ❌ TerrainTileMap node not found")
		return
	print()

	# Test 2: Test basic tile painting
	print("2. Testing basic tile painting...")
	test_basic_painting()
	print()

	# Test 3: Test chunk painting with mock data
	print("3. Testing chunk painting...")
	test_chunk_painting()
	print()

	# Test 4: Test terrain colors
	print("4. Testing terrain colors...")
	test_terrain_colors()
	print()

	print("=== TerrainTileMap Integration Test Complete ===")
	print("✅ Ready for chunk data integration")

# Test basic tile painting functionality
func test_basic_painting():
	# Paint a 3x3 grid around origin
	var positions = [
		Vector2i(-1, -1), Vector2i(0, -1), Vector2i(1, -1),
		Vector2i(-1, 0), Vector2i(0, 0), Vector2i(1, 0),
		Vector2i(-1, 1), Vector2i(0, 1), Vector2i(1, 1)
	]

	var terrains = ["Grass", "Forest", "Sand", "Water", "Dirt", "Snow"]

	for i in range(positions.size()):
		var pos = positions[i]
		var terrain = terrains[i % terrains.size()]
		terrain_tilemap.paint_terrain_tile(pos, terrain)
		print("   🎨 Painted ", terrain, " at ", pos)

	print("   ✅ Basic painting complete, used cells: ", terrain_tilemap.get_used_cells_count())

# Test chunk painting functionality
func test_chunk_painting():
	# Create mock chunk data
	var chunk_key = "0,0"
	var mock_terrain_data = [
		["Grass", "Grass", "Forest", "Forest"],
		["Grass", "Sand", "Sand", "Water"],
		["Dirt", "Dirt", "Snow", "Snow"],
		["Mountain", "Mountain", "Stone", "Stone"]
	]

	print("   📦 Mock chunk data created for ", chunk_key)
	terrain_tilemap.paint_chunk(chunk_key, mock_terrain_data)
	print("   ✅ Chunk painting complete, used cells: ", terrain_tilemap.get_used_cells_count())

# Test terrain color integration
func test_terrain_colors():
	var test_terrains = ["Grass", "Forest", "Sand", "Water", "Dirt", "Snow"]
	var test_pos = Vector2i(10, 10)

	for i in range(test_terrains.size()):
		var terrain = test_terrains[i]
		var pos = test_pos + Vector2i(i, 0)
		terrain_tilemap.paint_terrain_tile(pos, terrain)

		var color = Config.terrain_colors.get(terrain, Color.WHITE)
		print("   🎨 ", terrain, " -> ", color, " at ", pos)

	print("   ✅ Terrain color test complete")

func _input(event):
	# Test camera controls with arrow keys
	if event is InputEventKey:
		if event.pressed:
			var camera = $TerrainTileMap/Camera2D
			match event.keycode:
				KEY_UP:
					camera.position.y -= 100
				KEY_DOWN:
					camera.position.y += 100
				KEY_LEFT:
					camera.position.x -= 100
				KEY_RIGHT:
					camera.position.x += 100
				KEY_ESCAPE:
					get_tree().quit()