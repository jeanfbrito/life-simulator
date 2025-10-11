extends Node

func _ready():
	print("=== World Integration Test ===")
	print()

	# Test 1: Verify all systems are initialized
	print("1. Testing system initialization...")
	print("   Config: ", "✅" if Config != null else "❌")
	print("   ChunkManager: ", "✅" if ChunkManager != null else "❌")
	print("   WorldDataCache: ", "✅" if WorldDataCache != null else "❌")
	print()

	# Test 2: Test backend connectivity
	print("2. Testing backend connectivity...")
	_test_backend_connection()
	print()

	# Test 3: Test chunk loading
	print("3. Testing chunk loading...")
	await _test_chunk_loading()
	print()

	print("=== World Integration Test Complete ===")

func _test_backend_connection():
	print("   📡 Testing backend connection...")

	# Test world info endpoint
	var world_info = await ChunkManager.load_world_info()
	if world_info != null:
		print("   ✅ Backend connected, world: ", world_info.get("name", "Unknown"))
	else:
		print("   ❌ Backend connection failed")

func _test_chunk_loading():
	print("   📦 Testing chunk loading...")

	# Load some test chunks
	var test_chunks: Array[String] = ["0,0", "1,0", "0,1", "-1,0", "0,-1"]

	var chunk_data = await ChunkManager.load_chunk_batch(test_chunks)
	if chunk_data != null and chunk_data.has("chunks") and chunk_data.chunks.size() > 0:
		print("   ✅ Loaded ", chunk_data.chunks.size(), " chunks")

		# Test caching
		WorldDataCache.merge_chunk_data(chunk_data)

		# Test terrain retrieval
		var terrain = WorldDataCache.get_terrain_at(0, 0)
		print("   🗺️ Terrain at (0,0): ", terrain)
	else:
		print("   ❌ Failed to load chunks")

func _on_timer_timeout():
	get_tree().quit()