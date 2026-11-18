# CONTINUITY PROTOCOL — how every session keeps the project resumable from two ZIPs

**Goal:** the owner can start a brand-new Claude session, drop (1) the latest milestone ZIP and (2) the original proposal ZIP, paste the one-line prompt, and work continues exactly where it stopped. No session may depend on memory of earlier conversations.

## A. At the START of a session
Follow `START_HERE.md` section 0 and 9. Verify with `tools/check_handoff.py`. Never rely on anything the owner says you "did before" unless it is in these files; if the owner reports something that contradicts the files, say so and ask.

## B. DURING a session
- Work one bounded milestone/sub-step at a time. Do not start a feature while an earlier one is broken.
- After every meaningful step that the owner could not easily redo (new code, new results), send a **checkpoint ZIP** (same naming rules, patch version +1, suffix such as `_checkpoint`) with the docs updated, so a dropped session loses almost nothing. If the session might end soon, say so and send one immediately.
- Record real results only. Raw outputs the owner pastes or uploads go under `prototype/reference/results/...` (or the equivalent folder of the current milestone) unmodified.

## C. At the END of every milestone or checkpoint (mandatory checklist)
1. **Docs updated and consistent:** `README.md` (status line + reading order), `PROJECT_STATE.json` (version, date, milestone statuses, decisions, open questions, environment changes, next step), `PROJECT_LOG.md` (append an entry: what happened, what the owner did, results, mistakes and fixes), `SESSION_HANDOFF.md` (rewrite, do not append), `DEVELOPMENT_PROGRESS.md`, `CHANGELOG.md`, `TODO.md`, `KNOWN_ISSUES.md`, `TESTING.md`, `DECISIONS.md`, `ARCHITECTURE.md` if the design changed, `ENVIRONMENT_AND_REPO.md` if paths, accounts or commands changed, `CLAUDE.md` if rules or build commands changed, `MANIFEST.md`.
2. **START_HERE.md** section 2 (state), 3 (what the owner did), 5 (next step) must be rewritten for the new state; the rest changes only when the process changes. Keep it self-sufficient.
3. **Honesty pass:** every claim carries a verification label; nothing marked verified that was not run; unrun code says so.
4. **Integrity check:** run `python3 tools/check_handoff.py <project folder>` and again on the final ZIP; fix every FAIL. The checker confirms required files, version consistency, original-file hashes, no oversized files and no secrets-looking strings.
5. **Package:** `Platform_vMAJOR.MINOR.PATCH_milestone-NN_short-description.zip`, one top folder with the same name, complete (never a diff), never overwrite earlier ZIPs. Bump MINOR for a completed milestone, PATCH for checkpoints/fixes. Source and raw results go in; build outputs (`.apk`, `.so`, `target/`) do not (the workflow rebuilds them).
6. **Tell the owner, in the final message:** which ZIP is now the latest; "drop this ZIP together with your original `Platform_Design_and_Claude_Prompt_v0_1.zip` into a new session and paste the prompt from `PROMPT_TO_PASTE.txt`"; the exact commands (if any) to copy the new files into the repo/Godot project; the open decisions waiting for the owner.
7. If the original proposal files must change (the owner changes the design), keep the original untouched in `docs/original/`, create the new version as a separate file, record the change in `DECISIONS.md`, and update `docs/ORIGINAL_FILES_SHA256.txt` only for files that are meant to stay original.

## D. Suggestions that make resuming robust (recommended to the owner)
- Keep **both ZIPs together** in one phone folder (for example `Download/Platform_handoff/`) and keep one extra copy in a cloud drive. The latest milestone ZIP + the original proposal ZIP are the whole project.
- Mirror the docs into the private GitHub repo (the milestone ZIP is already laid out as a repo) and tag each milestone: `git tag m01-v0.1.1 && git push --tags`. The repo is a second backup; the ZIP stays canonical because Claude cannot read the repo.
- Put build outputs and local binaries in `.gitignore` (included).
- Never put tokens, passwords or personal data in the project files.
