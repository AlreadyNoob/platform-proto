# CLAUDE.md — persistent instructions for future sessions

**Project:** Platform (Android, 2D top-down particle/universe simulation). Product owner: the creator (may be learning; explain decisions plainly, avoid dumping internals).

## Session start (always)
Read, in order: `PLATFORM_GDD.md`, `CLAUDE.md`, `DEVELOPMENT_PROGRESS.md`, `ARCHITECTURE.md`, `CHANGELOG.md`, `TODO.md`, latest test report in `TESTING.md`, `SESSION_HANDOFF.md`. Then inspect real project files before editing. Master workflow: `docs/CLAUDE_MASTER_PROMPT.md`.
If documents conflict: stop, state the conflict, propose a resolution.

## Rules
- One milestone (or bounded sub-milestone) at a time. Do not start a new feature while a previous milestone is broken.
- Never claim something is implemented/tested/packaged unless it is. Label: designed / planned / implemented / tested / verified on device.
- Simulation core has **no dependency** on the engine, UI or renderer. UI mutates state only through commands; renders only from snapshots.
- Per-tick probabilities stay per-tick. No frame-rate-dependent outcomes. Randomness is seeded and its state is saved.
- Timeline internal identity is a stable id, never a rounded display value.
- Do not invent balancing numbers silently; mark them PROVISIONAL in `DECISIONS.md` and get owner approval for significant ones.
- Do not add placeholder buttons that do nothing. Do not fake state in the renderer.
- Never claim particle counts or FPS without a reproducible benchmark on named hardware.

## Build/test commands
None yet (Milestone 01 will fill this in).

## Milestone close-out
Update progress, changelog, TODO, known issues, session handoff, testing; package a complete ZIP named `Platform_vMAJOR.MINOR.PATCH_milestone-NN_short-description.zip`. Never overwrite earlier ZIPs.
