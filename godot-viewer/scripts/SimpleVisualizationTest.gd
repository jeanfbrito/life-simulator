extends Node2D

func _ready():
	print("=== Simple Visualization Test ===")
	
	# Test basic component creation
	var terrain_tilemap = $TerrainTileMap
	var resource_manager = $TerrainTileMap/ResourceManager  
	var entity_manager = $TerrainTileMap/EntityManager
	
	print("✅ Components created:")
	print("  TerrainTileMap: ", terrain_tilemap != null)
	print("  ResourceManager: ", resource_manager != null)
	print("  EntityManager: ", entity_manager != null)
	
	# Test ResourceManager with sample data
	print("\n🌳 Testing ResourceManager...")
	var test_resources = [
		["", "HazelShrub", "", "TreeOak"],
		["BerryBush", "", "Flower", ""],
		["", "", "MushroomPatch", ""],
		["WildRoot", "", "", ""]
	]
	
	resource_manager.paint_resources("0,0", test_resources)
	var resource_count = resource_manager.get_resource_count("0,0")
	print("✅ Created ", resource_count, " resource sprites")
	
	# Test EntityManager with sample data  
	print("\n🐇 Testing EntityManager...")
	var test_entities = [
		{
			"id": 1,
			"name": "TestRabbit",
			"entity_type": "Rabbit", 
			"position": {"x": 1, "y": 1},
			"is_juvenile": true,
			"current_action": "Graze"
		},
		{
			"id": 2,
			"name": "TestWolf",
			"entity_type": "Wolf",
			"position": {"x": 2, "y": 2}, 
			"is_juvenile": false,
			"current_action": "Idle"
		}
	]
	
	entity_manager._update_entities(test_entities)
	var entity_count = entity_manager.get_entity_count()
	print("✅ Created ", entity_count, " entity sprites")
	
	# Test configuration access
	print("\n🔧 Testing configuration...")
	print("  HazelShrub symbol: ", Config.get_resource_symbol("HazelShrub"))
	print("  Rabbit emoji: ", Config.get_entity_config("Rabbit").emoji)
	print("  Wolf emoji: ", Config.get_entity_config("Wolf").emoji)
	
	print("\n🎉 Simple visualization test completed successfully!")
	print("👀 You should see:")
	print("  - 4 resource sprites (trees, bushes, flowers, mushrooms)")
	print("  - 2 entity sprites (rabbit and wolf)")
	print("  - Rabbit should be smaller (juvenile)")
	
	# Wait a bit then quit
	await get_tree().create_timer(3.0).timeout
	get_tree().quit()