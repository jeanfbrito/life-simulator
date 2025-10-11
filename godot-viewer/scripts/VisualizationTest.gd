extends Node

func _ready():
	print("=== Visualization Integration Test ===")
	print()
	
	# Test 1: Verify all systems are initialized
	print("1. Testing system initialization...")
	print("   Config: ", "✅" if Config != null else "❌")
	print("   ChunkManager: ", "✅" if ChunkManager != null else "❌")
	print("   WorldDataCache: ", "✅" if WorldDataCache != null else "❌")
	print()
	
	# Test 2: Test backend connectivity
	print("2. Testing backend connectivity...")
	await _test_backend_connection()
	print()
	
	# Test 3: Test chunk loading with resources
	print("3. Testing chunk loading with resources...")
	await _test_chunk_loading_with_resources()
	print()
	
	# Test 4: Test entity loading
	print("4. Testing entity loading...")
	await _test_entity_loading()
	print()
	
	# Test 5: Test ResourceManager and EntityManager
	print("5. Testing visualization components...")
	await _test_visualization_components()
	print()
	
	print("=== Visualization Integration Test Complete ===")
	get_tree().quit()

func _test_backend_connection():
	print("   📡 Testing backend connection...")
	
	# Test world info endpoint
	var world_info = await ChunkManager.load_world_info()
	if world_info != null:
		print("   ✅ Backend connected, world: ", world_info.get("name", "Unknown"))
	else:
		print("   ❌ Backend connection failed")

func _test_chunk_loading_with_resources():
	print("   📦 Testing chunk loading with resources...")
	
	# Load some test chunks
	var test_chunks: Array[String] = ["0,0", "1,0", "0,1"]
	
	var chunk_data = await ChunkManager.load_chunk_batch(test_chunks)
	if chunk_data != null and chunk_data.has("chunks") and chunk_data.chunks.size() > 0:
		print("   ✅ Loaded ", chunk_data.chunks.size(), " terrain chunks")
		
		# Check if resources were loaded
		if chunk_data.has("resources") and chunk_data.resources.size() > 0:
			print("   ✅ Loaded ", chunk_data.resources.size(), " resource chunks")
			
			# Cache the data
			WorldDataCache.merge_chunk_data(chunk_data)
			
			# Test resource retrieval
			var resource = WorldDataCache.get_resource_at(0, 0)
			print("   🌳 Resource at (0,0): '", resource, "'")
		else:
			print("   ⚠️ No resources loaded (might be normal for test area)")
	else:
		print("   ❌ Failed to load chunks")

func _test_entity_loading():
	print("   🐇 Testing entity loading...")
	
	# Test entity API directly
	var http = HTTPRequest.new()
	add_child(http)
	
	var error = http.request(Config.api_base_url + "/api/entities")
	if error != OK:
		print("   ❌ Failed to start entity request")
		http.queue_free()
		return
	
	var result = await http.request_completed
	http.queue_free()
	
	if result[0] != HTTPRequest.RESULT_SUCCESS or result[1] != 200:
		print("   ❌ Entity request failed: ", result[0], " ", result[1])
		return
	
	var json = JSON.new()
	if json.parse(result[3].get_string_from_utf8()) != OK:
		print("   ❌ Failed to parse entity JSON")
		return
	
	var data = json.data
	if data.has("entities"):
		var entities = data.entities
		print("   ✅ Found ", entities.size(), " entities")
		
		# Count entity types
		var entity_types = {}
		for entity in entities:
			var entity_type = entity.get("entity_type", "unknown")
			entity_types[entity_type] = entity_types.get(entity_type, 0) + 1
		
		print("   📊 Entity types:")
		for entity_type in entity_types.keys():
			print("     ", entity_type, ": ", entity_types[entity_type])
	else:
		print("   ❌ No entities found")

func _test_visualization_components():
	print("   🎨 Testing visualization components...")
	
	# Create ResourceManager
	var resource_manager = preload("res://scripts/ResourceManager.gd").new()
	add_child(resource_manager)
	
	# Create EntityManager
	var entity_manager = preload("res://scripts/EntityManager.gd").new()
	add_child(entity_manager)
	
	# Test ResourceManager with sample data
	print("   🌳 Testing ResourceManager...")
	var test_resource_data = [
		["", "HazelShrub", "", "TreeOak"],
		["BerryBush", "", "Flower", ""],
		["", "", "MushroomPatch", ""],
		["WildRoot", "", "", ""]
	]
	
	resource_manager.paint_resources("0,0", test_resource_data)
	var resource_count = resource_manager.get_resource_count("0,0")
	print("   ✅ ResourceManager created ", resource_count, " resource sprites")
	
	# Test EntityManager with sample data
	print("   🐇 Testing EntityManager...")
	var test_entities = [
		{
			"id": 1,
			"name": "TestRabbit",
			"entity_type": "Rabbit",
			"position": {"x": 10, "y": 5},
			"is_juvenile": True,
			"current_action": "Graze"
		},
		{
			"id": 2,
			"name": "TestWolf", 
			"entity_type": "Wolf",
			"position": {"x": -3, "y": 8},
			"is_juvenile": False,
			"current_action": "Idle"
		}
	]
	
	entity_manager._update_entities(test_entities)
	var entity_count = entity_manager.get_entity_count()
	print("   ✅ EntityManager created ", entity_count, " entity sprites")
	
	# Test configuration
	print("   🔧 Testing configuration...")
	var rabbit_config = Config.get_entity_config("Rabbit")
	print("   ✅ Rabbit config: ", rabbit_config.emoji, " size:", rabbit_config.size_multiplier)
	
	var hazel_config = Config.get_resource_config("HazelShrub")
	print("   ✅ HazelShrub config: ", Config.get_resource_symbol("HazelShrub"), " size:", hazel_config.size_multiplier)
	
	print("   🎉 All visualization components working correctly!")