extends Node

func _ready():
	print("=== HTTP Connection Test ===")
	print()

	# Test basic HTTP request
	print("Testing basic HTTP request...")
	var http_request = HTTPRequest.new()
	add_child(http_request)

	var error = http_request.request("http://127.0.0.1:54321/api/world/current")
	if error != OK:
		print("❌ Failed to start request: ", error)
		get_tree().quit()
		return

	print("Request started, waiting for response...")

	# Wait for response
	var result = await http_request.request_completed
	print("Request completed with result: ", result[0], " code: ", result[1])

	if result[0] == HTTPRequest.RESULT_SUCCESS and result[1] == 200:
		var body = result[3]
		var json = JSON.new()
		var parse_result = json.parse(body.get_string_from_utf8())
		if parse_result == OK:
			var data = json.data
			print("✅ Successfully parsed JSON")
			print("World name: ", data.get("name", "Unknown"))
		else:
			print("❌ Failed to parse JSON")
	else:
		print("❌ Request failed")

	http_request.queue_free()
	print("=== HTTP Test Complete ===")
	get_tree().quit()