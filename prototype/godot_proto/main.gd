extends Node2D
# Milestone 01 prototype harness. Everything is built in code so there is nothing to click through.
# Sections are measured SEPARATELY:  [1] integration  [2] simulation (inside Rust)  [3] rendering (Godot only).
# Results are printed to the on-screen label, stdout (adb logcat -s godot) and copied with "Copy".

var core: Object
var label: Label
var cam: Camera2D
var mmi: MultiMeshInstance2D
var log_lines: PackedStringArray = []
var measuring := false
var frame_times: Array[float] = []
var measure_target := 0
var measure_name := ""

func _ready() -> void:
    cam = Camera2D.new(); add_child(cam)
    var ui := CanvasLayer.new(); add_child(ui)
    var box := VBoxContainer.new(); box.position = Vector2(8, 8); ui.add_child(box)
    label = Label.new(); label.custom_minimum_size = Vector2(1100, 300); box.add_child(label)
    var row := HFlowContainer.new(); box.add_child(row)
    _btn(row, "1 Integration", _test_integration)
    _btn(row, "2 Sim bench", _test_sim)
    for n in [10000, 100000, 300000, 1000000]:
        _btn(row, "3 Render %d" % n, _test_render.bind(n))
    _btn(row, "Copy", func(): DisplayServer.clipboard_set("\n".join(log_lines)))
    _log("Platform M01 prototype. Godot %s, renderer %s, device %s" % [Engine.get_version_info().string, RenderingServer.get_video_adapter_name(), OS.get_model_name()])
    if ClassDB.class_exists("CoreProbe"):
        core = ClassDB.instantiate("CoreProbe")
        _log("CoreProbe class FOUND (extension loaded)")
    else:
        _log("CoreProbe class MISSING: Rust library not loaded. Check bin/ path and .gdextension.")

func _btn(parent: Node, text: String, cb: Callable) -> void:
    var b := Button.new(); b.text = text; b.pressed.connect(cb); parent.add_child(b)

func _log(s: String) -> void:
    print(s); log_lines.append(s); label.text = "\n".join(log_lines.slice(maxi(0, log_lines.size() - 14)))

func _test_integration() -> void:
    if core == null: _log("no core"); return
    _log("info: " + core.core_info())
    var n := 200000
    var t0 := Time.get_ticks_usec()
    for i in n: core.ping()
    var us := Time.get_ticks_usec() - t0
    _log("call overhead: %.0f ns/call (%d calls, includes GDScript loop)" % [us * 1000.0 / n, n])
    for seed in [1, 2, 3]:
        _log("determinism seed=%d hash=%s" % [seed, core.determinism_hash(seed)])

func _test_sim() -> void:
    if core == null: _log("no core"); return
    # runs in the main thread on purpose for the prototype; UI will freeze for a few seconds
    for cfg in [[10000, 4, 1e-4], [100000, 4, 1e-4], [1000000, 4, 1e-4], [1000000, 4, 1e-2]]:
        _log("sim " + core.run_sim_bench(cfg[0], cfg[1], cfg[2]))

func _test_render(n: int) -> void:
    if mmi: mmi.queue_free()
    var t0 := Time.get_ticks_usec()
    var mm := MultiMesh.new()
    mm.transform_format = MultiMesh.TRANSFORM_2D
    var quad := QuadMesh.new(); quad.size = Vector2(3, 3); mm.mesh = quad
    mm.instance_count = n
    var side := int(ceil(sqrt(n)))
    for i in n:
        mm.set_instance_transform_2d(i, Transform2D(0.0, Vector2((i % side) * 4.0, (i / side) * 4.0)))
    mmi = MultiMeshInstance2D.new(); mmi.multimesh = mm; add_child(mmi)
    var setup_ms := (Time.get_ticks_usec() - t0) / 1000.0
    cam.position = Vector2(side * 2.0, side * 2.0); cam.zoom = Vector2.ONE * minf(1.0, 700.0 / (side * 4.0))
    _log("render n=%d (all in view) setup=%.0f ms; measuring 180 frames..." % [n, setup_ms])
    frame_times.clear(); measure_target = 180; measure_name = "render n=%d" % n; measuring = true

func _process(delta: float) -> void:
    if not measuring: return
    frame_times.append(delta * 1000.0)
    if frame_times.size() >= measure_target:
        measuring = false
        var a := frame_times.duplicate(); a.sort()
        var avg := 0.0
        for x in a: avg += x
        avg /= a.size()
        _log("%s: avg %.2f ms, p95 %.2f ms, worst %.2f ms (VISIBLE=instances in view; sim NOT running)" % [measure_name, avg, a[int(a.size() * 0.95)], a[-1]])

var _drag := false
func _input(e: InputEvent) -> void:
    if e is InputEventScreenDrag: cam.position -= e.relative / cam.zoom
    elif e is InputEventMouseMotion and Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT): cam.position -= e.relative / cam.zoom
