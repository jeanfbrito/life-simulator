extends Node
class_name TreeTextureManagerRCT2

## RCT2 Tree Texture Manager - loads extracted RCT2 grass tree textures
## Each tree has 4 isometric views (NE, SE, SW, NW)
## 11 different tree species from RollerCoaster Tycoon 2

# Tree species available
const TREE_SPECIES = [
	"aleppo_pine",
	"black_poplar",
	"caucasian_fir",
	"cedar_lebanon",
	"corsican_pine",
	"european_larch",
	"montezuma_pine",
	"red_fir",
	"red_fir_2",
	"red_fir_3",
	"scots_pine"
]

# Isometric views
const VIEWS = ["ne", "se", "sw", "nw"]

# Loaded textures: Dictionary[species_name][view] -> Texture2D
var tree_textures: Dictionary = {}
var tree_textures_loaded: bool = false

# Total texture count
var total_textures: int = 0


func _ready():
	load_tree_textures()


func load_tree_textures():
	"""Load all RCT2 tree textures (11 species × 4 views = 44 textures)"""
	print("🌲 Loading RCT2 tree textures...")

	var base_path = "assets/tiles/trees/rct2"

	for species in TREE_SPECIES:
		tree_textures[species] = {}

		for view in VIEWS:
			var filename = "tree_%s_%s.png" % [species, view]
			var file_path = base_path + "/" + filename

			var texture = _load_texture(file_path, filename)
			if texture:
				tree_textures[species][view] = texture
				total_textures += 1

	# Report results
	if total_textures > 0:
		print("✅ Loaded %d RCT2 tree textures (%d species × 4 views)" % [
			total_textures,
			TREE_SPECIES.size()
		])
		tree_textures_loaded = true
	else:
		push_error("❌ Failed to load any RCT2 tree textures")
		tree_textures_loaded = false


func _load_texture(file_path: String, filename: String) -> Texture2D:
	"""Helper to load a single texture from file path"""
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


func get_random_tree_texture() -> Texture2D:
	"""Get a random tree texture (random species, random view)

	Returns:
		Random tree texture or null if not available
	"""
	var data = get_random_tree_data()
	return data["texture"] if data else null


func get_random_tree_data() -> Dictionary:
	"""Get a random tree texture with metadata

	Returns:
		Dictionary with:
			- texture: Texture2D
			- species: String (tree species name)
			- view: String (ne/se/sw/nw)
		Returns empty dict if not available
	"""
	if not tree_textures_loaded or tree_textures.is_empty():
		return {}

	# Pick random species
	var species = TREE_SPECIES[randi() % TREE_SPECIES.size()]

	# Pick random view (or use specific view for consistent rendering)
	var view = VIEWS[randi() % VIEWS.size()]

	# Get texture
	if tree_textures.has(species) and tree_textures[species].has(view):
		return {
			"texture": tree_textures[species][view],
			"species": species,
			"view": view
		}

	return {}


func get_tree_texture(species: String, view: String = "") -> Texture2D:
	"""Get a specific tree texture by species and optional view

	Args:
		species: Tree species name (e.g., "scots_pine", "cedar_lebanon")
		view: Optional view direction ("ne", "se", "sw", "nw"). Random if not specified.

	Returns:
		Tree texture or null if not found
	"""
	if not tree_textures_loaded or not tree_textures.has(species):
		return null

	# If no view specified, pick random
	if view == "":
		view = VIEWS[randi() % VIEWS.size()]

	if tree_textures[species].has(view):
		return tree_textures[species][view]

	return null


func get_species_list() -> Array:
	"""Get list of all available tree species"""
	return TREE_SPECIES.duplicate()


func has_textures() -> bool:
	"""Check if tree textures were successfully loaded"""
	return tree_textures_loaded and total_textures > 0


func get_texture_count() -> int:
	"""Get the total number of loaded tree textures"""
	return total_textures


func get_species_count() -> int:
	"""Get the number of tree species available"""
	return tree_textures.size()
