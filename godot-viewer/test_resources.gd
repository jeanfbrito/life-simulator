# test_resources.gd - Simple test script for ResourceManager functionality
# Run with: godot --headless --script test_resources.gd

extends SceneTree

func _init():
	print("🧪 Testing ResourceManager functionality...")
	
	# Load required scripts
	var config_script = load("res://scripts/Config.gd")
	var world_data_cache_script = load("res://scripts/WorldDataCache.gd")
	var resource_manager_script = load("res://scripts/ResourceManager.gd")
	
	# Create instances
	var config = config_script.new()
	var world_data_cache = world_data_cache_script.new()
	var resource_manager = resource_manager_script.new()
	
	# Add to tree for proper initialization
	root.add_child(config)
	root.add_child(world_data_cache)
	root.add_child(resource_manager)
	
	# Wait for initialization
	await process_frame
	
	# Test resource data
	var test_chunk_key = "0,0"
	var test_resource_data = [
		["", "HazelShrub", "", "TreeOak"],
		["BerryBush", "", "Flower", ""],
		["", "", "MushroomPatch", ""],
		["WildRoot", "", "", ""]
	]
	
	print("📝 Testing with resource data:")
	for y in range(test_resource_data.size()):
		var row = ""
		for x in range(test_resource_data[y].size()):
			row += test_resource_data[y][x] + " "
		print("  ", row)
	
	# Store test data in cache
	world_data_cache.store_resource_chunk(test_chunk_key, test_resource_data)
	
	# Test ResourceManager
	print("🎨 Testing ResourceManager.paint_resources...")
	resource_manager.paint_resources(test_chunk_key, test_resource_data)
	
	# Check results
	var sprite_count = resource_manager.get_resource_count(test_chunk_key)
	print("✅ ResourceManager created ", sprite_count, " resource sprites")
	
	# Test resource symbols
	print("🔤 Testing resource symbols:")
	var resource_types = ["HazelShrub", "TreeOak", "BerryBush", "Flower", "MushroomPatch", "WildRoot"]
	for resource_type in resource_types:
		var symbol = config.get_resource_symbol(resource_type)
		print("  ", resource_type, " -> ", symbol)
	
	# Test clearing
	print("🗑️ Testing resource clearing...")
	resource_manager.clear_resources(test_chunk_key)
	var cleared_count = resource_manager.get_resource_count(test_chunk_key)
	print("✅ After clearing: ", cleared_count, " sprites")
	
	print("🎉 ResourceManager test completed successfully!")
	quit()