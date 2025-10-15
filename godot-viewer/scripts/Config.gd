# Config.gd - Singleton for configuration constants
# Replicates web-viewer JavaScript config values for Godot implementation

extends Node

signal configuration_loaded

# Display configuration - OpenRCT2 EXACT MATCH
# From: src/openrct2/world/Location.hpp
var TILE_SIZE: int = 64  # Tile width - OpenRCT2 isometric diamond (64×32)
var TILE_HEIGHT: int = 32  # Tile height - OpenRCT2 isometric diamond
var COORDS_XY_STEP: int = 32  # kCoordsXYStep - base coordinate step
var COORDS_Z_STEP: int = 8  # kCoordsZStep - pixels per Z level
var COORDS_Z_PER_TINY_Z: int = 16  # kCoordsZPerTinyZ - height division factor
var render_scale: float = 1.0  # Scale factor for rendering
var VIEW_SIZE_X: int = 100  # Dynamic view width based on container
var VIEW_SIZE_Y: int = 100  # Dynamic view height based on container

# Debug flags
var debug_show_position_markers: bool = false  # Show red cross at entity/resource origin
var slope_rotation: int = 1  # Slope corner rotation: 0=0°, 1=90°, 2=180°, 3=270° (TEST MODE)
var show_height_markers: bool = false  # Show height numbers on tiles (like OpenRCT2)
var show_grid: bool = false  # Show slope-following grid overlay (OpenRCT2 style)

# Performance settings
var target_fps: int = 60
var frame_delay: float = 1000.0 / 60.0

# Panning smoothing
var pan_smoothing: float = 0.2  # 0..1 how fast the camera catches up
var inertia_friction: float = 0.90  # 0..1 how quickly inertia slows down
var inertia_min_speed: float = 0.15  # px/frame threshold to stop inertia

# Chunk loading settings
var chunk_load_radius: int = 5
var chunk_load_debounce: int = 100  # ms delay for chunk loading
var chunk_batch_size: int = 10
var initial_chunk_radius: int = 5  # Load more chunks to show full world

# Zoom settings
var min_zoom: float = 0.25
var max_zoom: float = 4.0
var zoom_factor: float = 1.25

# Grass density visualization
var show_grass_density: bool = false  # Toggle for grass density overlay

# Network settings
var api_base_url: String = "http://localhost:54321"
var connection_timeout: int = 5000

# Terrain colors (from web-viewer)
var terrain_colors: Dictionary = {
	"Grass": Color("#3a7f47"),      # Brighter grass green
	"Stone": Color("#8b8680"),       # Lighter stone gray
	"Sand": Color("#f4d58f"),       # Brighter sand yellow
	"Water": Color("#4a90e2"),      # Brighter water blue
	"Dirt": Color("#8b6239"),       # Richer dirt brown
	"Snow": Color("#f0f0f0"),       # Slightly off-white snow
	"Forest": Color("#2d5a2d"),     # Darker forest green
	"Mountain": Color("#a8a8a8"),   # Lighter mountain gray
	"DeepWater": Color("#1e3a5f"),  # Darker deep water
	"ShallowWater": Color("#5ca7d8"), # Lighter shallow water
	"Swamp": Color("#5a6b3c"),      # Brighter swamp green
	"Desert": Color("#d4a76a")      # Brighter desert tan
}

# Resource colors and symbols
var resource_colors: Dictionary = {
	"TreeOak": Color("#0d4d0d"),
	"TreePine": Color("#0d3d0d"),
	"TreeBirch": Color("#1d5d1d"),
	"Rock": Color("#5a5a5a"),
	"Bush": Color("#2d4d2d"),
	"Flower": Color("#ff69b4")
}

# Resource symbols for rendering (emoji)
var resource_symbols: Dictionary = {
	"TreeOak": "🌳",
	"TreePine": "🌲",
	"TreeBirch": "🪾",
	"Rock": "🪨",
	"Bush": "🌳",
	"Flower": "🌸",
	"HazelShrub": "🌳",
	"OakTree": "🌳",
	"PineTree": "🌲",
	"BirchTree": "🪾",
	"Stone": "🪨",
	"BerryBush": "🫐",
	"MushroomPatch": "🍄",
	"WildRoot": "🥜"
}

# Resource rendering configuration
var resource_config: Dictionary = {
	"TreeOak": {
		"size_multiplier": 1.4,
		"offset_x": 0.0,
		"offset_y": -0.3
	},
	"TreePine": {
		"size_multiplier": 1.6,
		"offset_x": 0.0,
		"offset_y": -0.5
	},
	"TreeBirch": {
		"size_multiplier": 1.4,
		"offset_x": 0.0,
		"offset_y": -0.3
	},
	"Rock": {
		"size_multiplier": 0.6,
		"offset_x": 0.0,
		"offset_y": 0.1
	},
	"Bush": {
		"size_multiplier": 0.6,
		"offset_x": 0.0,
		"offset_y": 0.1
	},
	"Flower": {
		"size_multiplier": 0.4,
		"offset_x": 0.0,
		"offset_y": 0.0
	},
	"HazelShrub": {
		"size_multiplier": 0.8,
		"offset_x": 0.0,
		"offset_y": 0.1
	},
	"OakTree": {
		"size_multiplier": 1.4,
		"offset_x": 0.0,
		"offset_y": -0.3
	},
	"PineTree": {
		"size_multiplier": 1.6,
		"offset_x": 0.0,
		"offset_y": -0.5
	},
	"BirchTree": {
		"size_multiplier": 1.4,
		"offset_x": 0.0,
		"offset_y": -0.3
	},
	"Stone": {
		"size_multiplier": 0.6,
		"offset_x": 0.0,
		"offset_y": 0.1
	},
	"BerryBush": {
		"size_multiplier": 0.7,
		"offset_x": 0.0,
		"offset_y": 0.1
	},
	"MushroomPatch": {
		"size_multiplier": 0.5,
		"offset_x": 0.0,
		"offset_y": 0.0
	},
	"WildRoot": {
		"size_multiplier": 0.4,
		"offset_x": 0.0,
		"offset_y": 0.0
	}
}

# Entity configuration (loaded from API)
var entity_config: Dictionary = {
	"default": {
		"emoji": "❓",
		"size_multiplier": 1.0,
		"offset_x": 0.0,
		"offset_y": -0.2  # Default Y offset to keep feet in grid
	}
}

# Juvenile scales (will be loaded from API)
var juvenile_scales: Dictionary = {}

# Constants
var CHUNK_SIZE: int = 16
var DEFAULT_TERRAIN_TYPE: String = "DeepWater"
var DEFAULT_CENTER_CHUNK: Vector2i = Vector2i(0, 0)

# OpenRCT2 height limits - EXACT MATCH
# From: src/openrct2/Limits.h
const MIN_LAND_HEIGHT: int = 2
const MAX_TILE_HEIGHT: int = 254
const WATER_BASE_HEIGHT: int = 14

# Called when the node enters the scene tree for the first time.
func _ready():
	print("Config singleton initialized")

	# Load species configuration from backend API
	await load_species_config()

	configuration_loaded.emit()

# Update tile size based on zoom scale
func update_tile_size():
	TILE_SIZE = max(4, int(8 * render_scale))

# Get terrain color by type
func get_terrain_color(terrain_type: String) -> Color:
	return terrain_colors.get(terrain_type, terrain_colors[DEFAULT_TERRAIN_TYPE])

# Get resource symbol by type
func get_resource_symbol(resource_type: String) -> String:
	return resource_symbols.get(resource_type, "•")

# Get resource config by type
func get_resource_config(resource_type: String) -> Dictionary:
	return resource_config.get(resource_type, {
		"size_multiplier": 0.8,
		"offset_x": 0.0,
		"offset_y": 0.0
	})

# Get entity config by type
func get_entity_config(entity_type: String) -> Dictionary:
	return entity_config.get(entity_type, entity_config["default"])

# Load species configuration from API (equivalent to loadSpeciesConfig in web-viewer)
func load_species_config() -> void:
	print("Loading species configuration from API...")
	var http_request = HTTPRequest.new()
	add_child(http_request)

	# Make request to /api/species
	var error = http_request.request(api_base_url + "/api/species")
	if error != OK:
		print("⚠️ Failed to start species config request: ", error)
		http_request.queue_free()
		return

	# Wait for request to complete
	var result = await http_request.request_completed

	# result is [result, response_code, headers, body]
	if result[0] != HTTPRequest.RESULT_SUCCESS or result[1] != 200:
		print("⚠️ Failed to load species configuration, result: ", result[0], " code: ", result[1])
		http_request.queue_free()
		return

	var body = result[3]
	var json = JSON.new()
	var parse_result = json.parse(body.get_string_from_utf8())

	if parse_result != OK:
		print("⚠️ Failed to parse species config JSON")
		http_request.queue_free()
		return

	var data = json.data
	print("✅ Species configuration loaded from API")

	# Update entity config from API data
	if data.has("default_entity"):
		var default_data = data["default_entity"]
		entity_config["default"] = {
			"emoji": default_data.get("emoji", "❓"),
			"size_multiplier": default_data.get("sizeMultiplier", 1.0),
			"offset_x": default_data.get("offsetX", 0.0),
			"offset_y": default_data.get("offsetY", -0.2)
		}

	# Set species-specific configs
	if data.has("species"):
		for species_name in data["species"]:
			var species_data = data["species"][species_name]
			entity_config[species_name] = {
				"emoji": species_data.get("emoji", "❓"),
				"size_multiplier": species_data.get("viewer_scale", 1.0),
				"offset_x": 0.0,
				"offset_y": -0.2
			}

	# Set juvenile scales
	if data.has("juvenile_scales"):
		juvenile_scales = data["juvenile_scales"]

	print("Species loaded: ", entity_config.keys())

	# Clean up
	http_request.queue_free()