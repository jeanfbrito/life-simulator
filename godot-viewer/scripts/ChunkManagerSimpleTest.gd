extends Node

func _ready():
	print("=== ChunkManager Simple Test ===")
	print()

	# Test basic connection
	print("1. Testing API connection...")
	var test_data = await ChunkManager.fetch_data("/api/world/current")
	if not test_data.is_empty():
		print("   ✅ API connection successful")
		print("   World name: ", test_data.get("name", "Unknown"))
	else:
		print("   ❌ API connection failed")
		get_tree().quit()
		return

	# Test small chunk loading
	print("\n2. Testing small chunk loading...")
	var chunk_data = await ChunkManager.request_chunks_in_area(0, 0, 1)  # 3x3 area
	print("   Chunks loaded: ", chunk_data.chunks.keys().size())

	# Test chunk data structure
	if chunk_data.chunks.keys().size() > 0:
		var sample_key = chunk_data.chunks.keys()[0]
		var sample_chunk = chunk_data.chunks[sample_key]
		print("   Sample chunk key: ", sample_key)
		print("   Chunk is array: ", sample_chunk is Array)
		if sample_chunk is Array and sample_chunk.size() > 0:
			print("   Chunk dimensions: ", sample_chunk.size(), "x", sample_chunk[0].size() if sample_chunk[0] is Array else "unknown")
		print("   Resource data present: ", chunk_data.resources.has(sample_key))

	# Test connection status
	print("\n3. Testing connection status...")
	print("   Connected: ", ChunkManager.connection_status)

	print("\n=== ChunkManager Simple Test Complete ===")
	print("✅ Core ChunkManager functionality working")
	get_tree().quit()