# Test script for ParkFileParser
# Run this in Godot to test park file parsing

extends MainLoop

# Load the ParkFileParser class
const ParkFileParser = preload("res://scripts/ParkFileParser.gd")

func _initialize():
	print("=== Park File Parser Test ===")
	test_park_file_parsing()
	return true

func _process(_delta: float) -> bool:
	return true  # Exit immediately

func test_park_file_parsing():
	var park_file_path = "res://../good-generated-map.park"

	# Check if file exists
	if not FileAccess.file_exists(park_file_path):
		print("❌ Park file not found: ", park_file_path)
		print("Looking in current directory:")
		var dir = DirAccess.open("res://")
		if dir:
			dir.list_dir_begin()
			var file_name = dir.get_next()
			while file_name != "":
				print("  ", file_name)
				file_name = dir.get_next()
		return

	print("📁 Found park file: ", park_file_path)

	# Create parser
	var parser = ParkFileParser.new()

	# Parse the file
	if parser.parse_park_file(park_file_path):
		print("✅ Park file parsed successfully!")
		print(parser.get_debug_info())

		# Test terrain data generation
		var terrain_data = parser.generate_terrain_data()
		print("🗺️ Generated terrain data for ", terrain_data.size(), " chunks")

		# Show sample chunk data
		if terrain_data.size() > 0:
			var first_chunk_key = terrain_data.keys()[0]
			var first_chunk = terrain_data[first_chunk_key]
			print("📦 Sample chunk: ", first_chunk_key)
			if first_chunk.has("terrain"):
				var terrain = first_chunk.terrain
				if terrain is Array and terrain.size() > 0:
					print("   Terrain size: ", terrain.size(), "x", terrain[0].size() if terrain[0] is Array else "?")
			if first_chunk.has("resources"):
				var resources = first_chunk.resources
				if resources is Array and resources.size() > 0:
					print("   Resources size: ", resources.size(), "x", resources[0].size() if resources[0] is Array else "?")
	else:
		print("❌ Failed to parse park file: ", parser.error_message)

	print("=== Test Complete ===")