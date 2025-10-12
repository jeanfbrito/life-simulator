extends Node
class_name GrassTextureManager

## Simple grass texture manager - loads extracted stone-kingdoms grass textures
## Start with 1x1 tiles for simplicity, can expand to macro tiles later

# Preloaded grass textures (cached at startup)
var grass_1x1_textures: Array[Texture2D] = []
var grass_textures_loaded: bool = false

func _ready():
	load_grass_textures()

func load_grass_textures():
	"""Load all 1x1 grass texture variants from the extracted files."""
	print("🌿 Loading grass textures from extracted stone-kingdoms tiles...")

	var textures_loaded = 0

	# Get the actual filesystem path (not res://)
	var base_path = "assets/tiles/grass"

	# Load 1x1 grass tiles (8 variants)
	for i in range(1, 9):  # 01 through 08
		var filename = "abundant_grass_1x1_%02d.png" % i
		var file_path = base_path + "/" + filename

		# Try to load using Image first
		var image = Image.new()
		var error = image.load(file_path)

		if error == OK:
			var texture = ImageTexture.create_from_image(image)
			if texture:
				grass_1x1_textures.append(texture)
				textures_loaded += 1
				print("  ✅ Loaded " + filename)
		else:
			push_warning("🌿 Could not load grass texture: " + file_path + " (error: " + str(error) + ")")

	if grass_1x1_textures.size() > 0:
		print("✅ Loaded %d grass texture variants" % grass_1x1_textures.size())
		grass_textures_loaded = true
	else:
		push_error("❌ Failed to load any grass textures from res://assets/tiles/grass/")
		grass_textures_loaded = false

func get_random_grass_texture() -> Texture2D:
	"""Get a random grass texture variant."""
	if not grass_textures_loaded or grass_1x1_textures.is_empty():
		push_warning("🌿 No grass textures loaded, returning null")
		return null

	var index = randi() % grass_1x1_textures.size()
	return grass_1x1_textures[index]

func has_textures() -> bool:
	"""Check if grass textures were successfully loaded."""
	return grass_textures_loaded and not grass_1x1_textures.is_empty()

func get_texture_count() -> int:
	"""Get the number of loaded grass textures."""
	return grass_1x1_textures.size()
