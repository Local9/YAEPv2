# YAEP Rust (Tauri + SvelteKit)

Initial scaffold for rebuilding YAEP as a Windows-first Tauri desktop app with a SvelteKit frontend.

## Scope of this initial project

- SvelteKit frontend shell with route navigation for major feature areas.
- Tauri Rust backend with spec-aligned module boundaries:
  - `db`
  - `hotkeys`
  - `thumbnail_service`
  - `dwm`
  - `windows`
  - `eve_profile_tools`
- Typed command bridge for frontend/backend communication.
- SQLite schema bootstrap SQL matching the rebuild spec.

## Next implementation milestones

1. Wire SQLite initialization and bootstrap defaults (`Default` profile, `exefile`, etc.).
2. Implement profile CRUD and active profile switching commands.
3. Add Windows DWM thumbnail windows and overlay lifecycle management.
4. Add global hotkey registration/capture APIs.
5. Emit backend events for thumbnail/focus/profile updates.
