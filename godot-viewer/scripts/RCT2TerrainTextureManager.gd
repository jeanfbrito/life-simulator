extends Node
class_name RCT2TerrainTextureManager

## RCT2 Terrain Texture Manager - loads RollerCoaster Tycoon 2 terrain tiles
## Supports multiple terrain types (grass, sand, dirt, rock, etc.)
## Loads all 19 slope variations (slope_00 through slope_18) per terrain type

# Loaded terrain textures: Dictionary[terrain_type][slope_index] -> Texture2D
# Example: terrain_textures["grass"][0] = flat grass tile
#          terrain_textures["grass"][1] = grass with north corner up
var terrain_textures: Dictionary = {}
var textures_loaded: bool = false

# Number of slope variations (OpenRCT2 has 19: flat + 18 slopes)
const NUM_SLOPES = 19

# Available RCT2 terrain types
const TERRAIN_TYPES = {
	"grass": "grass",
	"sand": "sand",
	"dirt": "dirt",
	"rock": "rock",
	"ice": "ice",
	"sand_red": "sand_red",
	"sand_yellow": "sand_yellow",
	"grass_clumps": "grass_clumps",
	"grass_mowed": "grass_mowed",
}

func _ready():
	load_terrain_textures()

func load_terrain_textures():
	"""Load all 19 slope variants for each RCT2 terrain type."""
	print("🌍 Loading RCT2 terrain textures (all 19 slopes)...")

	var base_path = "assets/tiles/terrain/openrct2_placeholder"
	var loaded_terrain_count = 0
	var total_textures_loaded = 0

	for terrain_key in TERRAIN_TYPES.keys():
		var terrain_folder = TERRAIN_TYPES[terrain_key]
		terrain_textures[terrain_key] = {}
		var slopes_loaded = 0

		# Load all 19 slope variations (slope_00.png through slope_18.png)
		for slope_idx in range(NUM_SLOPES):
			var file_path = base_path + "/" + terrain_folder + "/slope_%02d.png" % slope_idx
			var image = Image.new()
			var error = image.load(file_path)

			if error == OK:
				var texture = ImageTexture.create_from_image(image)
				if texture:
					terrain_textures[terrain_key][slope_idx] = texture
					slopes_loaded += 1
					total_textures_loaded += 1
			else:
				# Only warn if slope_00 is missing (critical), others are optional
				if slope_idx == 0:
					push_warning("🌍 Could not load RCT2 terrain: " + file_path + " (error: " + str(error) + ")")

		if slopes_loaded > 0:
			loaded_terrain_count += 1
			if slopes_loaded < NUM_SLOPES:
				print("⚠️ Loaded %d/%d slopes for %s" % [slopes_loaded, NUM_SLOPES, terrain_key])

	if loaded_terrain_count > 0:
		textures_loaded = true
		print("✅ Loaded %d terrain types with %d total slope textures" % [loaded_terrain_count, total_textures_loaded])
	else:
		push_error("❌ Failed to load any RCT2 terrain textures")
		textures_loaded = false

func get_terrain_texture(terrain_type: String, slope_index: int = 0) -> Texture2D:
	"""Get RCT2 terrain texture for a specific type and slope.

	Args:
		terrain_type: Terrain type key (e.g., "grass", "sand", "dirt")
		slope_index: Slope variation (0-18, default 0 = flat)

	Returns:
		Terrain texture or null if not loaded
	"""
	var terrain_key = map_terrain_to_rct2(terrain_type)

	if terrain_key and terrain_textures.has(terrain_key):
		var terrain_slopes = terrain_textures[terrain_key]
		if terrain_slopes.has(slope_index):
			return terrain_slopes[slope_index]
		# Fallback to flat if slope not found
		elif terrain_slopes.has(0):
			return terrain_slopes[0]

	return null

func map_terrain_to_rct2(terrain_type: String) -> String:
	"""Map backend terrain type to RCT2 terrain key.

	Args:
		terrain_type: Backend terrain type (e.g., "Grass", "Forest", "Desert")

	Returns:
		RCT2 terrain key or empty string if no mapping
	"""
	var terrain_key = terrain_type.to_lower()

	# Direct match
	if TERRAIN_TYPES.has(terrain_key):
		return terrain_key

	# Try mapping backend terrain names to RCT2 names
	match terrain_key:
		"grass", "forest":
			return "grass"
		"sand", "desert":
			return "sand"
		"dirt":
			return "dirt"
		"stone", "mountain":
			return "rock"
		"snow":
			return "ice"
		_:
			return ""

func has_textures() -> bool:
	"""Check if terrain textures were successfully loaded."""
	return textures_loaded and not terrain_textures.is_empty()

func get_texture_count() -> int:
	"""Get the number of loaded terrain textures."""
	return terrain_textures.size()
