# AGENTS.md

## Project Overview
- This repo is a Rust/macOS remake of the classic QBasic Gorilla artillery game.
- The current app uses `macroquad` for rendering, input, timing, and audio.
- v1 scope is local multiplayer only: two human players alternate throws on the same machine.
- Visuals and audio are procedural/code-driven right now. There are no external asset files yet.

## Environment
- Rust edition: `2021`
- Expected toolchain: Rust `1.94.x` and Cargo `1.94.x`.
- Dependencies are now aligned with the Rust 1.94 toolchain. Run a full check/test pass after dependency upgrades.
- Primary development target is macOS. Desktop window behavior, fullscreen, and input feel matter.

## Useful Commands
- Prefer `Makefile` targets for day-to-day work:
  `make check`, `make test`, `make run`, `make fmt`, `make lint`.
- `cargo` commands are still valid, but keep docs and team usage centered on `make`.

## Code Map
- [src/main.rs](/Users/sandeep/rust/gorilla/src/main.rs): window config and async app bootstrap.
- [src/app.rs](/Users/sandeep/rust/gorilla/src/app.rs): top-level app loop, scene transitions, pause flow, and gameplay orchestration.
- [src/model.rs](/Users/sandeep/rust/gorilla/src/model.rs): core data model, match/round setup, skyline generation, and player state.
- [src/physics.rs](/Users/sandeep/rust/gorilla/src/physics.rs): projectile stepping, preview simulation, and collision resolution. Keep this logic pure/testable.
- [src/render.rs](/Users/sandeep/rust/gorilla/src/render.rs): virtual-resolution rendering, HUD, menus, city/gorilla drawing, and overlays.
- [src/audio.rs](/Users/sandeep/rust/gorilla/src/audio.rs): generated retro SFX and volume syncing.
- [src/persistence.rs](/Users/sandeep/rust/gorilla/src/persistence.rs): settings load/save in the user config directory.
- [src/assets.rs](/Users/sandeep/rust/gorilla/src/assets.rs): gameplay/render constants and palette helpers.
- [src/input.rs](/Users/sandeep/rust/gorilla/src/input.rs): viewport-space input helpers and simple UI button hit testing.
- [src/scene.rs](/Users/sandeep/rust/gorilla/src/scene.rs): scene enum for app state flow.

## Gameplay Invariants
- Fixed virtual resolution is `320x180`; keep gameplay authored in virtual coordinates and scale to screen.
- Internal render target should stay at least `1920x1080` for readability before presenting to the window.
- Core scene flow is:
  `MainMenu -> Settings -> MatchIntro -> RoundPlaying -> RoundResolve -> MatchOver`
- Match flow is round-based: hit gorilla -> score increments -> next round regenerates skyline/wind -> match ends at target score.
- Input model is text-entry throw controls (angle + velocity), with `Tab` to switch field and `Enter`/`Space` to throw.
- `physics.rs` should remain mostly renderer-independent so unit tests can cover motion and collisions without needing a window.
- Settings persistence currently includes fullscreen, volume, target score, trajectory preview, and screen shake.

## Implementation Guidance
- Prefer keeping gameplay math, round generation, and winner resolution in pure functions or simple state transitions.
- If you add polish, preserve the existing “chunky retro skyline” visual language instead of switching to generic UI/game art.
- If you add new settings, wire them through:
  `model::Settings` -> `persistence.rs` -> `app.rs`/`render.rs`/`audio.rs` as needed.
- If you touch rendering, keep upscale behavior readable and stable on Retina/non-Retina displays.
- If you change collision or physics tuning, add or update unit tests in `model.rs`, `physics.rs`, or `app.rs`.

## Validation Expectations
- Minimum before handing off changes:
  `make test`
- When changing gameplay, menus, settings, input, fullscreen, or rendering:
  run `make run` and do a short manual smoke test on macOS.
- Manual smoke test checklist:
  app launches to main menu, settings toggle persists, local match starts, text throw input works, projectile collisions resolve, camera shake stops shortly after impacts, pause works with `Esc`, fullscreen toggles with `Cmd+Enter`.

## Notes for Future Work
- There is no AI opponent, no external asset pipeline, and no networking.
- Audio is synthesized in code; if replacing it with files later, keep startup and packaging implications in mind.
- The current render path creates a virtual render target per frame. If performance work is needed later, this is a reasonable place to optimize.
