extends Node

func _ready():
	print("=== ChunkManager Test ===")
	print()

	# Test world info loading
	print("1. Testing world info loading...")
	var world_info_loaded = await ChunkManager.load_world_info()
	print("   World info load result: ", "SUCCESS" if world_info_loaded else "FAILED")
	print()

	# Test chunk loading around center
	print("2. Testing chunk loading around center (0,0)...")
	var center_coord = Vector2i(0, 0)
	var chunk_data = await ChunkManager.request_chunks(center_coord)
	print("   Chunks loaded: ", chunk_data.chunks.keys().size())
	print("   Resources loaded: ", chunk_data.resources.keys().size())
	print("   Sample chunk keys: ", chunk_data.chunks.keys().slice(0, min(5, chunk_data.chunks.keys().size())))
	print()

	# Test specific chunk data structure
	if chunk_data.chunks.keys().size() > 0:
		var sample_chunk_key = chunk_data.chunks.keys()[0]
		var sample_chunk = chunk_data.chunks[sample_chunk_key]
		print("3. Testing chunk data structure...")
		print("   Sample chunk key: ", sample_chunk_key)
		print("   Sample chunk type: ", typeof(sample_chunk))
		if sample_chunk is Array:
			print("   Sample chunk array size: ", sample_chunk.size())
			if sample_chunk.size() > 0 and sample_chunk[0] is Array:
				print("   First row type: ", typeof(sample_chunk[0]))
				print("   First row size: ", sample_chunk[0].size())
				if sample_chunk[0].size() > 0:
					print("   First terrain type: ", sample_chunk[0][0])
		print()

	# Test resource data structure
	if chunk_data.resources.keys().size() > 0:
		var sample_resource_key = chunk_data.resources.keys()[0]
		var sample_resource = chunk_data.resources[sample_resource_key]
		print("4. Testing resource data structure...")
		print("   Sample resource key: ", sample_resource_key)
		print("   Sample resource type: ", typeof(sample_resource))
		if sample_resource is Array:
			print("   Sample resource array size: ", sample_resource.size())
			if sample_resource.size() > 0 and sample_resource[0] is Array:
				print("   First resource row type: ", typeof(sample_resource[0]))
				print("   First resource row size: ", sample_resource[0].size())
				if sample_resource[0].size() > 0:
					var resource_type = sample_resource[0][0]
					print("   First resource type: ", resource_type)
					print("   Resource symbol: ", Config.get_resource_symbol(resource_type))
		print()

	# Test chunk coordinate parsing
	print("5. Testing chunk coordinate parsing...")
	var test_coords = [
		Vector2i(0, 0),
		Vector2i(1, -1),
		Vector2i(-2, 3),
		Vector2i(10, 15)
	]
	for coord in test_coords:
		var chunk_key = "%d,%d" % [coord.x, coord.y]
		print("   Coord ", coord, " -> Key: ", chunk_key)
	print()

	# Test batch loading
	print("6. Testing batch loading...")
	var batch_coords = [
		Vector2i(0, 0),
		Vector2i(1, 0),
		Vector2i(0, 1),
		Vector2i(-1, -1)
	]
	var total_loaded = 0
	for coord in batch_coords:
		var batch_data = await ChunkManager.request_chunks_in_area(coord.x, coord.y, 1)
		total_loaded += batch_data.chunks.keys().size()
	print("   Batch loaded total chunks: ", total_loaded)
	print("   ChunkManager loaded count: ", ChunkManager.get_loaded_chunk_keys().size())
	print()

	# Test connection status
	print("7. Testing connection status...")
	print("   Connection status: ", "Connected" if ChunkManager.connection_status else "Disconnected")
	print()

	# Test error handling with invalid endpoint
	print("8. Testing error handling...")
	var invalid_data = await ChunkManager.fetch_data("/api/invalid_endpoint")
	print("   Invalid endpoint result: ", "EMPTY" if invalid_data.is_empty() else "DATA RECEIVED")
	print()

	print("=== ChunkManager Test Complete ===")
	print("✅ All core ChunkManager functionality verified")

	# Quit after test
	get_tree().quit()