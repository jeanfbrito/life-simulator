extends Camera2D
class_name CameraController

# Camera control settings
var drag_sensitivity: float = 1.0
var zoom_sensitivity: float = 0.1
var min_zoom: float = 0.2
var max_zoom: float = 5.0
var zoom_speed: float = 8.0

# Drag state
var is_dragging: bool = false
var drag_start_position: Vector2
var camera_start_position: Vector2

# Smooth zoom
var target_zoom: Vector2
var zoom_velocity: Vector2

# Keyboard movement
var keyboard_speed: float = 400.0
var keyboard_zoom_speed: float = 0.5

# Edge scrolling
var edge_scroll_enabled: bool = false
var edge_scroll_margin: float = 50.0
var edge_scroll_speed: float = 800.0

func _ready() -> void:
	# Initialize target zoom
	target_zoom = zoom
	
	# Enable process for smooth updates
	set_process(true)
	set_process_input(true)
	set_process_unhandled_input(true)

func _process(delta: float) -> void:
	# Smooth zoom interpolation
	if zoom != target_zoom:
		zoom = zoom.lerp(target_zoom, zoom_speed * delta)
	
	# Handle keyboard movement
	var move_direction = Vector2.ZERO
	
	# Arrow keys or WASD
	if Input.is_key_pressed(KEY_LEFT) or Input.is_key_pressed(KEY_A):
		move_direction.x -= 1.0
	if Input.is_key_pressed(KEY_RIGHT) or Input.is_key_pressed(KEY_D):
		move_direction.x += 1.0
	if Input.is_key_pressed(KEY_UP) or Input.is_key_pressed(KEY_W):
		move_direction.y -= 1.0
	if Input.is_key_pressed(KEY_DOWN) or Input.is_key_pressed(KEY_S):
		move_direction.y += 1.0
	
	# Apply keyboard movement
	if move_direction.length() > 0:
		move_direction = move_direction.normalized()
		position += move_direction * keyboard_speed * delta / zoom.x
	
	# Edge scrolling
	if edge_scroll_enabled:
		var mouse_pos = get_local_mouse_position()
		var viewport_size = get_viewport().get_visible_rect().size
		var edge_move = Vector2.ZERO
		
		if mouse_pos.x < edge_scroll_margin:
			edge_move.x -= 1.0
		elif mouse_pos.x > viewport_size.x - edge_scroll_margin:
			edge_move.x += 1.0
			
		if mouse_pos.y < edge_scroll_margin:
			edge_move.y -= 1.0
		elif mouse_pos.y > viewport_size.y - edge_scroll_margin:
			edge_move.y += 1.0
		
		if edge_move.length() > 0:
			edge_move = edge_move.normalized()
			position += edge_move * edge_scroll_speed * delta / zoom.x

func _unhandled_input(event: InputEvent) -> void:
	# Mouse wheel zoom
	if event is InputEventMouseButton:
		var mouse_event = event as InputEventMouseButton

		if mouse_event.is_pressed():
			match mouse_event.button_index:
				MOUSE_BUTTON_WHEEL_UP:
					_zoom_at_screen_center(-zoom_sensitivity)
				MOUSE_BUTTON_WHEEL_DOWN:
					_zoom_at_screen_center(zoom_sensitivity)
				MOUSE_BUTTON_MIDDLE:
					# Start middle mouse drag
					is_dragging = true
					drag_start_position = mouse_event.position
					camera_start_position = position
				MOUSE_BUTTON_LEFT:
					# Start drag with Cmd+Click (macOS) or Space+Click
					if mouse_event.is_command_or_control_pressed() or Input.is_key_pressed(KEY_SPACE):
						is_dragging = true
						drag_start_position = mouse_event.position
						camera_start_position = position
		else:
			if mouse_event.button_index == MOUSE_BUTTON_MIDDLE:
				# End middle mouse drag
				is_dragging = false
			elif mouse_event.button_index == MOUSE_BUTTON_LEFT and is_dragging:
				# End Cmd+Click or Space+Click drag
				is_dragging = false

	# Mouse drag
	elif event is InputEventMouseMotion and is_dragging:
		var mouse_event = event as InputEventMouseMotion
		var drag_delta = (mouse_event.position - drag_start_position) * drag_sensitivity
		position = camera_start_position - drag_delta / zoom.x
	
	# Keyboard zoom
	elif event is InputEventKey:
		var key_event = event as InputEventKey
		if key_event.is_pressed():
			match key_event.keycode:
				KEY_EQUAL, KEY_KP_ADD:
					_zoom_at_screen_center(-keyboard_zoom_speed)
				KEY_MINUS, KEY_KP_SUBTRACT:
					_zoom_at_screen_center(keyboard_zoom_speed)

func _zoom_at_mouse_position(zoom_delta: float) -> void:
	var mouse_pos = get_global_mouse_position()
	var old_zoom = zoom.x
	
	# Calculate new zoom
	var new_zoom_x = clamp(old_zoom + zoom_delta, min_zoom, max_zoom)
	var new_zoom = Vector2(new_zoom_x, new_zoom_x)
	
	# Adjust position to zoom towards mouse position
	var zoom_ratio = new_zoom_x / old_zoom
	position = mouse_pos + (position - mouse_pos) * zoom_ratio
	
	target_zoom = new_zoom

func _zoom_at_screen_center(zoom_delta: float) -> void:
	var old_zoom = zoom.x
	var new_zoom_x = clamp(old_zoom + zoom_delta, min_zoom, max_zoom)
	target_zoom = Vector2(new_zoom_x, new_zoom_x)

# Public methods for programmatic control
func set_position_smooth(target_pos: Vector2, duration: float = 0.5) -> void:
	var tween = create_tween()
	tween.tween_property(self, "position", target_pos, duration).set_trans(Tween.TRANS_SINE).set_ease(Tween.EASE_OUT)

func set_zoom_smooth(target_zoom_level: float, duration: float = 0.3) -> void:
	var clamped_zoom = clamp(target_zoom_level, min_zoom, max_zoom)
	target_zoom = Vector2(clamped_zoom, clamped_zoom)

func focus_on_position(world_pos: Vector2, zoom_level: float = 1.0) -> void:
	position = world_pos
	set_zoom_smooth(zoom_level, 0.5)

func reset_camera() -> void:
	position = Vector2.ZERO
	set_zoom_smooth(1.0, 0.5)