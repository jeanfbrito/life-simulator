extends Control

func _ready():
	print("Life Simulator Viewer - Main scene loaded")

	# Test Config singleton
	print("Config singleton test:")
	print("- Tile Size: ", Config.TILE_SIZE)
	print("- API Base URL: ", Config.api_base_url)
	print("- Terrain Colors: ", Config.terrain_colors.keys().size(), " types loaded")
	print("- Resource Symbols: ", Config.resource_symbols.keys().size(), " types loaded")

	# Test some values match web-viewer
	var grass_color = Config.get_terrain_color("Grass")
	var tree_symbol = Config.get_resource_symbol("TreeOak")
	print("- Grass color: ", grass_color)
	print("- Tree Oak symbol: ", tree_symbol)

	$VBoxContainer/StatusLabel.text = "Main scene loaded successfully. Config singleton working.\nTerrain types: %d, Resource types: %d" % [Config.terrain_colors.size(), Config.resource_symbols.size()]

func _on_exit_button_pressed():
	print("Exit button pressed - test successful")
	get_tree().quit()