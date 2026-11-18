extends Node2D
# Milestone 01b prototype harness (v2). Everything is built in code.
# Buttons: 1 Integration | 2 Sim bench | 3 Render N | 4 Snapshot cost | 5a/5b Soak (sim + render) | 5c Overload (sim cannot keep up) | Stop | Copy
# Sim runs on a Rust WORKER THREAD; Godot only reads its counters. Sim and render are reported separately.

const SOAK_WINDOW_MS := 30000

var core = null
var label: Label
var cam: Camera2D
var mmi: MultiMeshInstance2D
var log_lines: PackedStringArray = []
var measuring := false
var frame_times: Array[float] = []
var measure_target := 0
var measure_name := ""
var soak_on := false
var soak_len_s := 600.0
var soak_t0_ms := 0
var soak_win_start_ms := 0
var soak_win: Array[float] = []
var soak_base_zoom := 1.0
var soak_center := Vector2.ZERO
var soak_side := 1.0

func _ready() -> void:
    cam = Camera2D.new(); add_child(cam)
    var ui := CanvasLayer.new(); add_child(ui)
    var box := VBoxContainer.new(); box.position = Vector2(8, 8); box.custom_minimum_size = Vector2(1260, 0); ui.add_child(box)
    # buttons FIRST (always reachable), log below; long lines wrap instead of widening the screen
    var row := HFlowContainer.new(); row.custom_minimum_size = Vector2(1260, 0); box.add_child(row)
    _btn(row, "1 Integration", _test_integration)
    _btn(row, "2 Sim bench", _test_sim)
    for n in [10000, 100000, 300000, 1000000]:
        _btn(row, "3 Render %d" % n, _test_render.bind(n))
    _btn(row, "4 Snapshot cost", _test_snapshot)
    _btn(row, "5a Soak 1 min", _start_soak.bind(60.0, 0.9))
    _btn(row, "5b Soak 10 min", _start_soak.bind(600.0, 0.9))
    _btn(row, "5c Overload 1 min", _start_soak.bind(60.0, 0.99))
    _btn(row, "Stop", _stop_soak)
    _btn(row, "Copy", func(): DisplayServer.clipboard_set("\n".join(log_lines)))
    label = Label.new()
    label.custom_minimum_size = Vector2(1260, 0)
    label.autowrap_mode = TextServer.AUTOWRAP_ARBITRARY
    label.add_theme_font_size_override("font_size", 16)
    box.add_child(label)
    _log("Platform M01b prototype v3. Godot %s, renderer %s, device %s" % [Engine.get_version_info().string, RenderingServer.get_video_adapter_name(), OS.get_model_name()])
    if ClassDB.class_exists("CoreProbe"):
        core = ClassDB.instantiate("CoreProbe")
        add_child(core)
        _log("CoreProbe class FOUND (extension loaded)")
    else:
        _log("CoreProbe class MISSING: Rust library not loaded. Check bin/ path and .gdextension.")
    _log("The whole log is copied to the clipboard automatically after every line; paste it into any notes app.")

func _btn(parent: Node, text: String, cb: Callable) -> void:
    var b := Button.new(); b.text = text; b.pressed.connect(cb); parent.add_child(b)

func _log(s: String) -> void:
    print(s); log_lines.append(s)
    label.text = "\n".join(log_lines.slice(maxi(0, log_lines.size() - 12)))
    DisplayServer.clipboard_set("\n".join(log_lines))

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
    # runs on the main thread on purpose (old test); the UI freezes for a few seconds
    for cfg in [[10000, 4, 1e-4], [100000, 4, 1e-4], [1000000, 4, 1e-4]]:
        _log("sim " + core.run_sim_bench(cfg[0], cfg[1], cfg[2]))

func _build_render(n: int) -> float:
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
    soak_side = side * 4.0
    soak_center = Vector2(side * 2.0, side * 2.0)
    soak_base_zoom = minf(1.0, 700.0 / (side * 4.0))
    cam.position = soak_center; cam.zoom = Vector2.ONE * soak_base_zoom
    return (Time.get_ticks_usec() - t0) / 1000.0

func _test_render(n: int) -> void:
    var setup_ms := _build_render(n)
    _log("render n=%d (all in view) setup=%.0f ms; measuring 180 frames..." % [n, setup_ms])
    frame_times.clear(); measure_target = 180; measure_name = "render n=%d" % n; measuring = true

func _time_snapshot(n: int) -> String:
    var reps := 30
    var t0 := Time.get_ticks_usec()
    var total := 0
    for i in reps:
        var a: PackedVector2Array = core.make_positions(n, 4.0)
        total += a.size()
    var us := Time.get_ticks_usec() - t0
    return "%.3f ms per call (%d points each)" % [us / 1000.0 / reps, total / reps]

func _test_snapshot() -> void:
    if core == null: _log("no core"); return
    for n in [10000, 100000]:
        _log("snapshot n=%d, sim idle: %s" % [n, _time_snapshot(n)])
    _log(core.sim_start(2, 250000, 0.9, 10000))
    await get_tree().create_timer(2.0).timeout
    for n in [10000, 100000]:
        _log("snapshot n=%d, sim running: %s" % [n, _time_snapshot(n)])
    _log("sim: " + core.sim_stats())
    core.sim_stop()

func _start_soak(seconds: float, p_hot: float) -> void:
    if core == null: _log("no core"); return
    var setup_ms := _build_render(100000)
    soak_len_s = seconds
    _log("SOAK %d s: 100k instances drawn, camera pans/zooms by itself, sim = timing wheel, 250k slots (~1M particles), p_hot %.2f, 10000x on a worker thread. render setup %.0f ms" % [int(seconds), p_hot, setup_ms])
    _log(core.sim_start(2, 250000, p_hot, 10000))
    core.sim_stats()   # primes the rate counters
    soak_win.clear()
    soak_t0_ms = Time.get_ticks_msec(); soak_win_start_ms = soak_t0_ms
    soak_on = true

func _stop_soak() -> void:
    if core != null: core.sim_stop()
    if soak_on: _log("soak stopped")
    soak_on = false

func _log_soak_window(elapsed_s: int) -> void:
    var a := soak_win.duplicate(); a.sort()
    var avg := 0.0
    for x in a: avg += x
    avg /= maxf(1.0, a.size())
    var p95 = a[int(a.size() * 0.95)] if a.size() > 0 else 0.0
    var worst = a[-1] if a.size() > 0 else 0.0
    _log("[%4d s] render avg %.2f ms, p95 %.2f, worst %.1f (%d frames)" % [elapsed_s, avg, p95, worst, a.size()])
    _log("        sim: %s" % core.sim_stats())
    _log("        memory: process rss %d MB, godot static %d MB" % [core.rss_kb() / 1024, OS.get_static_memory_usage() / 1048576])

func _process(delta: float) -> void:
    if soak_on:
        var now_ms := Time.get_ticks_msec()
        var t := (now_ms - soak_t0_ms) / 1000.0
        cam.position = soak_center + Vector2(sin(t * 0.5), cos(t * 0.37)) * soak_side * 0.3
        cam.zoom = Vector2.ONE * soak_base_zoom * (1.0 + 0.4 * sin(t * 0.3))
        soak_win.append(delta * 1000.0)
        if now_ms - soak_win_start_ms >= SOAK_WINDOW_MS:
            _log_soak_window(int(t))
            soak_win.clear(); soak_win_start_ms = now_ms
        if t >= soak_len_s:
            _log("soak finished")
            _stop_soak()
        return
    if not measuring: return
    frame_times.append(delta * 1000.0)
    if frame_times.size() >= measure_target:
        measuring = false
        var a := frame_times.duplicate(); a.sort()
        var avg := 0.0
        for x in a: avg += x
        avg /= a.size()
        _log("%s: avg %.2f ms, p95 %.2f ms, worst %.2f ms (sim NOT running)" % [measure_name, avg, a[int(a.size() * 0.95)], a[-1]])

func _input(e: InputEvent) -> void:
    var d := e as InputEventScreenDrag
    if d != null:
        cam.position -= d.relative / cam.zoom
        return
    var m := e as InputEventMouseMotion
    if m != null and Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
        cam.position -= m.relative / cam.zoom
