M02a check app (com.example.platformm02), Godot 4.7.2 official, TECNO CM7, 2026-11-19. Raw log pasted by the owner, unmodified.
CI run 44406424568 (ci.yml): jobs core and android-arm64-so both green (commit f7e2f85).
Note: the line "tick 40393 -> 293" is a harness artifact: the owner pressed button 4 while the pause check (button 3) was still waiting, and button 4 starts a fresh run (tick restarts at 0). Not a core fault (confirmed by reading game/main.gd).
