#!/usr/bin/env python3
# Run from the repo root:  python3 PATCH_main_gd.py
# 1) untyped `core` so GDScript calls the Rust methods dynamically (safer parse in the editor)
# 2) cast in _input instead of relying on `is` type narrowing
# 3) drop the 20-second 1%-collapse sim config from the in-app bench (could trigger an Android "not responding" dialog);
#    that case is already measured in Termux.
p = "prototype/godot_proto/main.gd"
s = open(p).read()
if "PATCH_main_gd" in s:
    print("already patched"); raise SystemExit
a = "var core: Object\n"
b = """func _input(e: InputEvent) -> void:
    if e is InputEventScreenDrag: cam.position -= e.relative / cam.zoom
    elif e is InputEventMouseMotion and Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT): cam.position -= e.relative / cam.zoom
"""
c = "    for cfg in [[10000, 4, 1e-4], [100000, 4, 1e-4], [1000000, 4, 1e-4], [1000000, 4, 1e-2]]:\n"
assert a in s and b in s and c in s, "main.gd not as expected"
s = s.replace(a, "var core = null  # PATCH_main_gd\n")
s = s.replace(b, """func _input(e: InputEvent) -> void:
    var d := e as InputEventScreenDrag
    if d != null:
        cam.position -= d.relative / cam.zoom
        return
    var m := e as InputEventMouseMotion
    if m != null and Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
        cam.position -= m.relative / cam.zoom
""")
s = s.replace(c, "    for cfg in [[10000, 4, 1e-4], [100000, 4, 1e-4], [1000000, 4, 1e-4]]:\n")
open(p, "w").write(s)
print("patched")
