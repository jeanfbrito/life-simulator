extends Node
class_name RCT2TerrainTextureManager

## RCT2 Terrain Texture Manager - loads RollerCoaster Tycoon 2 terrain tiles
## Supports multiple terrain types (grass, sand, dirt, rock, etc.)
## Uses flat tiles (slope_00) for simple terrain rendering

# Loaded terrain textures: Dictionary[terrain_type] -> Texture2D
var terrain_textures: Dictionary = {}
var textures_loaded: bool = false

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
	"""Load flat RCT2 terrain tiles (slope_00) for all terrain types."""
	print("🌍 Loading RCT2 terrain textures...")

	var base_path = "assets/tiles/terrain/openrct2_placeholder"
	var loaded_count = 0

	for terrain_key in TERRAIN_TYPES.keys():
		var terrain_folder = TERRAIN_TYPES[terrain_key]
		var file_path = base_path + "/" + terrain_folder + "/slope_00.png"

		var image = Image.new()
		var error = image.load(file_path)

		if error == OK:
			var texture = ImageTexture.create_from_image(image)
			if texture:
				terrain_textures[terrain_key] = texture
				loaded_count += 1
		else:
			push_warning("🌍 Could not load RCT2 terrain: " + file_path + " (error: " + str(error) + ")")

	if loaded_count > 0:
		textures_loaded = true
		print("✅ Loaded %d RCT2 terrain textures" % loaded_count)
	else:
		push_error("❌ Failed to load any RCT2 terrain textures")
		textures_loaded = false

func get_terrain_texture(terrain_type: String) -> Texture2D:
	"""Get RCT2 flat terrain texture for a specific type.

	Args:
		terrain_type: Terrain type key (e.g., "grass", "sand", "dirt")

	Returns:
		Terrain texture or null if not loaded
	"""
	var terrain_key = terrain_type.to_lower()

	# Direct match
	if terrain_textures.has(terrain_key):
		return terrain_textures[terrain_key]

	# Try mapping backend terrain names to RCT2 names
	match terrain_key:
		"grass", "forest":
			return terrain_textures.get("grass")
		"sand", "desert":
			return terrain_textures.get("sand")
		"dirt":
			return terrain_textures.get("dirt")
		"stone", "mountain":
			return terrain_textures.get("rock")
		"snow":
			return terrain_textures.get("ice")
		_:
			return null

func has_textures() -> bool:
	"""Check if terrain textures were successfully loaded."""
	return textures_loaded and not terrain_textures.is_empty()

func get_texture_count() -> int:
	"""Get the number of loaded terrain textures."""
	return terrain_textures.size()
