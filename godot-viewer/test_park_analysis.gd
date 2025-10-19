#!/usr/bin/env godot
# Test script to analyze the park file data more thoroughly
extends MainLoop

const ParkFileParser = preload("res://scripts/ParkFileParser.gd")

var test_complete = false

func _initialize():
	print("=== Park File Analysis Test ===")
	analyze_park_file()
	return true

func _process(_delta: float) -> bool:
	return test_complete

func analyze_park_file():
	var park_file_path = "res://../good-generated-map.park"

	if not FileAccess.file_exists(park_file_path):
		print("❌ Park file not found: ", park_file_path)
		test_complete = true
		return

	print("🎢 Analyzing park file structure...")

	var parser = ParkFileParser.new()
	if not parser.parse_park_file(park_file_path):
		print("❌ Failed to parse park file: ", parser.error_message)
		test_complete = true
		return

	print("✅ Park file parsed successfully!")
	print("   Map size: ", parser.map_size.x, "x", parser.map_size.y)
	print("   Surface elements: ", parser.tile_elements.size())

	# Analyze surface elements in detail
	print("\n📊 Surface Element Analysis:")
	var height_distribution = {}
	var slope_distribution = {}
	var water_distribution = 0
	var terrain_distribution = {}

	for i in range(parser.tile_elements.size()):
		var element = parser.tile_elements[i]

		# Count heights
		var height = element.base_height
		height_distribution[height] = height_distribution.get(height, 0) + 1

		# Count slopes
		var slope = element.slope
		slope_distribution[slope] = slope_distribution.get(slope, 0) + 1

		# Count water
		if element.water_height > 0:
			water_distribution += 1

		# Count terrain types
		var terrain_type = parser.get_terrain_type_for_element(element)
		terrain_distribution[terrain_type] = terrain_distribution.get(terrain_type, 0) + 1

	print("🏔️ Height Distribution:")
	for height in height_distribution.keys():
		print("   Height ", height, ": ", height_distribution[height], " tiles")

	print("⛰️ Slope Distribution:")
	for slope in slope_distribution.keys():
		var slope_desc = parser.get_slope_description(slope)
		print("   Slope ", slope, " (", slope_desc, "): ", slope_distribution[slope], " tiles")

	print("💧 Water tiles: ", water_distribution)

	print("🌍 Terrain Distribution:")
	for terrain in terrain_distribution.keys():
		print("   ", terrain, ": ", terrain_distribution[terrain], " tiles")

	# Calculate expected vs actual
	var total_tiles = parser.map_size.x * parser.map_size.y
	var surface_tiles = parser.tile_elements.size()

	print("\n📈 Coverage Analysis:")
	print("   Total map tiles: ", total_tiles)
	print("   Surface elements found: ", surface_tiles)
	print("   Coverage: ", (float(surface_tiles) / float(total_tiles) * 100.0), "%")

	if surface_tiles < total_tiles:
		print("⚠️  This park has sparse terrain data (", surface_tiles, "/", total_tiles, " tiles have surface data)")
		print("   The remaining ", total_tiles - surface_tiles, " tiles will appear as default terrain")

	print("\n✅ Park file analysis complete!")
	test_complete = true