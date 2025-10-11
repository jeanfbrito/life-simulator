# test_integration.gd - Integration test for ResourceManager and EntityManager
# Run with: godot --headless --script test_integration.gd

extends SceneTree

func _init():
	print("🧪 Starting integration test for ResourceManager and EntityManager...")
	
	# Test the configuration loading
	test_config_loading()
	
	# Test resource manager logic
	test_resource_manager()
	
	# Test entity manager logic  
	test_entity_manager()
	
	print("🎉 Integration test completed!")
	quit()

func test_config_loading():
	print("\n📋 Testing Config loading...")
	
	var config_script = load("res://scripts/Config.gd")
	var config = config_script.new()
	root.add_child(config)
	
	# Test resource symbols
	var resource_types = ["HazelShrub", "BerryBush", "Flower", "MushroomPatch", "WildRoot"]
	print("🌳 Resource symbols:")
	for resource_type in resource_types:
		var symbol = config.get_resource_symbol(resource_type)
		print(f"  {resource_type} -> {symbol}")
	
	# Test entity config
	var entity_types = ["Rabbit", "Wolf", "Bear", "Fox"]
	print("🐇 Entity configurations:")
	for entity_type in entity_types:
		var entity_config = config.get_entity_config(entity_type)
		print(f"  {entity_type} -> {entity_config.emoji} (size: {entity_config.size_multiplier})")

func test_resource_manager():
	print("\n🌳 Testing ResourceManager logic...")
	
	var resource_manager_script = load("res://scripts/ResourceManager.gd")
	var resource_manager = resource_manager_script.new()
	root.add_child(resource_manager)
	
	# Test resource data
	var test_chunk_key = "0,0"
	var test_resource_data = [
		["", "HazelShrub", "", "TreeOak"],
		["BerryBush", "", "Flower", ""],
		["", "", "MushroomPatch", ""],
		["WildRoot", "", "", ""]
	]
	
	print(f"🎨 Testing paint_resources for chunk {test_chunk_key}")
	resource_manager.paint_resources(test_chunk_key, test_resource_data)
	
	var sprite_count = resource_manager.get_resource_count(test_chunk_key)
	print(f"✅ Created {sprite_count} resource sprites")
	
	# Test clearing
	resource_manager.clear_resources(test_chunk_key)
	var cleared_count = resource_manager.get_resource_count(test_chunk_key)
	print(f"✅ After clearing: {cleared_count} sprites")

func test_entity_manager():
	print("\n🐇 Testing EntityManager logic...")
	
	var entity_manager_script = load("res://scripts/EntityManager.gd")
	var entity_manager = entity_manager_script.new()
	root.add_child(entity_manager)
	
	# Test entity data
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
	
	print(f"🐇 Testing entity creation for {test_entities.size()} entities")
	entity_manager._update_entities(test_entities)
	
	var entity_count = entity_manager.get_entity_count()
	print(f"✅ Created {entity_count} entity sprites")
	
	# Test entity types
	var rabbits = entity_manager.get_entities_by_type("Rabbit")
	var wolves = entity_manager.get_entities_by_type("Wolf")
	print(f"🐇 Rabbits: {rabbits.size()}, Wolves: {wolves.size()}")
	
	# Test clearing
	entity_manager.clear_all_entities()
	var cleared_count = entity_manager.get_entity_count()
	print(f"✅ After clearing: {cleared_count} entities")