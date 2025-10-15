# TopBar.gd - Main UI top bar with statistics and action buttons
extends CanvasLayer
class_name TopBar

# UI Elements - Statistics (Left)
@onready var stats_container: HBoxContainer = $Container/StatsContainer
@onready var world_name_label: Label = $Container/StatsContainer/WorldNameLabel
@onready var separator1: Label = $Container/StatsContainer/Separator1
@onready var fps_label: Label = $Container/StatsContainer/FPSLabel
@onready var separator2: Label = $Container/StatsContainer/Separator2
@onready var entity_label: Label = $Container/StatsContainer/EntityLabel
@onready var separator3: Label = $Container/StatsContainer/Separator3
@onready var chunk_label: Label = $Container/StatsContainer/ChunkLabel

# UI Elements - Actions (Right)
@onready var actions_container: HBoxContainer = $Container/ActionsContainer
@onready var grid_button: Button = $Container/ActionsContainer/GridButton
@onready var camera_button: Button = $Container/ActionsContainer/CameraButton
@onready var stats_button: Button = $Container/ActionsContainer/StatsButton
@onready var help_button: Button = $Container/ActionsContainer/HelpButton
@onready var refresh_button: Button = $Container/ActionsContainer/RefreshButton
@onready var rotation_button: Button = $Container/ActionsContainer/RotationButton
@onready var height_button: Button = $Container/ActionsContainer/HeightButton

# References to other components
var world_renderer: Node2D = null
var statistics_hud: Control = null
var controls_overlay: Control = null
var height_marker_overlay = null  # HeightMarkerOverlay reference

# Update timing
var update_interval: float = 0.5  # Update statistics every 0.5 seconds
var time_since_update: float = 0.0

func _ready() -> void:
	print("📊 TopBar _ready() called")
	print("  Layer: ", layer)
	print("  Visible: ", visible)

	# Make absolutely sure we're visible
	visible = true
	layer = 100

	# Debug: Check if Container exists
	var container = get_node_or_null("Container")
	if container:
		print("  ✅ Container found")
		container.visible = true
	else:
		print("  ❌ Container NOT found!")
		return

	# Setup tooltips for action buttons
	setup_button_tooltips()

	# Connect button signals
	connect_button_signals()

	# Initial statistics update
	update_statistics()

	print("📊 TopBar initialized successfully")

func _process(delta: float) -> void:
	# Update statistics periodically
	time_since_update += delta
	if time_since_update >= update_interval:
		update_statistics()
		time_since_update = 0.0

# Setup tooltips for all action buttons
func setup_button_tooltips() -> void:
	if grid_button:
		grid_button.tooltip_text = "Toggle Grid (G)"
	if camera_button:
		camera_button.tooltip_text = "Reset Camera (R)"
	if stats_button:
		stats_button.tooltip_text = "Toggle Statistics (Tab)"
	if help_button:
		help_button.tooltip_text = "Toggle Help (H)"
	if refresh_button:
		refresh_button.tooltip_text = "Reload World (Full Refresh - R)"
	if rotation_button:
		_update_rotation_button_text()  # Set initial text

# Connect all button signals
func connect_button_signals() -> void:
	if grid_button:
		grid_button.pressed.connect(_on_grid_button_pressed)
	if camera_button:
		camera_button.pressed.connect(_on_camera_reset_pressed)
	if stats_button:
		stats_button.pressed.connect(_on_stats_toggle_pressed)
	if help_button:
		help_button.pressed.connect(_on_help_toggle_pressed)
	if refresh_button:
		refresh_button.pressed.connect(_on_refresh_pressed)
	if rotation_button:
		rotation_button.pressed.connect(_on_rotation_button_pressed)
	if height_button:
		height_button.pressed.connect(_on_height_button_pressed)

# Update all statistics displays
func update_statistics() -> void:
	update_world_name()
	update_fps()
	update_entity_count()
	update_chunk_progress()

# Update world name display
func update_world_name() -> void:
	if not world_name_label:
		return

	# Try to get current map name from ChunkManager or use default
	var map_name = "🌍 Life Simulator"
	
	# Check if we have recent world info
	if ChunkManager:
		# You could store the last loaded world name in ChunkManager
		# For now, we'll use a default that indicates auto-loading
		map_name = "🗺️ Latest Map"
	
	world_name_label.text = map_name

# Update FPS display
func update_fps() -> void:
	if not fps_label:
		return

	var fps = Engine.get_frames_per_second()
	fps_label.text = "FPS: " + str(fps)

	# Color code FPS for visual feedback
	if fps >= 50:
		fps_label.modulate = Color(0.3, 1.0, 0.3)  # Green for good FPS
	elif fps >= 30:
		fps_label.modulate = Color(1.0, 0.8, 0.0)  # Yellow for medium FPS
	else:
		fps_label.modulate = Color(1.0, 0.3, 0.3)  # Red for low FPS

# Update entity count display
func update_entity_count() -> void:
	if not entity_label:
		return

	# TODO: Get entity count from backend API
	entity_label.text = "🐾 0"

# Update chunk progress display
func update_chunk_progress() -> void:
	if not chunk_label:
		return

	var loaded_chunks = 0
	var total_chunks = 0

	if ChunkManager:
		loaded_chunks = ChunkManager.get_loaded_chunk_count()
		total_chunks = ChunkManager.get_total_chunk_count()

	chunk_label.text = "📦 " + str(loaded_chunks) + "/" + str(total_chunks)

	# Color code based on loading progress
	if total_chunks > 0:
		var progress = float(loaded_chunks) / float(total_chunks)
		if progress >= 1.0:
			chunk_label.modulate = Color(0.3, 1.0, 0.3)  # Green when fully loaded
		elif progress >= 0.5:
			chunk_label.modulate = Color(1.0, 0.8, 0.0)  # Yellow when half loaded
		else:
			chunk_label.modulate = Color(1.0, 0.5, 0.3)  # Orange when loading

# Set reference to WorldRenderer
func set_world_renderer(renderer: Node2D) -> void:
	world_renderer = renderer
	print("✅ TopBar: WorldRenderer reference set")

# Set reference to StatisticsHUD
func set_statistics_hud(hud) -> void:
	statistics_hud = hud
	print("✅ TopBar: StatisticsHUD reference set")

# Set reference to ControlsOverlay
func set_controls_overlay(overlay) -> void:
	controls_overlay = overlay
	print("✅ TopBar: ControlsOverlay reference set")

# Set reference to HeightMarkerOverlay
func set_height_marker_overlay(overlay) -> void:
	height_marker_overlay = overlay
	print("✅ TopBar: HeightMarkerOverlay reference set")

# Button Action Handlers

func _on_grid_button_pressed() -> void:
	print("🔲 TopBar: Grid toggle button pressed")
	if world_renderer and world_renderer.grid_overlay:
		world_renderer.grid_overlay.toggle_grid()
	else:
		print("⚠️ TopBar: Grid overlay not available")

func _on_camera_reset_pressed() -> void:
	print("📹 TopBar: Camera reset button pressed")
	if world_renderer:
		world_renderer.reset_camera_to_origin()
	else:
		print("⚠️ TopBar: WorldRenderer not available")

func _on_stats_toggle_pressed() -> void:
	print("📊 TopBar: Stats toggle button pressed")
	if statistics_hud:
		statistics_hud._on_toggle_pressed()
	else:
		print("⚠️ TopBar: StatisticsHUD not available")

func _on_help_toggle_pressed() -> void:
	print("❓ TopBar: Help toggle button pressed")
	if controls_overlay:
		controls_overlay._on_toggle_pressed()
	else:
		print("⚠️ TopBar: ControlsOverlay not available")

func _on_refresh_pressed() -> void:
	print("🔄 TopBar: Reload button pressed - reloading latest map")
	if world_renderer:
		world_renderer.reload_latest_map()
	else:
		print("⚠️ TopBar: WorldRenderer not available")

func _on_rotation_button_pressed() -> void:
	print("🔄 TopBar: Rotation test button pressed")
	# Cycle through rotations: 0 → 1 → 2 → 3 → 0
	Config.slope_rotation = (Config.slope_rotation + 1) % 4
	_update_rotation_button_text()

	# Force refresh to apply new rotation
	if world_renderer:
		print("   Rotation changed to: ", Config.slope_rotation, " (", _get_rotation_degrees(), "°)")
		world_renderer.force_refresh_chunks()
	else:
		print("⚠️ TopBar: WorldRenderer not available")

func _update_rotation_button_text() -> void:
	if rotation_button:
		rotation_button.text = "Rot: " + _get_rotation_degrees() + "°"
		rotation_button.tooltip_text = "Cycle Slope Rotation (Testing: " + _get_rotation_degrees() + "°)"

func _get_rotation_degrees() -> String:
	match Config.slope_rotation:
		0: return "0"
		1: return "90"
		2: return "180"
		3: return "270"
		_: return "?"

func _on_height_button_pressed() -> void:
	print("📏 TopBar: Height markers toggle button pressed")
	if height_marker_overlay:
		height_marker_overlay.toggle_visibility()
	else:
		print("⚠️ TopBar: HeightMarkerOverlay not available")

# Handle keyboard shortcuts (R for reset camera)
func _input(event: InputEvent) -> void:
	if event is InputEventKey and event.is_pressed():
		if event.keycode == KEY_R:
			_on_camera_reset_pressed()

# Public method to force statistics update
func force_update() -> void:
	update_statistics()
	time_since_update = 0.0
