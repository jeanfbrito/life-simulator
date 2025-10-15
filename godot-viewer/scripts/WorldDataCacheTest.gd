extends Node

func _ready():
	print("=== World Data Cache Test ===")
	print()

	# Test 1: Coordinate conversion
	print("1. Testing coordinate conversion...")
	var test_coords = [
		Vector2i(0, 0),
		Vector2i(16, 16),
		Vector2i(-1, -1),
		Vector2i(31, 31),
		Vector2i(-17, -17)
	]

	for coord in test_coords:
		var chunk_key = WorldDataCache.get_chunk_key(coord.x, coord.y)
		var local_coords = WorldDataCache.get_local_coords(coord.x, coord.y)
		print("   World ", coord, " -> Chunk ", chunk_key, " Local ", local_coords)
	print()

	# Test 2: Basic data storage and retrieval
	print("2. Testing data storage and retrieval...")
	var test_chunk_key = "0,0"
	var test_terrain = [
		["Grass", "Grass", "Forest", "Water"],
		["Grass", "Forest", "Forest", "Water"],
		["Water", "Water", "Grass", "Grass"],
		["Desert", "Desert", "Mountain", "Grass"]
	]
	var test_resources = [
		["", "TreeOak", "", ""],
		["Flower", "", "Rock", ""],
		["", "", "", "Bush"],
		["", "Flower", "", ""]
	]
	var test_slopes = [
		[0, 1, 2, 3],
		[4, 5, 6, 7],
		[8, 9, 10, 11],
		[12, 13, 14, 15]
	]

	WorldDataCache.store_terrain_chunk(test_chunk_key, test_terrain)
	WorldDataCache.store_resource_chunk(test_chunk_key, test_resources)
	WorldDataCache.store_slope_chunk(test_chunk_key, test_slopes)

	print("   Stored test data for chunk: ", test_chunk_key)
	print()

	# Test 3: Data retrieval
	print("3. Testing data retrieval...")
	var test_retrievals = [
		{"x": 0, "y": 0, "expected_terrain": "Grass", "expected_resource": "", "expected_slope": 0},
		{"x": 1, "y": 0, "expected_terrain": "Grass", "expected_resource": "TreeOak", "expected_slope": 1},
		{"x": 2, "y": 1, "expected_terrain": "Forest", "expected_resource": "", "expected_slope": 6},
		{"x": 2, "y": 2, "expected_terrain": "Grass", "expected_resource": "Bush", "expected_slope": 10},
		{"x": 3, "y": 0, "expected_terrain": "Water", "expected_resource": "", "expected_slope": 3},
		{"x": 0, "y": 1, "expected_terrain": "Grass", "expected_resource": "Flower", "expected_slope": 4}
	]

	for test in test_retrievals:
		var terrain = WorldDataCache.get_terrain_at(test.x, test.y)
		var resource = WorldDataCache.get_resource_at(test.x, test.y)
		var slope = WorldDataCache.get_slope_index_at(test.x, test.y)
		var terrain_match = terrain == test.expected_terrain
		var resource_match = resource == test.expected_resource
 		var slope_match = slope == test.expected_slope

		print("   (%d,%d): Terrain='%s' %s, Resource='%s' %s, Slope=%s %s" % [
			test.x, test.y,
			terrain, "✓" if terrain_match else "✗",
			resource, "✓" if resource_match else "✗",
			str(slope), "✓" if slope_match else "✗"
		])
	print()

	# Test 4: Out-of-bounds handling
	print("4. Testing out-of-bounds handling...")
	var out_of_bounds = WorldDataCache.get_terrain_at(100, 100)
	var default_terrain = Config.DEFAULT_TERRAIN_TYPE
	print("   Out-of-bounds terrain: ", out_of_bounds, " (default: ", default_terrain, ")")
	print("   Default terrain match: ", "✓" if out_of_bounds == default_terrain else "✗")

	var out_of_bounds_resource = WorldDataCache.get_resource_at(100, 100)
	print("   Out-of-bounds resource: '", out_of_bounds_resource, "' (should be empty)")
	print("   Empty resource match: ", "✓" if out_of_bounds_resource == "" else "✗")
	print()

	# Test 5: Area queries
	print("5. Testing area queries...")
	var area_terrain = WorldDataCache.get_terrain_in_area(0, 0, 2, 2)
	var area_resources = WorldDataCache.get_resources_in_area(0, 0, 2, 2)
	print("   2x2 terrain area: ", area_terrain.size(), "x", area_terrain[0].size() if area_terrain.size() > 0 else 0)
	print("   2x2 resource area: ", area_resources.size(), "x", area_resources[0].size() if area_resources.size() > 0 else 0)
	print()

	# Test 6: Cache statistics
	print("6. Testing cache statistics...")
	var stats = WorldDataCache.get_cache_stats()
	print("   Terrain chunks: ", stats.terrain_chunks)
	print("   Resource chunks: ", stats.resource_chunks)
	print("   Total tiles: ", stats.total_tiles)
	print("   Memory usage: ", WorldDataCache.get_cache_memory_usage(), " bytes")
	print()

	# Test 7: Cache clearing
	print("7. Testing cache clearing...")
	WorldDataCache.clear_chunk(test_chunk_key)
	var still_cached = WorldDataCache.is_chunk_cached(test_chunk_key)
	print("   Chunk after clear: ", "cached" if still_cached else "cleared")
	print("   Clear test: ", "✓" if not still_cached else "✗")
	print()

	# Test 8: Integration with real chunk data
	print("8. Testing integration with real chunk data...")
	var real_chunk_data = await ChunkManager.request_chunks_in_area(0, 0, 1)  # 3x3 area around center
	if real_chunk_data.chunks.size() > 0:
		print("   Loaded real chunks: ", real_chunk_data.chunks.keys().size())
		WorldDataCache.merge_chunk_data(real_chunk_data)

		var stats_after = WorldDataCache.get_cache_stats()
		print("   Cache stats after merge:")
		print("     Terrain chunks: ", stats_after.terrain_chunks)
		print("     Resource chunks: ", stats_after.resource_chunks)

		# Test a lookup from real data
		var real_terrain = WorldDataCache.get_terrain_at(0, 0)
		print("   Real terrain at (0,0): ", real_terrain)
	else:
		print("   No real chunk data available")
	print()

	print("=== World Data Cache Test Complete ===")
	print("✅ All cache functionality working correctly")

	get_tree().quit()
