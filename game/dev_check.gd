extends Control
# Developer checks (Milestone 02a tests, opened from the shell Settings). NOT the game. Everything is built in code.
# Buttons: 1 Info | 2 Run | 3 Pause/Resume | 4 Overload (governor) | 5 Save/Load | Stop | Copy
# The whole log is copied to the clipboard after every line.

signal closed
var ui_scale := 1.0
var inset := Vector2.ZERO
var W := 1260.0
var core = null
var label: Label
var live: Label
var log_lines: PackedStringArray = []
var busy := false
var paused := false
var poll_t := 0.0
var fields: PackedStringArray = []

func _ready() -> void:
    set_anchors_preset(Control.PRESET_FULL_RECT)
    W = maxf(300.0, get_viewport_rect().size.x - 16.0 - inset.x * 2.0)
    var box := VBoxContainer.new()
    box.position = Vector2(8, 8) + inset
    box.custom_minimum_size = Vector2(W, 0)
    add_child(box)
    var row := HFlowContainer.new()
    row.custom_minimum_size = Vector2(W, 0)
    box.add_child(row)
    _btn(row, "1 Info", _test_info)
    _btn(row, "2 Run 1000x", _test_run)
    _btn(row, "3 Pause/Resume", _test_pause)
    _btn(row, "4 Overload", _test_overload)
    _btn(row, "5 Save/Load", _test_save)
    _btn(row, "Stop", _test_stop)
    _btn(row, "Close", _close)
    _btn(row, "Copy", func(): DisplayServer.clipboard_set("\n".join(log_lines)))
    live = Label.new()
    live.custom_minimum_size = Vector2(W, 0)
    live.autowrap_mode = TextServer.AUTOWRAP_ARBITRARY
    live.add_theme_font_size_override("font_size", int(14 * ui_scale))
    box.add_child(live)
    label = Label.new()
    label.custom_minimum_size = Vector2(W, 0)
    label.autowrap_mode = TextServer.AUTOWRAP_ARBITRARY
    label.add_theme_font_size_override("font_size", int(14 * ui_scale))
    box.add_child(label)
    _log("Platform M02a check. Godot %s, device %s" % [Engine.get_version_info().string, OS.get_model_name()])
    if ClassDB.class_exists("PlatformCore"):
        core = ClassDB.instantiate("PlatformCore")
        add_child(core)
        fields = String(core.stats_fields()).split(",")
        _log("PlatformCore FOUND (extension loaded)")
    else:
        _log("PlatformCore MISSING: Rust library not loaded. Check bin/ path and the .gdextension file.")

func _close() -> void:
    _test_stop()
    closed.emit()

func _btn(parent: Node, text: String, cb: Callable) -> void:
    var b := Button.new()
    b.text = text
    b.pressed.connect(cb)
    parent.add_child(b)

func _log(s: String) -> void:
    print(s)
    log_lines.append(s)
    label.text = "\n".join(log_lines.slice(maxi(0, log_lines.size() - 14)))
    DisplayServer.clipboard_set("\n".join(log_lines))

func _stat(field: String) -> int:
    var a: PackedInt64Array = core.stats()
    var i := fields.find(field)
    return a[i] if i >= 0 and i < a.size() else -1

func _line() -> String:
    return "tick=%d req=%d eff=%d paused=%d owed=%d drops=%d raises=%d busy=%.1f%%" % [
        _stat("tick"), _stat("requested_speed"), _stat("effective_speed"), _stat("paused"),
        _stat("owed_ticks"), _stat("governor_drops"), _stat("governor_raises"), _stat("busy_permille") / 10.0]

func _process(delta: float) -> void:
    poll_t += delta
    if poll_t >= 0.25 and core != null and core.running():
        poll_t = 0.0
        live.text = "LIVE " + _line()

func _test_info() -> void:
    if core == null: _log("no core"); return
    _log("info: " + core.core_version())
    _log("stat fields: " + core.stats_fields())

func _test_run() -> void:
    if core == null or busy: return
    busy = true
    core.start(12345, 1000, 0)
    _log("started at 1000x, no extra load; expect tick ~ 1000/s, no governor action")
    await get_tree().create_timer(5.0).timeout
    _log("after 5 s: " + _line())
    busy = false

func _test_pause() -> void:
    if core == null or not core.running(): _log("start a run first (button 2)"); return
    paused = not paused
    core.set_paused(paused)
    _log("paused=%s" % str(paused))
    await get_tree().create_timer(1.0).timeout
    var t1 := _stat("tick")
    await get_tree().create_timer(1.0).timeout
    _log("1 s apart: tick %d -> %d (%s)" % [t1, _stat("tick"), "must be equal while paused" if paused else "must grow"])

func _test_overload() -> void:
    if core == null or busy: return
    busy = true
    core.start(12345, 10000, 1000000)
    _log("OVERLOAD: request 10000x but every tick costs 1 ms (device can do <= ~1000 ticks/s). Expect effective speed to fall to ~900 and stay near it.")
    for i in 12:
        await get_tree().create_timer(1.0).timeout
        _log("t=%2ds %s" % [i + 1, _line()])
    core.set_burn_ns(0)
    _log("load removed. Expect effective speed to climb back toward 10000.")
    for i in 25:
        await get_tree().create_timer(1.0).timeout
        if i % 3 == 2: _log("t=%2ds %s" % [i + 1, _line()])
    busy = false

func _test_save() -> void:
    if core == null or not core.running(): _log("start a run first (button 2)"); return
    var path := ProjectSettings.globalize_path("user://m02a_test.sav")
    _log(core.save_check(path, "m02a check"))
    await get_tree().create_timer(0.5).timeout
    _log(core.save_check(path, "m02a check 2"))
    _log("backup file exists: %s" % str(FileAccess.file_exists("user://m02a_test.bak")))
    _log(core.load_check(path))

func _test_stop() -> void:
    if core != null: core.stop()
    busy = false
    live.text = ""
    _log("stopped")
