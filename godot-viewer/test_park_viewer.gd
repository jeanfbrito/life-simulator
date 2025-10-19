# Test park file loading in Godot viewer
extends MainLoop

const ParkFileParser = preload("res://scripts/ParkFileParser.gd")

var parser = null
var test_complete = false

func _initialize():
	print("=== Park File Viewer Test ===")
	test_park_file_loading_in_viewer()
	return true

func _process(_delta: float) -> bool:
	return test_complete

func test_park_file_loading_in_viewer():
	var park_file_path = "res://../good-generated-map.park"

	if not FileAccess.file_exists(park_file_path):
		print("❌ Park file not found: ", park_file_path)
		test_complete = true
		return

	print("🎢 Testing park file loading workflow...")

	# Step 1: Parse the park file
	parser = ParkFileParser.new()
	if not parser.parse_park_file(park_file_path):
		print("❌ Failed to parse park file: ", parser.error_message)
		test_complete = true
		return

	print("✅ Step 1: Park file parsed successfully")

	# Step 2: Generate terrain data
	var terrain_data = parser.generate_terrain_data()
	print("✅ Step 2: Generated terrain data for ", terrain_data.size(), " chunks")

	if terrain_data.size() == 0:
		print("⚠️ No terrain data generated")
		test_complete = true
		return

	# Step 3: Validate terrain data structure
	var first_chunk_key = terrain_data.keys()[0]
	var first_chunk = terrain_data[first_chunk_key]

	print("📊 Step 3: Validating terrain data structure...")
	print("   Chunk key: ", first_chunk_key)

	if first_chunk.has("terrain"):
		var terrain = first_chunk.terrain
		if terrain is Array:
			print("   Terrain array: ", terrain.size(), " elements")
			if terrain.size() > 0:
				var first_row = terrain[0]
				if first_row is Array:
					print("   First row: ", first_row.size(), " elements")
					print("   Sample terrain type: ", first_row[0] if first_row.size() > 0 else "N/A")

	if first_chunk.has("resources"):
		var resources = first_chunk.resources
		if resources is Array:
			print("   Resources array: ", resources.size(), " elements")

	# Step 4: Test coordinate conversion
	print("🗺️ Step 4: Testing coordinate conversion...")

	# Test a few surface elements
	var elements_to_test = min(5, parser.tile_elements.size())
	for i in range(elements_to_test):
		var element = parser.tile_elements[i]
		var tile_pos = parser.get_tile_position(i)
		var terrain_type = parser.get_terrain_type_for_element(element)
		var slope_desc = parser.get_slope_description(element.slope)

		print("   Element ", i, ": tile(", tile_pos.x, ",", tile_pos.y, ") → ", terrain_type, " (", slope_desc, ")")

	# Step 5: Summary
	print("\n📋 Summary:")
	print("   Park file: ", park_file_path)
	print("   Surface elements: ", parser.tile_elements.size())
	print("   Chunks generated: ", terrain_data.size())
	print("   Terrain types found: ", _count_terrain_types(terrain_data))

	print("\n✅ Park file loading workflow test completed successfully!")
	test_complete = true

func _count_terrain_types(terrain_data: Dictionary) -> Dictionary:
	var types = {}
	for chunk_key in terrain_data:
		var chunk = terrain_data[chunk_key]
		if chunk.has("terrain") and chunk.terrain is Array:
			for row in chunk.terrain:
				if row is Array:
					for terrain_type in row:
						if terrain_type != null and terrain_type != "":
							types[terrain_type] = types.get(terrain_type, 0) + 1
	return types