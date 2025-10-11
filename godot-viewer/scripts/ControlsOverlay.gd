extends Control
class_name ControlsOverlay

@onready var controls_panel: Panel = $ControlsPanel
@onready var controls_text: RichTextLabel = $ControlsPanel/ControlsText
@onready var toggle_button: Button = $ToggleButton

var is_visible: bool = true

func _ready() -> void:
	# Setup the controls text
	update_controls_text()
	
	# Connect toggle button
	toggle_button.pressed.connect(_on_toggle_pressed)
	
	# Position the panel
	controls_panel.position = Vector2(10, 10)
	toggle_button.position = Vector2(10, controls_panel.size.y + 15)

func update_controls_text() -> void:
	var controls = """
[b]Camera Controls[/b]

🖱️ [b]Mouse Wheel[/b] - Zoom in/out
🖱️ [b]Middle Mouse + Drag[/b] - Pan camera
🖱️ [b]Edge Scrolling[/b] - Move to edges

⌨️ [b]Arrow Keys / WASD[/b] - Move camera
⌨️ [+/- Keys[/b] - Zoom in/out

[b]Settings[/b]
• Zoom Range: 0.2x - 5.0x
• Edge Scrolling: Enabled
• Smooth Transitions: Yes
"""
	controls_text.text = controls

func _on_toggle_pressed() -> void:
	is_visible = !is_visible
	controls_panel.visible = is_visible
	
	if is_visible:
		toggle_button.text = "Hide Controls (H)"
	else:
		toggle_button.text = "Show Controls (H)"

func _input(event: InputEvent) -> void:
	# Toggle with H key
	if event is InputEventKey and event.is_pressed():
		if event.keycode == KEY_H:
			_on_toggle_pressed()