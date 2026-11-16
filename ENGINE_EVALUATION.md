# ENGINE_EVALUATION (2026-11-16) — PROVISIONAL, not yet validated by a prototype

**Honesty note:** this session had no network in the build sandbox, no Godot/Android toolchain and no device. Everything below comes from reading official pages and release notes; **no prototype was built and nothing was benchmarked**. The recommendation is a hypothesis that Milestone 01 must confirm or reject.

## Sources consulted
- Godot homepage and release blog (godotengine.org): latest stable 4.7.2 (18 Aug 2026); 4.7 released 18 Jun 2026; 4.8 dev snapshots in progress.
- Godot 4.7 release notes (godotengine.org/releases/4.7): Android Perfetto tracing, per-platform export-template downloader, `Control` offset transforms (UI animation), `DrawableTexture2D`, GDExtension listing in Project Settings, Android picture-in-picture.
- "Creating games entirely on Android" (godotengine.org, 2 Jun 2026): GABE companion app gives on-device Gradle APK/AAB export.
- Godot docs on C# platform status (via godot-docs history): C# on Android was *experimental* from 4.2. Current status in 4.7 **not verified**.
- Godot docs "Compiling for Android" (templates need JDK 17 for source builds; only relevant if custom templates are ever needed).
- libGDX site (libgdx.com): current releases 1.14.1 and 1.14.2, Apache 2.0, community-maintained.
- NOT consulted yet: godot-rust compatibility with 4.7, Godot 2D MultiMesh/instancing limits on Android, Godot Android renderer docs. Listed as prototype questions.

## Project needs that drive the choice
2D top-down; dense technical UI; high animation/shader quality; sparse grid with huge stored counts but few visible; very high sim speed; robust saves; maintainable across many sessions by a creator who is learning; builds must work from the creator's existing workflow (phone + CI).

## Candidates
| Criterion | Godot 4.7.x | libGDX 1.14.x (Kotlin) | Native Android (Kotlin + GL/Compose) |
|---|---|---|---|
| 2D rendering, batching, shaders | Built in; MultiMesh/canvas items, shader editor, Compatibility + Mobile renderers | SpriteBatch, own GLSL; more manual | Fully manual |
| Dense UI + UI animation | Control nodes, themes, Tween/AnimationPlayer, offset transforms | Scene2D (functional, dated) | Compose (good) but separate from GL view |
| Animation/VFX tooling | Strong (editor, particles, tweens) | Code-only | Code-only |
| Android export workflow | Editor + export templates; on-device Gradle via GABE; also CI | Gradle project; very familiar to Android CI | Gradle |
| Simulation language/perf | GDScript (slow for core); C#/C++/Rust via extensions | JVM (Kotlin) – fast enough with primitive arrays, GC care needed | Kotlin/NDK |
| Native optimization path | GDExtension (C/C++/Rust…) | JNI/NDK | NDK |
| Testability of core | Core can be engine-free if designed so | Core is plain Kotlin, unit-testable | Same |
| Learning curve for creator | Editor-driven; GDScript easy | Code-first; steeper for art/animation | Highest |
| Risks | Rust/GDExtension on Android untested here; scale of 2D instancing unproven | Animation polish costs much more code; UI work heavier | Everything is custom |

## Recommendation (provisional)
**Godot 4.7.x (pin 4.7.2) for presentation, with the simulation core written as an engine-independent library** and connected through a thin binding (D-002, D-003). Reasons: the project's differentiator is animation/shader polish and a dense UI, which Godot provides as tools rather than code; the core's independence keeps the engine swappable (the real mitigation for engine risk); on-device and CI export paths both exist.
**Core language:** Rust is the leading candidate (determinism, memory layout control, cargo tests in CI, the creator's existing Rust experience); alternatives are C++ via GDExtension, or a Kotlin core if libGDX wins. This is open until the prototype measures binding overhead and Android build friction.
**Avoid:** C# in Godot for the core (Android status unverified), and GDScript for the core (performance ceiling).

## What would make us switch to libGDX
Any of: Android export or GDExtension build proves unreliable in the creator's workflow; 2D instancing cannot render ≥100k visible cells/particles at an acceptable frame time on the target device; binding overhead dominates. A time-boxed libGDX spike (PROTOTYPE_PLAN §4) is only run if one of these triggers.

## Performance expectations
No claim is made about particle counts or FPS. Tiers will be defined from measured data (TESTING.md).

## Milestone 01 update (2026-11-17)
- Still **no engine measurements**. Sandbox lacked Rust/Godot/Android tooling.
- Additional source read: godot-rust book — gdext supports stable Godot ≥ 4.0; Android export needs Android command-line tools, JDK 17, a per-triple linker setting (`CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER`) and a `.gdextension` entry `android.debug.arm64`; docs name an NDK version Godot relied on at the time of writing (23.2.8568313) which may not match 4.7.2.
- Core-language question is now narrowed: the algorithmic risk (event-driven scheduling) is language-independent and validated in a reference model. What remains open is purely integration: Rust build on Android, binding overhead, rendering throughput.
- Alternatives to evaluate if integration fails are listed in MILESTONE_01_REPORT.md §5.
