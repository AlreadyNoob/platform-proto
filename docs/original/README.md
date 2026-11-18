# Platform — Project Documents

This package contains the initial design and development handoff documents for the Android game **Platform**.

## Files
- `PLATFORM_GDD.md` — game design document, including the original concept transcript in Appendix A.
- `CLAUDE_MASTER_PROMPT.md` — instructions for Claude to evaluate the engine and develop the project in small, verified milestones.

## How to use
1. Upload both Markdown files to Claude.
2. Start a project or conversation dedicated to Platform.
3. Paste the contents of `CLAUDE_MASTER_PROMPT.md` or ask Claude to follow it.
4. Ask Claude to begin with Phase 0 only. Do not ask it to build the whole game at once.
5. Keep every complete ZIP milestone it delivers. For each new session, provide the latest ZIP and `SESSION_HANDOFF.md`.

## Important
The engine recommendation is provisional. Claude must compare Godot 4 and LibGDX using current official documentation and a small Android prototype before committing. The long-term millions-of-particles goal is an ambition to benchmark, not a guaranteed performance claim.
