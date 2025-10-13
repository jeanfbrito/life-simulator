extends Node
class_name TreeTextureManager

## Tree texture manager - loads extracted stone-kingdoms tree textures
## Supports multiple tree types (pine, birch) with multiple variants each

# Preloaded tree textures by type
var pine_tree_textures: Array[Texture2D] = []
var birch_tree_textures: Array[Texture2D] = []
var tree_textures_loaded: bool = false

# Stone-kingdoms tree offsets from quad_offset.lua
# Format: Vector2(offsetX, offsetY) in pixels
var pine_offsets: Array = [
	Vector2(26, 23), Vector2(27, 24), Vector2(26, 23), Vector2(26, 24), Vector2(26, 23),
	Vector2(26, 23), Vector2(26, 23), Vector2(26, 23), Vector2(26, 23), Vector2(26, 22),
	Vector2(26, 22), Vector2(26, 22), Vector2(26, 21), Vector2(26, 21), Vector2(26, 21),
	Vector2(26, 21), Vector2(26, 21), Vector2(26, 20), Vector2(26, 20), Vector2(26, 20),
	Vector2(26, 20), Vector2(26, 19), Vector2(26, 19), Vector2(26, 19), Vector2(26, 19)
]

var birch_offsets: Array = [
	Vector2(39, 27), Vector2(39, 27), Vector2(39, 28), Vector2(40, 27), Vector2(40, 28),
	Vector2(40, 28), Vector2(41, 28), Vector2(41, 28), Vector2(41, 28), Vector2(41, 28),
	Vector2(41, 29), Vector2(41, 29), Vector2(41, 29), Vector2(42, 30), Vector2(42, 30),
	Vector2(42, 31), Vector2(42, 31), Vector2(42, 32), Vector2(42, 32), Vector2(42, 32),
	Vector2(42, 31), Vector2(42, 31)
]

# Base tree offsets from Tree.lua
const TREE_BASE_OFFSET_X = -41  # (-3 - 38)
const TREE_BASE_OFFSET_Y = -166

func _ready():
	load_tree_textures()

func load_tree_textures():
	"""Load all tree texture variants from the extracted files."""
	print("🌲 Loading tree textures from extracted stone-kingdoms tiles...")

	var base_path = "assets/tiles/trees"

	# Load pine trees (25 variants)
	print("  Loading pine trees...")
	for i in range(1, 26):  # 01 through 25
		var filename = "tree_pine_large_%02d.png" % i
		var texture = _load_texture(base_path + "/" + filename, filename)
		if texture:
			pine_tree_textures.append(texture)

	# Load birch trees (22 variants)
	print("  Loading birch trees...")
	for i in range(1, 23):  # 01 through 22
		var filename = "tree_birch_large_%02d.png" % i
		var texture = _load_texture(base_path + "/" + filename, filename)
		if texture:
			birch_tree_textures.append(texture)

	# Report results
	var total_loaded = pine_tree_textures.size() + birch_tree_textures.size()
	if total_loaded > 0:
		print("✅ Loaded %d tree textures (Pine: %d, Birch: %d)" % [
			total_loaded,
			pine_tree_textures.size(),
			birch_tree_textures.size()
		])
		tree_textures_loaded = true
	else:
		push_error("❌ Failed to load any tree textures")
		tree_textures_loaded = false

func _load_texture(file_path: String, filename: String) -> Texture2D:
	"""Helper to load a single texture from file path."""
	var image = Image.new()
	var error = image.load(file_path)

	if error == OK:
		var texture = ImageTexture.create_from_image(image)
		if texture:
			return texture
		else:
			push_warning("🌲 Failed to create texture from image: " + filename)
	else:
		push_warning("🌲 Could not load tree texture: " + file_path + " (error: " + str(error) + ")")

	return null

func get_random_tree_texture(tree_type: String = "Wood") -> Texture2D:
	"""Get a random tree texture for the specified type.

	Args:
		tree_type: Resource type name (e.g., "Wood", "Pine", "Birch")

	Returns:
		Random tree texture or null if not available
	"""
	var data = get_random_tree_data(tree_type)
	return data["texture"] if data else null

func get_random_tree_data(tree_type: String = "Wood") -> Dictionary:
	"""Get a random tree texture and its stone-kingdoms offset data.

	Args:
		tree_type: Resource type name (e.g., "Wood", "Pine", "Birch")

	Returns:
		Dictionary with:
			- texture: Texture2D
			- offset: Vector2 (stone-kingdoms quad_offset)
			- is_pine: bool
			- index: int
		Returns empty dict if not available
	"""
	if not tree_textures_loaded:
		return {}

	var is_pine = false
	var texture_array: Array[Texture2D] = []
	var offset_array: Array = []

	match tree_type.to_lower():
		"pine":
			is_pine = true
			texture_array = pine_tree_textures
			offset_array = pine_offsets
		"birch":
			is_pine = false
			texture_array = birch_tree_textures
			offset_array = birch_offsets
		"wood", _:  # Default: randomly choose pine or birch
			# 50/50 mix of pine and birch
			if randf() < 0.5 and not pine_tree_textures.is_empty():
				is_pine = true
				texture_array = pine_tree_textures
				offset_array = pine_offsets
			elif not birch_tree_textures.is_empty():
				is_pine = false
				texture_array = birch_tree_textures
				offset_array = birch_offsets
			else:
				is_pine = true
				texture_array = pine_tree_textures
				offset_array = pine_offsets

	if texture_array.is_empty():
		return {}

	var index = randi() % texture_array.size()
	return {
		"texture": texture_array[index],
		"offset": offset_array[index] if index < offset_array.size() else Vector2.ZERO,
		"is_pine": is_pine,
		"index": index
	}

func has_textures() -> bool:
	"""Check if tree textures were successfully loaded."""
	return tree_textures_loaded and (
		not pine_tree_textures.is_empty() or
		not birch_tree_textures.is_empty()
	)

func get_texture_count() -> int:
	"""Get the total number of loaded tree textures."""
	return pine_tree_textures.size() + birch_tree_textures.size()

func get_pine_count() -> int:
	"""Get the number of loaded pine tree textures."""
	return pine_tree_textures.size()

func get_birch_count() -> int:
	"""Get the number of loaded birch tree textures."""
	return birch_tree_textures.size()
