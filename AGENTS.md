# AGENTS.md

## Project Overview
- This repo is a Rust/macOS remake of the classic QBasic Gorilla artillery game.
- It uses `macroquad` for rendering, input, timing, and audio.
- The game is local multiplayer only: two human players share one machine and alternate turns.
- Art and audio are procedural/code-driven. There are no checked-in sprite sheets, sound files, or font assets.

## Environment
- Rust edition: `2021`
- Expected toolchain: Rust `1.94.x`, Cargo `1.94.x`
- Primary dependencies:
  `macroquad = 0.4.14` with `audio`, `rand = 0.9.2`, `dirs = 6.0.0`, `toml = 0.9.5`
- Primary target is macOS. Fullscreen behavior, text clarity, and desktop input feel matter.

## Preferred Commands
- Use the `Makefile` first:
  `make check`, `make test`, `make run`, `make fmt`, `make lint`
- Direct cargo commands are fine when needed, but project docs and workflows should stay Makefile-first.

## Code Map
- [src/main.rs](/Users/sandeep/rust/gorilla/src/main.rs): window config and app bootstrap.
- [src/app.rs](/Users/sandeep/rust/gorilla/src/app.rs): top-level game loop, scene transitions, pause handling, throw parsing, and turn orchestration.
- [src/model.rs](/Users/sandeep/rust/gorilla/src/model.rs): game state structs, skyline generation, balanced spawn selection, and throw-input defaults.
- [src/physics.rs](/Users/sandeep/rust/gorilla/src/physics.rs): projectile stepping, preview simulation, collision handling, and out-of-bounds policy.
- [src/render.rs](/Users/sandeep/rust/gorilla/src/render.rs): world rendering plus screen-space UI overlays for crisp text/panels.
- [src/audio.rs](/Users/sandeep/rust/gorilla/src/audio.rs): generated retro SFX and volume syncing.
- [src/persistence.rs](/Users/sandeep/rust/gorilla/src/persistence.rs): settings load/save in the user config directory.
- [src/assets.rs](/Users/sandeep/rust/gorilla/src/assets.rs): gameplay constants and palette helpers.
- [src/input.rs](/Users/sandeep/rust/gorilla/src/input.rs): viewport-space input helpers and button hit testing.
- [src/scene.rs](/Users/sandeep/rust/gorilla/src/scene.rs): app scene enum.

## Gameplay Rules and Invariants
- World logic uses a virtual resolution of `320x180`.
- Buildings are intentionally wider now: `BUILDING_WIDTH = 24.0`, roughly 2x gorilla diameter.
- Skyline generation is smoothed and spawn rooftops are selected as a balanced left/right pair. Do not revert to fully random left/right spawn picks.
- Match flow is:
  `MainMenu -> Settings -> MatchIntro -> RoundPlaying -> RoundResolve -> MatchOver`
- Projectile behavior:
  shots may leave the top or side of the screen and come back; they only die when they fall below the world or travel extremely far sideways.
- Throw input:
  players enter text values for angle and velocity, `Tab` switches fields, `Enter` or `Space` throws.
- Angle semantics are mirrored:
  Player 1 `45` means up-right toward Player 2.
  Player 2 `45` means up-left toward Player 1.
  The UI label should stay `Angle (0-90)`.
- New throw fields start blank. Do not silently prefill them with `45`, `135`, `65`, etc.

## Rendering Guidance
- Keep gameplay world rendering in virtual coordinates.
- Keep UI text and panels in screen space after the world texture is presented.
  This is the key fix for crisp fonts on macOS.
- Do not move HUD/menu/pause/input text back into the gameplay camera pass unless you are intentionally reworking text rendering from scratch.
- The current text pipeline loads a system TTF font on macOS at startup and uses `draw_text_ex`.
- If you touch menu or pause layouts, preserve alignment between measured text and panel/button rectangles. Recent bugs came from mismatched text scaling and box sizes.

## Pause/Menu Behavior
- `Esc` during gameplay toggles pause.
- While paused, pause buttons must still be interactive.
  Do not early-return from `update()` before pause click handling runs.
- While paused, the throw-input overlay and gameplay footer should stay hidden behind the pause overlay to avoid visual clutter.

## Settings
- Persisted settings currently include:
  fullscreen, master volume, sfx volume, target score, trajectory preview, screen shake
- Any new setting should be wired through:
  `model::Settings -> persistence.rs -> app.rs/render.rs/audio.rs`

## Validation Expectations
- Minimum before handoff:
  `make lint`
  `make test`
- When changing gameplay, rendering, menus, pause flow, or input:
  run `make run` and do a short manual smoke test on macOS.
- Manual smoke test checklist:
  main menu text is crisp and aligned
  pause overlay opens with `Esc`, looks clean, and buttons work
  throw fields start blank
  Player 2 mirrored angle input works (`45` aims toward Player 1)
  skyline feels fair from both sides
  projectiles can leave the screen and return
  fullscreen still toggles with `Cmd+Enter`

## Notes
- There is no AI, networking, or external asset pipeline yet.
- Audio is synthesized in code. If replacing it later with files, consider packaging and startup implications.
- Tests currently cover physics behavior, spawn fairness basics, target score handling, and mirrored Player 2 input. Extend those tests when changing gameplay rules.
