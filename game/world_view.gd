extends Control
# World view (Milestone 03d): pan/zoom grid + particles + placement input + collapse-wave animation.
# Presentation only: it never decides anything. It reports WHICH cells the player touched (place_requested) and draws what the core
# published (set_cells, add_events). The simulation result never depends on anything drawn here (D-025).
# World units are grid cells; cell (x, y) covers [x, x+1) x [y, y+1). `center` is the world point at the middle of the view,
# `zoom` is screen pixels per cell.
# Input, tap mode (default):   one finger: tap = place one particle, drag = pan.      Two fingers: pinch-zoom + pan.
# Input, paint mode:           one finger: tap = place one, drag = paint a line of particles.   Two fingers: pinch-zoom + pan.
# Desktop (testing): left mouse acts like one finger, right/middle drag pans, wheel zooms.

signal view_changed
signal stroke_started
signal place_requested(cell: Vector2i)
signal stroke_ended

const MODE_TAP := 0
const MODE_PAINT := 1
const TAP_MAX_MS := 800
const MOVE_THRESHOLD_UI := 9.0     # finger movement (in ui units) before a touch stops being a tap
const MAX_LINE_CELLS := 400        # safety cap for one drag segment
const MAX_EFFECTS := 400
const REPLACE_AFTER_MS := 250      # paint mode: wait this long before sending a cell of the same drag again
const WAVE_STEP_S := 0.07          # delay between wave depths
const WAVE_LIFE_S := 0.55

var center := Vector2.ZERO
var zoom := 24.0
var zoom_min := 1.0
var zoom_max := 400.0
var ui := 1.0
var input_enabled := true
var mode := MODE_TAP
var placing_enabled := true
var can_afford := true
var col_bg := Color("f7f6f2")
var col_minor := Color("e3e1da")
var col_major := Color("cfcbc0")
var col_particle := Color("2f5f8f")
var col_ok := Color("3f9f5f")
var col_bad := Color("c0504d")
var col_wave := Color("b5651d")

var cells := PackedInt32Array()
var occupied := {}            # Vector2i -> type id
var effects: Array = []       # {pos: Vector2 (cell centre), t0: float, delay: float}

var _touches := {}
var _mouse_ok: bool = not (OS.has_feature("android") or OS.has_feature("ios"))
var _pan_mouse := false
var _g_start := Vector2.ZERO      # global position where the gesture began
var _g_ms := 0
var _g_max := 0                   # most fingers down at once during this gesture
var _g_moved := false
var _stroke_active := false
var _stroke_cells := {}
var _last_cell := Vector2i.ZERO
var _ghost_on := false
var _ghost_cell := Vector2i.ZERO

func _ready() -> void:
    clip_contents = true
    mouse_filter = Control.MOUSE_FILTER_IGNORE
    resized.connect(queue_redraw)

func set_colors(bg: Color, minor: Color, major: Color) -> void:
    col_bg = bg
    col_minor = minor
    col_major = major
    queue_redraw()

func set_particle_colors(particle: Color, ok: Color, bad: Color, wave: Color) -> void:
    col_particle = particle
    col_ok = ok
    col_bad = bad
    col_wave = wave
    queue_redraw()

func to_world(local_pos: Vector2) -> Vector2:
    return center + (local_pos - size * 0.5) / zoom

func cell_at_global(global_pos: Vector2) -> Vector2i:
    var w: Vector2 = to_world(global_pos - global_position)
    return Vector2i(int(floor(w.x)), int(floor(w.y)))

func _changed() -> void:
    queue_redraw()
    view_changed.emit()

# ---------- data from the core ----------
func set_cells(arr: PackedInt32Array) -> void:
    cells = arr
    occupied.clear()
    var i: int = 0
    while i + 2 < arr.size():
        occupied[Vector2i(arr[i], arr[i + 1])] = arr[i + 2]
        i += 3
    queue_redraw()

# ev = [tick, x, y, chain, depth, ...]. With animate == false (high speed) nothing is drawn: the state is exact either way.
func add_events(ev: PackedInt64Array, animate: bool) -> void:
    if not animate:
        return
    var now: float = Time.get_ticks_msec() / 1000.0
    var i: int = 0
    while i + 4 < ev.size():
        if effects.size() >= MAX_EFFECTS:
            break
        var depth: int = int(ev[i + 4])
        effects.append({"pos": Vector2(float(ev[i + 1]) + 0.5, float(ev[i + 2]) + 0.5), "t0": now, "delay": float(depth) * WAVE_STEP_S})
        i += 5
    queue_redraw()

func clear_effects() -> void:
    effects.clear()
    queue_redraw()

func _process(_delta: float) -> void:
    if not effects.is_empty():
        queue_redraw()

# ---------- input ----------
func _zoom_about(local_pos: Vector2, factor: float) -> void:
    var wp: Vector2 = to_world(local_pos)
    zoom = clampf(zoom * factor, zoom_min, zoom_max)
    center = wp - (local_pos - size * 0.5) / zoom

func _can_place() -> bool:
    return placing_enabled and input_enabled

func _begin_gesture(pos: Vector2) -> void:
    _g_start = pos
    _g_ms = Time.get_ticks_msec()
    _g_max = 1
    _g_moved = false
    _stroke_active = false
    _stroke_cells.clear()
    _last_cell = cell_at_global(pos)
    _ghost_cell = _last_cell
    _ghost_on = _can_place()
    queue_redraw()

func _end_stroke() -> void:
    if _stroke_active:
        _stroke_active = false
        stroke_ended.emit()
    _stroke_cells.clear()

# A cell is sent at most once per REPLACE_AFTER_MS while it is (still) shown as occupied. Once it is empty again (the particle collapsed),
# the same drag may fill it again. The short delay covers the lag until the published cells show a fresh placement; a duplicate command
# would be harmless anyway (the core ignores occupied cells and charges nothing).
func _emit_place(c: Vector2i) -> void:
    var now: int = Time.get_ticks_msec()
    if _stroke_cells.has(c):
        var sent: int = _stroke_cells[c]
        if occupied.has(c) or now - sent < REPLACE_AFTER_MS:
            return
    _stroke_cells[c] = now
    place_requested.emit(c)

# Every cell on the line between two cells (Bresenham), so a fast drag leaves no gaps.
func _paint_to(target: Vector2i) -> void:
    var x0: int = _last_cell.x
    var y0: int = _last_cell.y
    var dx: int = absi(target.x - x0)
    var dy: int = -absi(target.y - y0)
    var sx: int = 1 if x0 < target.x else -1
    var sy: int = 1 if y0 < target.y else -1
    var err: int = dx + dy
    var n: int = 0
    while n < MAX_LINE_CELLS:
        _emit_place(Vector2i(x0, y0))
        if x0 == target.x and y0 == target.y:
            break
        var e2: int = 2 * err
        if e2 >= dy:
            err += dy
            x0 += sx
        if e2 <= dx:
            err += dx
            y0 += sy
        n += 1
    _last_cell = target

func _press(index: int, pos: Vector2) -> void:
    _touches[index] = pos
    if _touches.size() == 1:
        _begin_gesture(pos)
    else:
        _g_max = 2
        _ghost_on = false
        _end_stroke()
        queue_redraw()

func _release(index: int, _pos: Vector2) -> void:
    if not _touches.has(index):
        return
    var single: bool = _touches.size() == 1 and _g_max == 1
    _touches.erase(index)
    if single:
        if _stroke_active:
            _end_stroke()
        elif not _g_moved and _can_place() and (Time.get_ticks_msec() - _g_ms) <= TAP_MAX_MS:
            stroke_started.emit()
            place_requested.emit(cell_at_global(_g_start))
            stroke_ended.emit()
    if _touches.is_empty():
        _ghost_on = false
        _end_stroke()
        queue_redraw()

func _drag_one(pos: Vector2, rel: Vector2) -> void:
    var far: bool = pos.distance_to(_g_start) > MOVE_THRESHOLD_UI * ui
    if mode == MODE_PAINT and _can_place():
        if not _stroke_active and far:
            _g_moved = true
            _stroke_active = true
            stroke_started.emit()
            _last_cell = cell_at_global(_g_start)
        if _stroke_active:
            var c: Vector2i = cell_at_global(pos)
            _ghost_cell = c
            _paint_to(c)
            queue_redraw()
    else:
        if far:
            _g_moved = true
            _ghost_on = false
        if _g_moved:
            center -= rel / zoom
            _changed()

func _drag_two(old: Dictionary, now: Dictionary) -> void:
    var keys: Array = now.keys()
    var a_old: Vector2 = old[keys[0]]
    var b_old: Vector2 = old[keys[1]]
    var a_new: Vector2 = now[keys[0]]
    var b_new: Vector2 = now[keys[1]]
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

func _input(e: InputEvent) -> void:
    if not input_enabled or not is_visible_in_tree():
        if not _touches.is_empty():
            _touches.clear()   # a panel opened in the middle of a touch: forget it, so no stale finger is left behind
            _ghost_on = false
            _end_stroke()
            queue_redraw()
        return
    var rect: Rect2 = get_global_rect()
    if e is InputEventScreenTouch:
        var t: InputEventScreenTouch = e
        if t.pressed:
            if rect.has_point(t.position):
                _press(t.index, t.position)
        else:
            _release(t.index, t.position)
    elif e is InputEventScreenDrag:
        var d: InputEventScreenDrag = e
        if not _touches.has(d.index):
            return
        var old: Dictionary = _touches.duplicate()
        _touches[d.index] = d.position
        if _touches.size() == 1:
            _drag_one(d.position, d.relative)
        elif _touches.size() == 2:
            _drag_two(old, _touches)
    elif _mouse_ok and e is InputEventMouseButton:
        var mb: InputEventMouseButton = e
        if mb.button_index == MOUSE_BUTTON_LEFT:
            if mb.pressed:
                if rect.has_point(mb.position):
                    _press(-1, mb.position)
            else:
                _release(-1, mb.position)
        elif mb.button_index == MOUSE_BUTTON_RIGHT or mb.button_index == MOUSE_BUTTON_MIDDLE:
            _pan_mouse = mb.pressed and rect.has_point(mb.position)
        elif mb.pressed and rect.has_point(mb.position):
            if mb.button_index == MOUSE_BUTTON_WHEEL_UP:
                _zoom_about(mb.position - global_position, 1.15)
                _changed()
            elif mb.button_index == MOUSE_BUTTON_WHEEL_DOWN:
                _zoom_about(mb.position - global_position, 1.0 / 1.15)
                _changed()
    elif _mouse_ok and e is InputEventMouseMotion:
        var mm: InputEventMouseMotion = e
        if _pan_mouse:
            center -= mm.relative / zoom
            _changed()
        elif _touches.has(-1):
            _touches[-1] = mm.position
            _drag_one(mm.position, mm.relative)

# ---------- drawing ----------
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
    _draw_particles(x0, y0, x1, y1)
    _draw_ghost(x0, y0)
    _draw_effects(x0, y0)

func _draw_particles(x0: float, y0: float, x1: float, y1: float) -> void:
    var gap: float = 1.0 if zoom >= 6.0 else 0.0
    var sz: float = maxf(1.0, zoom - gap)
    var i: int = 0
    while i + 2 < cells.size():
        var cx: int = cells[i]
        var cy: int = cells[i + 1]
        i += 3
        if float(cx) + 1.0 < x0 or float(cx) > x1 or float(cy) + 1.0 < y0 or float(cy) > y1:
            continue
        draw_rect(Rect2((float(cx) - x0) * zoom + gap * 0.5, (float(cy) - y0) * zoom + gap * 0.5, sz, sz), col_particle)

func _draw_ghost(x0: float, y0: float) -> void:
    if not _ghost_on:
        return
    var ok: bool = can_afford and not occupied.has(_ghost_cell)
    var col: Color = col_ok if ok else col_bad
    var r := Rect2((float(_ghost_cell.x) - x0) * zoom, (float(_ghost_cell.y) - y0) * zoom, zoom, zoom)
    draw_rect(r, Color(col.r, col.g, col.b, 0.35))
    draw_rect(r, col, false, maxf(1.5, ui * 1.5))

func _draw_effects(x0: float, y0: float) -> void:
    if effects.is_empty():
        return
    var now: float = Time.get_ticks_msec() / 1000.0
    var keep: Array = []
    for fx in effects:
        var age: float = now - float(fx["t0"]) - float(fx["delay"])
        if age >= WAVE_LIFE_S:
            continue
        keep.append(fx)
        if age < 0.0:
            continue
        var k: float = age / WAVE_LIFE_S
        var pos: Vector2 = fx["pos"]
        var sp := Vector2((pos.x - x0) * zoom, (pos.y - y0) * zoom)
        var r: float = zoom * (0.5 + 1.1 * k)
        var a: float = 1.0 - k
        draw_arc(sp, r, 0.0, TAU, 24, Color(col_wave.r, col_wave.g, col_wave.b, a), maxf(1.5, ui * 2.0))
    effects = keep
