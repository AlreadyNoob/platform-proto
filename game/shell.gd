extends Control
# Platform game shell (Milestone 02b). Presentation only: it reads core stats and sends commands
# (set_speed, set_paused). No game rules exist yet; the world view shows only a pan/zoom grid.

const WorldViewScript := preload("res://world_view.gd")
const SPEEDS: Array[int] = [1, 10, 100, 1000, 10000]
const SPEED_LABELS: Array[String] = ["1x", "10x", "100x", "1k", "10k"]
const SETTINGS_PATH := "user://settings.cfg"
const DEV_SEED := 12345   # PROVISIONAL: placeholder seed until a real world exists

var U := 1.0                 # UI scale (dp-like), from screen DPI
var dark := false
var show_debug := false
var portrait := false
var core = null
var idx := {}
var st: PackedInt64Array = PackedInt64Array()
var poll_t := 0.0
var eff_smooth := -1.0
var limited := false
var limit_t := 0.0
var last_req := -1
var p := {}

var margin: MarginContainer
var bar_margin: MarginContainer
var cutouts_seen := 0
var inset := Vector4.ZERO   # safe-area insets: left, top, right, bottom (px)
var view = null
var status: Label
var debug_label: Label
var pause_btn: Button
var speed_btns: Array[Button] = []
var menu_btn: Button
var settings_layer: Control
var dark_btn: Button
var debug_btn: Button
var portrait_btn: Button
var dev_layer: Control = null
const SLOT_NAMES: Array[String] = ["autosave", "slot1", "slot2", "slot3"]
const SLOT_TITLES: Array[String] = ["Autosave", "Slot 1", "Slot 2", "Slot 3"]
var saves_layer: Control
var saves_title: Label
var slot_labels := {}
var bg_paused := false

func _ready() -> void:
    get_tree().set_auto_accept_quit(false)
    get_tree().set_quit_on_go_back(false)   # v0.1.8: otherwise one Back press also quits the app
    # v0.1.6: force 1:1 pixels regardless of project/editor settings (the v0.1.5 APK showed a
    # 1280x720 viewport letterboxed inside a 1080x2436 window).
    var win: Window = get_window()
    win.content_scale_mode = Window.CONTENT_SCALE_MODE_DISABLED
    win.content_scale_aspect = Window.CONTENT_SCALE_ASPECT_EXPAND
    win.content_scale_size = Vector2i.ZERO
    win.content_scale_factor = 1.0
    Engine.max_fps = 0
    DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_ENABLED)
    var dpi: int = DisplayServer.screen_get_dpi()
    U = clampf(float(dpi) / 160.0, 1.0, 4.0)
    _load_settings()
    _apply_orientation()
    _ensure_audio_buses()
    p = _palette(dark)
    theme = _build_theme()
    _build_ui()
    _apply_palette()
    get_viewport().size_changed.connect(_apply_safe_area)
    _apply_safe_area()
    _start_core()

# ---------- audio foundation (buses only; no sounds exist yet) ----------
func _ensure_audio_buses() -> void:
    for bus_name in ["Music", "Sfx"]:
        if AudioServer.get_bus_index(bus_name) == -1:
            AudioServer.add_bus()
            var i: int = AudioServer.bus_count - 1
            AudioServer.set_bus_name(i, bus_name)
            AudioServer.set_bus_send(i, "Master")

# ---------- settings ----------
func _load_settings() -> void:
    var cf := ConfigFile.new()
    if cf.load(SETTINGS_PATH) == OK:
        dark = bool(cf.get_value("ui", "dark", false))
        show_debug = bool(cf.get_value("ui", "debug", false))
        portrait = bool(cf.get_value("ui", "portrait", false))

func _save_settings() -> void:
    var cf := ConfigFile.new()
    cf.set_value("ui", "dark", dark)
    cf.set_value("ui", "debug", show_debug)
    cf.set_value("ui", "portrait", portrait)
    cf.save(SETTINGS_PATH)

# ---------- theme tokens ----------
func _palette(is_dark: bool) -> Dictionary:
    if is_dark:
        return {"bg": Color("0e0f12"), "minor": Color("1a1c21"), "major": Color("2a2d35"),
            "text": Color("e8e8ec"), "dim": Color("8d909a"), "panel": Color("16181d"),
            "border": Color("2a2d35"), "button": Color("1e2127"), "hover": Color("272b33"),
            "accent": Color("6fa3d8"), "accent_text": Color("0e0f12"), "warn": Color("e0a050")}
    return {"bg": Color("f7f6f2"), "minor": Color("e3e1da"), "major": Color("cfcbc0"),
        "text": Color("1b1b1f"), "dim": Color("6a6a72"), "panel": Color("ffffff"),
        "border": Color("d6d3ca"), "button": Color("efede6"), "hover": Color("e6e3da"),
        "accent": Color("2f5f8f"), "accent_text": Color("ffffff"), "warn": Color("b5651d")}

func _sb(fill: Color, border: Color) -> StyleBoxFlat:
    var s := StyleBoxFlat.new()
    s.bg_color = fill
    s.border_color = border
    s.set_border_width_all(maxi(1, int(U * 0.7)))
    s.set_corner_radius_all(int(6 * U))
    s.content_margin_left = 7 * U
    s.content_margin_right = 7 * U
    s.content_margin_top = 6 * U
    s.content_margin_bottom = 6 * U
    return s

func _build_theme() -> Theme:
    var t := Theme.new()
    t.default_font_size = int(16 * U)
    t.set_color("font_color", "Label", p["text"])
    t.set_stylebox("panel", "PanelContainer", _sb(p["panel"], p["border"]))
    t.set_stylebox("normal", "Button", _sb(p["button"], p["border"]))
    t.set_stylebox("hover", "Button", _sb(p["hover"], p["border"]))
    t.set_stylebox("pressed", "Button", _sb(p["accent"], p["accent"]))
    t.set_stylebox("hover_pressed", "Button", _sb(p["accent"], p["accent"]))
    t.set_stylebox("focus", "Button", StyleBoxEmpty.new())
    t.set_stylebox("disabled", "Button", _sb(p["button"], p["border"]))
    t.set_color("font_color", "Button", p["text"])
    t.set_color("font_hover_color", "Button", p["text"])
    t.set_color("font_pressed_color", "Button", p["accent_text"])
    t.set_color("font_hover_pressed_color", "Button", p["accent_text"])
    t.set_color("font_focus_color", "Button", p["text"])
    t.set_color("font_disabled_color", "Button", p["dim"])
    return t

func _apply_palette() -> void:
    p = _palette(dark)
    theme = _build_theme()
    RenderingServer.set_default_clear_color(p["bg"])
    if view != null:
        view.call("set_colors", p["bg"], p["minor"], p["major"])
    if debug_label != null:
        debug_label.add_theme_color_override("font_color", p["dim"])
    _refresh_status()

# ---------- layout ----------
func _build_ui() -> void:
    margin = MarginContainer.new()
    margin.set_anchors_preset(Control.PRESET_FULL_RECT)
    add_child(margin)
    var col := VBoxContainer.new()
    col.add_theme_constant_override("separation", int(6 * U))
    margin.add_child(col)

    var bar := PanelContainer.new()
    col.add_child(bar)
    var bar_box := VBoxContainer.new()
    bar_box.add_theme_constant_override("separation", int(4 * U))
    bar_margin = MarginContainer.new()
    bar.add_child(bar_margin)
    bar_margin.add_child(bar_box)
    var row := HBoxContainer.new()
    row.add_theme_constant_override("separation", int(4 * U))
    bar_box.add_child(row)

    pause_btn = Button.new()
    pause_btn.text = "Pause"
    pause_btn.custom_minimum_size = Vector2(60 * U, 44 * U)
    pause_btn.pressed.connect(_on_pause)
    row.add_child(pause_btn)

    var group := ButtonGroup.new()
    for i in SPEEDS.size():
        var b := Button.new()
        b.text = SPEED_LABELS[i]
        b.toggle_mode = true
        b.button_group = group
        b.custom_minimum_size = Vector2(34 * U, 44 * U)
        b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
        b.pressed.connect(_on_speed.bind(i))
        row.add_child(b)
        speed_btns.append(b)
    speed_btns[0].set_pressed_no_signal(true)

    menu_btn = Button.new()
    menu_btn.text = "Menu"
    menu_btn.custom_minimum_size = Vector2(54 * U, 44 * U)
    menu_btn.pressed.connect(_open_settings)
    row.add_child(menu_btn)

    status = Label.new()
    bar_box.add_child(status)

    view = WorldViewScript.new()
    view.ui = U
    view.zoom = 24.0 * U
    view.zoom_max = 400.0 * U
    view.size_flags_vertical = Control.SIZE_EXPAND_FILL
    view.view_changed.connect(_refresh_debug)
    col.add_child(view)

    debug_label = Label.new()
    debug_label.position = Vector2(8 * U, 8 * U)
    debug_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
    debug_label.add_theme_font_size_override("font_size", int(13 * U))
    debug_label.visible = show_debug
    view.add_child(debug_label)

    _build_settings()
    _build_saves()

func _build_settings() -> void:
    settings_layer = Control.new()
    settings_layer.set_anchors_preset(Control.PRESET_FULL_RECT)
    settings_layer.visible = false
    add_child(settings_layer)
    var dim := ColorRect.new()
    dim.color = Color(0, 0, 0, 0.45)
    dim.set_anchors_preset(Control.PRESET_FULL_RECT)
    settings_layer.add_child(dim)
    var center := CenterContainer.new()
    center.set_anchors_preset(Control.PRESET_FULL_RECT)
    settings_layer.add_child(center)
    var panel := PanelContainer.new()
    center.add_child(panel)
    var box := VBoxContainer.new()
    box.custom_minimum_size = Vector2(280 * U, 0)
    box.add_theme_constant_override("separation", int(8 * U))
    panel.add_child(box)
    var title := Label.new()
    title.text = "Settings"
    box.add_child(title)
    dark_btn = _toggle(box, "Dark theme", dark, _on_dark)
    debug_btn = _toggle(box, "Debug overlay", show_debug, _on_debug)
    portrait_btn = _toggle(box, "Portrait mode (off = landscape)", portrait, _on_portrait)
    var saves_btn := Button.new()
    saves_btn.text = "Saves"
    saves_btn.custom_minimum_size = Vector2(0, 44 * U)
    saves_btn.pressed.connect(_open_saves)
    box.add_child(saves_btn)
    var dev := Button.new()
    dev.text = "Developer checks"
    dev.custom_minimum_size = Vector2(0, 44 * U)
    dev.pressed.connect(_open_dev_checks)
    box.add_child(dev)
    var close := Button.new()
    close.text = "Close"
    close.custom_minimum_size = Vector2(0, 44 * U)
    close.pressed.connect(_close_settings)
    box.add_child(close)

func _build_saves() -> void:
    saves_layer = Control.new()
    saves_layer.set_anchors_preset(Control.PRESET_FULL_RECT)
    saves_layer.visible = false
    add_child(saves_layer)
    var dim := ColorRect.new()
    dim.color = Color(0, 0, 0, 0.45)
    dim.set_anchors_preset(Control.PRESET_FULL_RECT)
    saves_layer.add_child(dim)
    var center := CenterContainer.new()
    center.set_anchors_preset(Control.PRESET_FULL_RECT)
    saves_layer.add_child(center)
    var panel := PanelContainer.new()
    center.add_child(panel)
    var box := VBoxContainer.new()
    box.add_theme_constant_override("separation", int(6 * U))
    panel.add_child(box)
    saves_title = Label.new()
    saves_title.text = "Saves"
    saves_title.custom_minimum_size = Vector2(320 * U, 0)
    saves_title.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
    box.add_child(saves_title)
    for i in SLOT_NAMES.size():
        var row := HBoxContainer.new()
        row.add_theme_constant_override("separation", int(6 * U))
        var lbl := Label.new()
        lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
        lbl.custom_minimum_size = Vector2(190 * U, 0)
        row.add_child(lbl)
        slot_labels[SLOT_NAMES[i]] = lbl
        if SLOT_NAMES[i] != "autosave":
            var sb := Button.new()
            sb.text = "Save"
            sb.custom_minimum_size = Vector2(64 * U, 40 * U)
            sb.pressed.connect(_save_slot.bind(SLOT_NAMES[i]))
            row.add_child(sb)
        var lb := Button.new()
        lb.text = "Load"
        lb.custom_minimum_size = Vector2(64 * U, 40 * U)
        lb.pressed.connect(_load_slot.bind(SLOT_NAMES[i]))
        row.add_child(lb)
        box.add_child(row)
    var close := Button.new()
    close.text = "Close"
    close.custom_minimum_size = Vector2(0, 40 * U)
    close.pressed.connect(_close_saves)
    box.add_child(close)

func _slot_path(slot: String) -> String:
    return ProjectSettings.globalize_path("user://%s.sav" % slot)

func _refresh_slots() -> void:
    for i in SLOT_NAMES.size():
        var s: String = "unavailable"
        if core != null:
            s = str(core.call("slot_summary", _slot_path(SLOT_NAMES[i])))
        slot_labels[SLOT_NAMES[i]].text = "%s: %s" % [SLOT_TITLES[i], s]

func _open_saves() -> void:
    _close_settings()
    saves_title.text = "Saves (tick and speed only; no game world yet)"
    _refresh_slots()
    saves_layer.visible = true
    view.input_enabled = false

func _close_saves() -> void:
    saves_layer.visible = false
    view.input_enabled = true

func _save_slot(slot: String) -> void:
    if core == null:
        return
    saves_title.text = str(core.call("save_game", _slot_path(slot), "manual " + slot))
    _refresh_slots()

func _load_slot(slot: String) -> void:
    if core == null:
        return
    var r: String = str(core.call("load_game", _slot_path(slot)))
    saves_title.text = r
    eff_smooth = -1.0
    limited = false
    last_req = -1
    bg_paused = false
    _refresh_slots()

func _toggle(parent: Node, text: String, on: bool, cb: Callable) -> Button:
    var b := Button.new()
    b.text = text
    b.toggle_mode = true
    b.custom_minimum_size = Vector2(0, 44 * U)
    b.set_pressed_no_signal(on)
    b.toggled.connect(cb)
    parent.add_child(b)
    return b

func _apply_safe_area() -> void:
    # v0.1.8: the grid and the bar background run edge to edge; only the controls stay
    # inside the safe area (notch / system bars).
    var l := 0.0
    var t := 0.0
    var r := 0.0
    var b := 0.0
    if OS.has_feature("android") or OS.has_feature("ios"):
        var sa: Rect2i = DisplayServer.get_display_safe_area()
        var scr: Vector2i = DisplayServer.screen_get_size()
        l = maxf(0.0, float(sa.position.x))
        t = maxf(0.0, float(sa.position.y))
        r = maxf(0.0, float(scr.x - sa.end.x))
        b = maxf(0.0, float(scr.y - sa.end.y))
    inset = Vector4(l, t, r, b)
    for side in ["left", "top", "right", "bottom"]:
        margin.add_theme_constant_override("margin_" + side, 0)
    # The bar sits along the top edge: only inset it where a display cutout (camera hole) really
    # overlaps the bar's band. A side inset caused by a hole in the middle of the long edge
    # (landscape) does not touch the bar.
    var bl := l
    var br := r
    var n_cut := 0
    if OS.has_feature("android") or OS.has_feature("ios"):
        var cuts: Array = DisplayServer.get_display_cutouts()
        n_cut = cuts.size()
        if n_cut > 0:
            var scr_w: float = float(DisplayServer.screen_get_size().x)
            var band: float = 100.0 * U + t
            var left_hit := 0.0
            var right_hit := 0.0
            for c in cuts:
                var cr: Rect2 = c
                if cr.position.y < band:
                    if cr.get_center().x < scr_w * 0.5:
                        left_hit = maxf(left_hit, cr.end.x)
                    else:
                        right_hit = maxf(right_hit, scr_w - cr.position.x)
            bl = minf(l, left_hit)
            br = minf(r, right_hit)
    cutouts_seen = n_cut
    bar_margin.add_theme_constant_override("margin_left", int(bl))
    bar_margin.add_theme_constant_override("margin_top", int(t))
    bar_margin.add_theme_constant_override("margin_right", int(br))
    if debug_label != null:
        debug_label.position = Vector2(8 * U + l, 8 * U)

# ---------- core ----------
func _start_core() -> void:
    if core != null:
        return
    if not ClassDB.class_exists("PlatformCore"):
        status.text = "Simulation core not loaded (Rust library missing)"
        for b in speed_btns:
            b.disabled = true
        pause_btn.disabled = true
        return
    core = ClassDB.instantiate("PlatformCore")
    add_child(core)
    var names: PackedStringArray = String(core.stats_fields()).split(",")
    for i in names.size():
        idx[names[i]] = i
    core.call("start", DEV_SEED, 1, 0)
    eff_smooth = -1.0
    limited = false
    last_req = -1

func _stop_core() -> void:
    if core != null:
        core.call("stop")
        core.queue_free()
        core = null

func _s(field: String) -> int:
    var i: int = idx.get(field, -1)
    if i >= 0 and i < st.size():
        return st[i]
    return 0

func _on_pause() -> void:
    if core == null:
        return
    core.call("set_paused", _s("paused") == 0)

func _on_speed(i: int) -> void:
    if core != null:
        core.call("set_speed", SPEEDS[i])

# ---------- status (smoothed "limited speed" indicator, K-34) ----------
func _fmt_speed(v: float) -> String:
    if v >= 10000.0:
        return "%dk" % int(round(v / 1000.0))
    if v >= 1000.0:
        var k: float = round(v / 100.0) / 10.0
        return ("%dk" % int(k)) if is_equal_approx(k, floorf(k)) else ("%.1fk" % k)
    var mag: float = pow(10.0, floorf(log(maxf(v, 1.0)) / log(10.0)) - 1.0)
    return "%d" % int(round(v / maxf(mag, 1.0)) * maxf(mag, 1.0))

func _update_limited(dt: float) -> void:
    var req: float = float(_s("requested_speed"))
    var eff: float = float(_s("effective_speed"))
    if int(req) != last_req:
        last_req = int(req)
        eff_smooth = -1.0
        limited = false
        limit_t = 0.0
    eff_smooth = eff if eff_smooth < 0.0 else lerpf(eff_smooth, eff, 0.3)
    var ratio: float = eff_smooth / req if req > 0.0 else 1.0
    var flip_cond: bool = ratio >= 0.97 if limited else ratio < 0.95
    limit_t = limit_t + dt if flip_cond else 0.0
    if limit_t >= 1.5:
        limited = not limited
        limit_t = 0.0

func _refresh_status() -> void:
    if core == null or status == null:
        return
    var req: int = _s("requested_speed")
    var paused: bool = _s("paused") == 1
    pause_btn.text = "Play" if paused else "Pause"
    for i in speed_btns.size():
        speed_btns[i].set_pressed_no_signal(SPEEDS[i] == req)
    if paused:
        status.text = "Paused at %sx" % _fmt_speed(float(req))
        status.add_theme_color_override("font_color", p["dim"])
    elif limited:
        status.text = "Limited speed: about %sx of %sx" % [_fmt_speed(eff_smooth), _fmt_speed(float(req))]
        status.add_theme_color_override("font_color", p["warn"])
    else:
        status.text = "Running at %sx" % _fmt_speed(float(req))
        status.add_theme_color_override("font_color", p["text"])

func _refresh_debug() -> void:
    if debug_label == null or not debug_label.visible:
        return
    var lines: PackedStringArray = []
    lines.append("fps %d" % int(Engine.get_frames_per_second()))
    lines.append("view center (%.1f, %.1f) zoom %.1f px/cell" % [view.center.x, view.center.y, view.zoom])
    lines.append("ui scale %.2f  window %s vp %s" % [U, str(DisplayServer.window_get_size()), str(get_viewport_rect().size)])
    lines.append("safe insets l %d t %d r %d b %d  cutouts %d" % [int(inset.x), int(inset.y), int(inset.z), int(inset.w), cutouts_seen])
    var w: Window = get_window()
    lines.append("scale mode %d aspect %d  display %.0f Hz  vsync %d" % [w.content_scale_mode, w.content_scale_aspect, DisplayServer.screen_get_refresh_rate(), DisplayServer.window_get_vsync_mode()])
    if core != null:
        lines.append("tick %d  req %d  eff %d  owed %d" % [_s("tick"), _s("requested_speed"), _s("effective_speed"), _s("owed_ticks")])
        lines.append("busy %.1f%%  governor drops %d raises %d" % [_s("busy_permille") / 10.0, _s("governor_drops"), _s("governor_raises")])
        lines.append("smoothed eff %.0f  limited %s" % [eff_smooth, str(limited)])
        lines.append("rss %d MB" % (int(core.call("rss_kb")) / 1024))
    debug_label.text = "\n".join(lines)

func _process(delta: float) -> void:
    poll_t += delta
    if poll_t < 0.25:
        return
    var dt: float = poll_t
    poll_t = 0.0
    if core != null:
        st = core.call("stats")
        _update_limited(dt)
        _refresh_status()
    _refresh_debug()

# ---------- panels ----------
func _open_settings() -> void:
    settings_layer.visible = true
    view.input_enabled = false

func _close_settings() -> void:
    settings_layer.visible = false
    view.input_enabled = true

func _on_dark(on: bool) -> void:
    dark = on
    _save_settings()
    _apply_palette()

func _apply_orientation() -> void:
    DisplayServer.screen_set_orientation(DisplayServer.SCREEN_SENSOR_PORTRAIT if portrait else DisplayServer.SCREEN_SENSOR_LANDSCAPE)

func _on_portrait(on: bool) -> void:
    portrait = on
    _save_settings()
    _apply_orientation()

func _on_debug(on: bool) -> void:
    show_debug = on
    debug_label.visible = on
    _save_settings()
    _refresh_debug()

func _open_dev_checks() -> void:
    if dev_layer != null:
        return
    _close_settings()
    _stop_core()
    dev_layer = Control.new()
    dev_layer.set_anchors_preset(Control.PRESET_FULL_RECT)
    var bg := ColorRect.new()
    bg.color = p["bg"]
    bg.set_anchors_preset(Control.PRESET_FULL_RECT)
    dev_layer.add_child(bg)
    var dc = (load("res://dev_check.tscn") as PackedScene).instantiate()
    dc.set("ui_scale", U)
    dc.set("inset", Vector2(inset.x, inset.y))
    dc.connect("closed", _close_dev_checks)
    dev_layer.add_child(dc)
    add_child(dev_layer)
    margin.visible = false

func _close_dev_checks() -> void:
    if dev_layer != null:
        dev_layer.queue_free()
        dev_layer = null
    margin.visible = true
    _start_core()

# ---------- Android back / close ----------
func _notification(what: int) -> void:
    if what == NOTIFICATION_WM_GO_BACK_REQUEST:
        if dev_layer != null:
            _close_dev_checks()
        elif saves_layer != null and saves_layer.visible:
            _close_saves()
        elif settings_layer != null and settings_layer.visible:
            _close_settings()
        else:
            _quit()
    elif what == NOTIFICATION_WM_CLOSE_REQUEST:
        _quit()
    elif what == NOTIFICATION_APPLICATION_PAUSED:
        _on_app_paused()
    elif what == NOTIFICATION_APPLICATION_RESUMED:
        _on_app_resumed()

# Android lifecycle (D-023: no offline progress). Going to the background autosaves and pauses the
# simulation; coming back resumes it only if the player had not paused it.
func _on_app_paused() -> void:
    if core == null:
        return
    core.call("save_game", _slot_path("autosave"), "autosave")
    if _s("paused") == 0:
        core.call("set_paused", true)
        bg_paused = true

func _on_app_resumed() -> void:
    if core != null and bg_paused:
        core.call("set_paused", false)
    bg_paused = false

func _quit() -> void:
    if core != null:
        core.call("save_game", _slot_path("autosave"), "autosave")
    _stop_core()
    get_tree().quit()
