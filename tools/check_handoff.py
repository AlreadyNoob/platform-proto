#!/usr/bin/env python3
"""Handoff integrity checker. Usage: python3 tools/check_handoff.py <project folder or .zip>
Checks: required files, version consistency, original proposal hashes, oversized files, secret-looking strings.
Exit code 0 = all PASS (warnings allowed), 1 = at least one FAIL."""
import sys, os, json, re, hashlib, zipfile, tempfile, shutil

REQUIRED = ["START_HERE.md", "PROMPT_TO_PASTE.txt", "PROJECT_STATE.json", "PROJECT_LOG.md", "CONTINUITY_PROTOCOL.md",
            "ENVIRONMENT_AND_REPO.md", "README.md", "CLAUDE.md", "SESSION_HANDOFF.md", "DECISIONS.md", "ARCHITECTURE.md",
            "ENGINE_EVALUATION.md", "PROJECT_UNDERSTANDING.md", "PROTOTYPE_PLAN.md", "DEVELOPMENT_PROGRESS.md", "CHANGELOG.md",
            "TODO.md", "TESTING.md", "KNOWN_ISSUES.md", "MANIFEST.md", "MILESTONE_01_REPORT.md", "PLATFORM_GDD.md",
            "docs/CLAUDE_MASTER_PROMPT.md", "docs/ORIGINAL_FILES_SHA256.txt", "docs/original/PLATFORM_GDD.md",
            "docs/original/CLAUDE_MASTER_PROMPT.md", "docs/original/README.md", "tools/check_handoff.py", ".gitignore"]
SECRET_PATTERNS = [r"ghp_[A-Za-z0-9]{20,}", r"github_pat_[A-Za-z0-9_]{20,}", r"-----BEGIN [A-Z ]*PRIVATE KEY-----", r"AKIA[0-9A-Z]{16}"]
MAX_MB = 25

def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()

def main(arg):
    tmp = None
    if os.path.isfile(arg) and arg.endswith(".zip"):
        tmp = tempfile.mkdtemp()
        zipfile.ZipFile(arg).extractall(tmp)
        tops = [d for d in os.listdir(tmp) if os.path.isdir(os.path.join(tmp, d))]
        root = os.path.join(tmp, tops[0]) if len(tops) == 1 else tmp
        base = os.path.basename(arg)
        print("ZIP:", base, "| top folders:", tops)
        if len(tops) != 1:
            print("FAIL zip must contain exactly one top folder")
        if not re.match(r"Platform_v\d+\.\d+\.\d+_milestone-[0-9a-z]+_[a-z0-9-_]+\.zip$", base):
            print("WARN zip name does not follow Platform_vX.Y.Z_milestone-NN_description.zip")
    else:
        root = arg
    fails = 0
    def ok(c, msg, warn=False):
        nonlocal fails
        tag = "PASS" if c else ("WARN" if warn else "FAIL")
        if not c and not warn:
            fails += 1
        print(f"{tag} {msg}")
    for f in REQUIRED:
        ok(os.path.isfile(os.path.join(root, f)), f"required file {f}")
    try:
        st = json.load(open(os.path.join(root, "PROJECT_STATE.json")))
        ok(True, "PROJECT_STATE.json parses")
        ver = st.get("handoff_package_version", "")
        for f in ("START_HERE.md", "README.md", "CHANGELOG.md", "MANIFEST.md", "SESSION_HANDOFF.md"):
            p = os.path.join(root, f)
            ok(os.path.isfile(p) and ("v" + ver) in open(p, encoding="utf-8", errors="ignore").read(), f"{f} mentions v{ver}")
        ok(st.get("latest_zip", "").startswith("Platform_v" + ver), "PROJECT_STATE latest_zip matches version")
        if os.path.basename(root).startswith("Platform_v"):
            ok(os.path.basename(root).startswith("Platform_v" + ver), "folder name matches version")
    except Exception as e:
        ok(False, f"PROJECT_STATE.json problem: {e}")
    # original files
    hashes = {}
    try:
        for line in open(os.path.join(root, "docs/ORIGINAL_FILES_SHA256.txt")):
            if line.strip() and not line.startswith("#"):
                h, n = line.split()
                hashes[n] = h
    except Exception:
        pass
    where = {"PLATFORM_GDD.md": ["PLATFORM_GDD.md", "docs/original/PLATFORM_GDD.md"],
             "CLAUDE_MASTER_PROMPT.md": ["docs/CLAUDE_MASTER_PROMPT.md", "docs/original/CLAUDE_MASTER_PROMPT.md"],
             "README.md": ["docs/original/README.md"]}   # the root README.md is the project's own, not the original
    ok(set(hashes) == set(where), "ORIGINAL_FILES_SHA256.txt lists the three original files")
    for n, h in hashes.items():
        for loc in where.get(n, []):
            p = os.path.join(root, loc)
            ok(os.path.isfile(p) and sha(p) == h, f"original file {loc} is byte-identical to the owner's original")
    try:
        for f in st.get("files_to_read_in_order", []):
            ok(os.path.isfile(os.path.join(root, f)), f"reading-list file exists: {f}")
    except Exception:
        pass
    # sizes, secrets, binaries
    big, secrets, bins = [], [], []
    for d, _, fs in os.walk(root):
        for f in fs:
            p = os.path.join(d, f)
            rel = os.path.relpath(p, root)
            if os.path.getsize(p) > MAX_MB * 1024 * 1024:
                big.append(rel)
            if f.endswith((".so", ".apk", ".aab")):
                bins.append(rel)
            if os.path.getsize(p) < 2_000_000 and not f.endswith((".jpg", ".png", ".zip")):
                try:
                    t = open(p, encoding="utf-8", errors="ignore").read()
                except Exception:
                    continue
                if rel != os.path.join("tools", "check_handoff.py") and any(re.search(s, t) for s in SECRET_PATTERNS):
                    secrets.append(rel)
    ok(not big, f"no file larger than {MAX_MB} MB {big}")
    ok(not secrets, f"no secret-looking strings {secrets}")
    ok(not bins, f"no build binaries included {bins}", warn=True)
    print("RESULT:", "ALL PASS" if fails == 0 else f"{fails} FAIL")
    if tmp:
        shutil.rmtree(tmp, ignore_errors=True)
    return 1 if fails else 0

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(__doc__); sys.exit(2)
    sys.exit(main(sys.argv[1]))
