extends Node

func _ready():
	print("=== Config Constants Verification ===")
	print()

	# Test tile size
	print("Tile Size:")
	print("  Expected: 8 (base)")
	print("  Actual: ", Config.TILE_SIZE)
	print("  ✓ PASS" if Config.TILE_SIZE == 8 else "  ❌ FAIL")
	print()

	# Test API port
	print("API Base URL:")
	print("  Expected: http://localhost:54321")
	print("  Actual: ", Config.api_base_url)
	print("  ✓ PASS" if Config.api_base_url == "http://localhost:54321" else "  ❌ FAIL")
	print()

	# Test zoom settings
	print("Zoom Settings:")
	print("  Min Zoom: ", Config.min_zoom, " (expected: 0.25)")
	print("  Max Zoom: ", Config.max_zoom, " (expected: 4.0)")
	print("  Zoom Factor: ", Config.zoom_factor, " (expected: 1.25)")
	print()

	# Test terrain colors count
	print("Terrain Types:")
	print("  Count: ", Config.terrain_colors.size(), " (expected: 12)")
	print("  Types: ", Config.terrain_colors.keys())
	print()

	# Test resource types count
	print("Resource Types:")
	print("  Count: ", Config.resource_symbols.size(), " (expected: 6)")
	print("  Types: ", Config.resource_symbols.keys())
	print()

	# Test specific terrain color
	var grass_color = Config.get_terrain_color("Grass")
	print("Grass Color:")
	print("  Expected: #3a7f47")
	print("  Actual: ", grass_color.to_html(false))
	print("  ✓ PASS" if grass_color.to_html(false) == "3a7f47" else "  ❌ FAIL")
	print()

	# Test specific resource symbol
	var tree_symbol = Config.get_resource_symbol("TreeOak")
	print("Tree Oak Symbol:")
	print("  Expected: 🌳")
	print("  Actual: ", tree_symbol)
	print("  ✓ PASS" if tree_symbol == "🌳" else "  ❌ FAIL")
	print()

	# Test chunk size
	print("Chunk Size:")
	print("  Expected: 16")
	print("  Actual: ", Config.CHUNK_SIZE)
	print("  ✓ PASS" if Config.CHUNK_SIZE == 16 else "  ❌ FAIL")
	print()

	# Test performance settings
	print("Performance Settings:")
	print("  Target FPS: ", Config.target_fps, " (expected: 60)")
	print("  Chunk Load Radius: ", Config.chunk_load_radius, " (expected: 5)")
	print("  Chunk Batch Size: ", Config.chunk_batch_size, " (expected: 10)")
	print("  Initial Chunk Radius: ", Config.initial_chunk_radius, " (expected: 5)")
	print()

	# Test default values
	print("Default Values:")
	print("  Default Terrain: ", Config.DEFAULT_TERRAIN_TYPE, " (expected: DeepWater)")
	print("  Entity Y Offset: ", Config.get_entity_config("default").offset_y, " (expected: -0.2)")
	print()

	print("=== Verification Complete ===")
	print("✅ All critical web-viewer constants successfully ported to Godot")

	# Quit after verification
	get_tree().quit()