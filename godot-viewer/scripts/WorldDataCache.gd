# WorldDataCache.gd - Manages world data caching and coordinate translation
# Provides efficient lookup for terrain/resource data by world coordinates

extends Node

signal cache_updated(chunk_key: String)
signal cache_cleared()

# Cached world data storage
var terrain_cache: Dictionary = {}  # chunk_key -> 2D array of terrain strings
var resource_cache: Dictionary = {}  # chunk_key -> 2D array of resource strings
var cache_stats: Dictionary = {
	"terrain_chunks": 0,
	"resource_chunks": 0,
	"total_tiles": 0
}

# Constants for coordinate translation
const CHUNK_SIZE: int = 16

# Called when the node enters the scene tree for the first time.
func _ready():
	print("WorldDataCache initialized")

# Store terrain data for a chunk
func store_terrain_chunk(chunk_key: String, terrain_data: Array):
	if terrain_data is Array and terrain_data.size() > 0:
		terrain_cache[chunk_key] = terrain_data
		update_stats()
		print("🗂️ Stored terrain chunk: ", chunk_key, " (", terrain_data.size(), "x", terrain_data[0].size() if terrain_data[0] is Array else "?", ")")
		cache_updated.emit(chunk_key)

# Store resource data for a chunk
func store_resource_chunk(chunk_key: String, resource_data: Array):
	if resource_data is Array and resource_data.size() > 0:
		resource_cache[chunk_key] = resource_data
		update_stats()
		print("🗂️ Stored resource chunk: ", chunk_key, " (", resource_data.size(), "x", resource_data[0].size() if resource_data[0] is Array else "?", ")")
		cache_updated.emit(chunk_key)

# Store both terrain and resource data for a chunk
func store_chunk_data(chunk_key: String, chunk_data: Dictionary):
	if chunk_data.has("terrain"):
		store_terrain_chunk(chunk_key, chunk_data.terrain)
	if chunk_data.has("resources"):
		store_resource_chunk(chunk_key, chunk_data.resources)

# Merge chunk data from ChunkManager response
func merge_chunk_data(chunk_data_response: Dictionary):
	if chunk_data_response.has("chunks"):
		for chunk_key in chunk_data_response.chunks:
			store_terrain_chunk(chunk_key, chunk_data_response.chunks[chunk_key])

	if chunk_data_response.has("resources"):
		for chunk_key in chunk_data_response.resources:
			store_resource_chunk(chunk_key, chunk_data_response.resources[chunk_key])

# Get terrain type at world coordinates
func get_terrain_at(world_x: int, world_y: int) -> String:
	var chunk_key = get_chunk_key(world_x, world_y)
	var local_coords = get_local_coords(world_x, world_y)

	if terrain_cache.has(chunk_key):
		var chunk = terrain_cache[chunk_key]
		if local_coords.y < chunk.size() and local_coords.x < chunk[local_coords.y].size():
			return chunk[local_coords.y][local_coords.x]

	# Return default terrain if not found
	return Config.DEFAULT_TERRAIN_TYPE

# Get resource type at world coordinates
func get_resource_at(world_x: int, world_y: int) -> String:
	var chunk_key = get_chunk_key(world_x, world_y)
	var local_coords = get_local_coords(world_x, world_y)

	if resource_cache.has(chunk_key):
		var chunk = resource_cache[chunk_key]
		if local_coords.y < chunk.size() and local_coords.x < chunk[local_coords.y].size():
			return chunk[local_coords.y][local_coords.x]

	# Return empty string if no resource found
	return ""

# Check if a chunk is cached
func is_chunk_cached(chunk_key: String) -> bool:
	return terrain_cache.has(chunk_key) or resource_cache.has(chunk_key)

# Check if terrain is cached for a chunk
func is_terrain_cached(chunk_key: String) -> bool:
	return terrain_cache.has(chunk_key)

# Check if resources are cached for a chunk
func are_resources_cached(chunk_key: String) -> bool:
	return resource_cache.has(chunk_key)

# Get all cached chunk keys
func get_cached_chunk_keys() -> Array[String]:
	var all_keys: Array[String] = []

	# Combine keys from both caches
	for key in terrain_cache.keys():
		if not all_keys.has(key):
			all_keys.append(key)

	for key in resource_cache.keys():
		if not all_keys.has(key):
			all_keys.append(key)

	return all_keys

# Get cached terrain chunk data
func get_terrain_chunk(chunk_key: String) -> Array:
	return terrain_cache.get(chunk_key, [])

# Get cached resource chunk data
func get_resource_chunk(chunk_key: String) -> Array:
	return resource_cache.get(chunk_key, [])

# Convert world coordinates to chunk key
func get_chunk_key(world_x: int, world_y: int) -> String:
	var chunk_x = int(floor(float(world_x) / float(CHUNK_SIZE)))
	var chunk_y = int(floor(float(world_y) / float(CHUNK_SIZE)))
	return "%d,%d" % [chunk_x, chunk_y]

# Convert world coordinates to local chunk coordinates
func get_local_coords(world_x: int, world_y: int) -> Vector2i:
	var local_x = ((world_x % CHUNK_SIZE) + CHUNK_SIZE) % CHUNK_SIZE
	var local_y = ((world_y % CHUNK_SIZE) + CHUNK_SIZE) % CHUNK_SIZE
	return Vector2i(local_x, local_y)

# Convert chunk key to world coordinates of chunk origin
func chunk_key_to_world_origin(chunk_key: String) -> Vector2i:
	var parts = chunk_key.split(",")
	if parts.size() != 2:
		return Vector2i(0, 0)

	var chunk_x = int(parts[0])
	var chunk_y = int(parts[1])

	return Vector2i(chunk_x * CHUNK_SIZE, chunk_y * CHUNK_SIZE)

# Get terrain in a rectangular area (for efficient bulk queries)
func get_terrain_in_area(start_x: int, start_y: int, width: int, height: int) -> Array:
	var result: Array = []

	for y in range(height):
		var row: Array = []
		for x in range(width):
			var world_x = start_x + x
			var world_y = start_y + y
			row.append(get_terrain_at(world_x, world_y))
		result.append(row)

	return result

# Get resources in a rectangular area
func get_resources_in_area(start_x: int, start_y: int, width: int, height: int) -> Array:
	var result: Array = []

	for y in range(height):
		var row: Array = []
		for x in range(width):
			var world_x = start_x + x
			var world_y = start_y + y
			row.append(get_resource_at(world_x, world_y))
		result.append(row)

	return result

# Get cache statistics
func get_cache_stats() -> Dictionary:
	update_stats()
	return cache_stats.duplicate()

# Update internal cache statistics
func update_stats():
	cache_stats.terrain_chunks = terrain_cache.size()
	cache_stats.resource_chunks = resource_cache.size()

	# Calculate total tiles
	var total_tiles = 0
	for chunk_key in terrain_cache:
		var chunk = terrain_cache[chunk_key]
		if chunk is Array and chunk.size() > 0:
			total_tiles += chunk.size() * (chunk[0].size() if chunk[0] is Array else 0)

	cache_stats.total_tiles = total_tiles

# Clear all cached data
func clear_cache():
	print("🗂️ Clearing world data cache")
	var cleared_chunks = get_cached_chunk_keys().size()

	terrain_cache.clear()
	resource_cache.clear()
	update_stats()

	cache_cleared.emit()
	print("🗂️ Cleared %d chunks from cache" % cleared_chunks)

# Clear specific chunk from cache
func clear_chunk(chunk_key: String):
	var was_cached = false

	if terrain_cache.has(chunk_key):
		terrain_cache.erase(chunk_key)
		was_cached = true

	if resource_cache.has(chunk_key):
		resource_cache.erase(chunk_key)
		was_cached = true

	if was_cached:
		update_stats()
		print("🗂️ Cleared chunk from cache: ", chunk_key)

# Get cache size in bytes (rough estimate)
func get_cache_memory_usage() -> int:
	var size = 0

	# Estimate terrain cache size
	for chunk_key in terrain_cache:
		var chunk = terrain_cache[chunk_key]
		if chunk is Array:
			size += chunk.size() * 8  # Rough estimate per string reference

	# Estimate resource cache size
	for chunk_key in resource_cache:
		var chunk = resource_cache[chunk_key]
		if chunk is Array:
			size += chunk.size() * 8  # Rough estimate per string reference

	return size

# Print cache information for debugging
func debug_print_cache():
	print("=== World Data Cache Debug ===")
	print("Terrain chunks cached: ", terrain_cache.size())
	print("Resource chunks cached: ", resource_cache.size())
	print("Total unique chunks: ", get_cached_chunk_keys().size())
	print("Estimated memory usage: ", get_cache_memory_usage(), " bytes")
	print("Cached chunk keys: ", get_cached_chunk_keys())
	print("=== End Debug ===")

# Test cache functionality
func run_self_test():
	print("=== World Data Cache Self Test ===")

	# Test coordinate conversion
	var test_coords = [
		Vector2i(0, 0),
		Vector2i(16, 16),
		Vector2i(-1, -1),
		Vector2i(31, 31),
		Vector2i(-17, -17)
	]

	print("Testing coordinate conversion:")
	for coord in test_coords:
		var chunk_key = get_chunk_key(coord.x, coord.y)
		var local_coords = get_local_coords(coord.x, coord.y)
		print("  World ", coord, " -> Chunk ", chunk_key, " Local ", local_coords)

	# Test data storage and retrieval
	print("\nTesting data storage:")
	var test_chunk_key = "0,0"
	var test_terrain = [
		["Grass", "Grass", "Forest"],
		["Grass", "Forest", "Forest"],
		["Water", "Water", "Grass"]
	]
	var test_resources = [
		["", "TreeOak", ""],
		["Flower", "", "Rock"],
		["", "", ""]
	]

	store_terrain_chunk(test_chunk_key, test_terrain)
	store_resource_chunk(test_chunk_key, test_resources)

	# Test retrieval
	print("\nTesting data retrieval:")
	print("  Terrain at (0,0): ", get_terrain_at(0, 0))  # Should be "Grass"
	print("  Terrain at (1,1): ", get_terrain_at(1, 1))  # Should be "Forest"
	print("  Terrain at (2,0): ", get_terrain_at(2, 0))  # Should be "Forest"
	print("  Resource at (1,0): ", get_resource_at(1, 0))  # Should be "TreeOak"
	print("  Resource at (0,1): ", get_resource_at(0, 1))  # Should be "Flower"
	print("  Resource at (2,2): ", get_resource_at(2, 2))  # Should be ""

	# Test out-of-bounds
	print("\nTesting out-of-bounds:")
	print("  Terrain at (100, 100): ", get_terrain_at(100, 100))  # Should be default
	print("  Resource at (100, 100): ", get_resource_at(100, 100))  # Should be ""

	print("=== World Data Cache Self Test Complete ===")