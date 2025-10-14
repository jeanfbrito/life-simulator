extends Node
class_name GrassTextureManagerRCT2

## RCT2 Grass Texture Manager - loads RollerCoaster Tycoon 2 grass terrain tiles
## Uses the flat grass tile (slope_00) for simple 1x1 grass decoration

# Preloaded grass texture (flat tile)
var grass_flat_texture: Texture2D = null
var grass_texture_loaded: bool = false

func _ready():
	load_grass_texture()

func load_grass_texture():
	"""Load the flat RCT2 grass tile for decoration."""
	print("🌿 Loading RCT2 grass texture...")

	var file_path = "assets/tiles/terrain/openrct2_placeholder/grass/slope_00.png"

	var image = Image.new()
	var error = image.load(file_path)

	if error == OK:
		grass_flat_texture = ImageTexture.create_from_image(image)
		if grass_flat_texture:
			grass_texture_loaded = true
			print("✅ Loaded RCT2 grass texture (64×31 isometric tile)")
		else:
			push_error("❌ Failed to create texture from grass image")
	else:
		push_error("❌ Could not load RCT2 grass texture: " + file_path + " (error: " + str(error) + ")")

func get_grass_texture() -> Texture2D:
	"""Get the RCT2 flat grass texture.

	Returns:
		Flat grass tile texture or null if not loaded
	"""
	return grass_flat_texture if grass_texture_loaded else null

func has_textures() -> bool:
	"""Check if grass texture was successfully loaded."""
	return grass_texture_loaded and grass_flat_texture != null
