extends Control
# World view (Milestone 02b): pan/zoom grid only. Presentation only; no particles exist yet.
# World units are grid cells. `center` is the world point at the middle of the view,
# `zoom` is screen pixels per cell. Touch: one finger pans, two fingers pinch-zoom.
# Desktop (for testing): left-drag pans, mouse wheel zooms.

signal view_changed

var center := Vector2.ZERO
var zoom := 24.0
var zoom_min := 1.0
var zoom_max := 400.0
var ui := 1.0
var input_enabled := true
var col_bg := Color("f7f6f2")
var col_minor := Color("e3e1da")
var col_major := Color("cfcbc0")

var _touches := {}
var _mouse_ok: bool = not (OS.has_feature("android") or OS.has_feature("ios"))
var _mouse_drag := false

func _ready() -> void:
    clip_contents = true
    mouse_filter = Control.MOUSE_FILTER_IGNORE
    resized.connect(queue_redraw)

func set_colors(bg: Color, minor: Color, major: Color) -> void:
    col_bg = bg
    col_minor = minor
    col_major = major
    queue_redraw()

func to_world(local_pos: Vector2) -> Vector2:
    return center + (local_pos - size * 0.5) / zoom

func _changed() -> void:
    queue_redraw()
    view_changed.emit()

func _zoom_about(local_pos: Vector2, factor: float) -> void:
    var wp: Vector2 = to_world(local_pos)
    zoom = clampf(zoom * factor, zoom_min, zoom_max)
    center = wp - (local_pos - size * 0.5) / zoom

func _input(e: InputEvent) -> void:
    if not input_enabled or not is_visible_in_tree():
        return
    var rect: Rect2 = get_global_rect()
    if e is InputEventScreenTouch:
        var t: InputEventScreenTouch = e
        if t.pressed:
            if rect.has_point(t.position):
                _touches[t.index] = t.position
        else:
            _touches.erase(t.index)
    elif e is InputEventScreenDrag:
        var d: InputEventScreenDrag = e
        if not _touches.has(d.index):
            return
        var old: Dictionary = _touches.duplicate()
        _touches[d.index] = d.position
        if _touches.size() == 1:
            center -= d.relative / zoom
        elif _touches.size() == 2:
            var keys: Array = _touches.keys()
            var a_old: Vector2 = old[keys[0]]
            var b_old: Vector2 = old[keys[1]]
            var a_new: Vector2 = _touches[keys[0]]
            var b_new: Vector2 = _touches[keys[1]]
            var d_old: float = a_old.distance_to(b_old)
            var d_new: float = a_new.distance_to(b_new)
            if d_old < 1.0:
                return
            var mid_old: Vector2 = (a_old + b_old) * 0.5 - global_position
            var mid_new: Vector2 = (a_new + b_new) * 0.5 - global_position
            var wm: Vector2 = to_world(mid_old)
            zoom = clampf(zoom * d_new / d_old, zoom_min, zoom_max)
            center = wm - (mid_new - size * 0.5) / zoom
        _changed()
    elif _mouse_ok and e is InputEventMouseButton:
        var mb: InputEventMouseButton = e
        if mb.button_index == MOUSE_BUTTON_LEFT:
            _mouse_drag = mb.pressed and rect.has_point(mb.position)
        elif mb.pressed and rect.has_point(mb.position):
            if mb.button_index == MOUSE_BUTTON_WHEEL_UP:
                _zoom_about(mb.position - global_position, 1.15)
                _changed()
            elif mb.button_index == MOUSE_BUTTON_WHEEL_DOWN:
                _zoom_about(mb.position - global_position, 1.0 / 1.15)
                _changed()
    elif _mouse_ok and e is InputEventMouseMotion and _mouse_drag:
        var mm: InputEventMouseMotion = e
        center -= mm.relative / zoom
        _changed()

func _draw() -> void:
    draw_rect(Rect2(Vector2.ZERO, size), col_bg)
    var min_px: float = 14.0 * ui
    var step: int = 1
    var flip: bool = false
    while zoom * float(step) < min_px:
        step *= 5 if flip else 2
        flip = not flip
    var w: float = maxf(1.0, ui * 0.6)
    var x0: float = center.x - size.x * 0.5 / zoom
    var y0: float = center.y - size.y * 0.5 / zoom
    var x1: float = center.x + size.x * 0.5 / zoom
    var y1: float = center.y + size.y * 0.5 / zoom
    for i in range(int(ceil(x0 / step)), int(floor(x1 / step)) + 1):
        var x: float = (float(i * step) - x0) * zoom
        var major: bool = posmod(i, 5) == 0
        draw_line(Vector2(x, 0.0), Vector2(x, size.y), col_major if major else col_minor, w * (1.5 if major else 1.0))
    for j in range(int(ceil(y0 / step)), int(floor(y1 / step)) + 1):
        var y: float = (float(j * step) - y0) * zoom
        var major_y: bool = posmod(j, 5) == 0
        draw_line(Vector2(0.0, y), Vector2(size.x, y), col_major if major_y else col_minor, w * (1.5 if major_y else 1.0))
