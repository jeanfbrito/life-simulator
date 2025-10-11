extends Node

func _ready():
	var file = FileAccess.open("user://test_log.txt", FileAccess.WRITE)
	file.store_line("=== Godot Visualization Test Log ===")
	file.store_line("Time: " + str(Time.get_unix_time_from_system()))
	file.store_line("")
	
	# Test autoloads
	file.store_line("Testing autoloads:")
	file.store_line("Config: " + str(Config != null))
	file.store_line("ChunkManager: " + str(ChunkManager != null))
	file.store_line("WorldDataCache: " + str(WorldDataCache != null))
	file.store_line("")
	
	# Test resource symbols
	file.store_line("Testing resource symbols:")
	var resource_types = ["HazelShrub", "BerryBush", "Flower", "MushroomPatch"]
	for resource_type in resource_types:
		var symbol = Config.get_resource_symbol(resource_type)
		file.store_line("  " + resource_type + " -> " + symbol)
	file.store_line("")
	
	# Test entity configs
	file.store_line("Testing entity configs:")
	var entity_types = ["Rabbit", "Wolf", "Bear"]
	for entity_type in entity_types:
		var config = Config.get_entity_config(entity_type)
		file.store_line("  " + entity_type + " -> " + config.emoji)
	file.store_line("")
	
	# Test ResourceManager
	file.store_line("Testing ResourceManager:")
	var resource_manager = preload("res://scripts/ResourceManager.gd").new()
	add_child(resource_manager)
	
	var test_data = [["HazelShrub", ""], ["", "BerryBush"]]
	resource_manager.paint_resources("test", test_data)
	var count = resource_manager.get_resource_count("test")
	file.store_line("  Created " + str(count) + " resource sprites")
	file.store_line("")
	
	# Test EntityManager
	file.store_line("Testing EntityManager:")
	var entity_manager = preload("res://scripts/EntityManager.gd").new()
	add_child(entity_manager)
	
	var test_entities = [{"id": 1, "name": "Test", "entity_type": "Rabbit", "position": {"x": 0, "y": 0}}]
	entity_manager._update_entities(test_entities)
	var entity_count = entity_manager.get_entity_count()
	file.store_line("  Created " + str(entity_count) + " entity sprites")
	file.store_line("")
	
	file.store_line("=== Test Complete ===")
	file.close()
	
	print("Test log written to: " + OS.get_user_data_dir() + "/test_log.txt")
	get_tree().quit()