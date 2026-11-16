#!/usr/bin/env python3
# Run from the repo root:  python3 PATCH_workflow_release_upload.py
# Replaces the artifact upload of the Android .so with a GitHub Release asset (artifact storage quota is full),
# and makes the desktop-results upload non-fatal.
p = ".github/workflows/prototype-m01.yml"
s = open(p).read()
if "m01-android-so" in s:
    print("already patched"); raise SystemExit
old_so = """      - uses: actions/upload-artifact@v4
        with: { name: m01-android-arm64-so, path: prototype/godot_proto/bin/android_arm64/libplatform_bridge_proto.so }
"""
new_so = """      - name: Publish .so as a release asset (does not use artifact storage)
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          gh release delete m01-android-so --yes --cleanup-tag --repo "$GITHUB_REPOSITORY" || true
          gh release create m01-android-so godot_proto/bin/android_arm64/libplatform_bridge_proto.so --repo "$GITHUB_REPOSITORY" --target "$GITHUB_SHA" --prerelease --title "M01 Android arm64 .so" --notes "Prototype build of the Godot bridge, run $GITHUB_RUN_ID"
"""
old_core = """      - uses: actions/upload-artifact@v4
        with: { name: m01-desktop-results, path: "prototype/*.txt" }
"""
new_core = """      - uses: actions/upload-artifact@v4
        continue-on-error: true
        with: { name: m01-desktop-results, path: "prototype/*.txt" }
"""
assert old_so in s and old_core in s, "workflow text not as expected"
s = s.replace(old_so, new_so).replace(old_core, new_core)
s = s.replace("defaults:\n", "permissions:\n  contents: write\ndefaults:\n", 1)
open(p, "w").write(s)
print("patched")
