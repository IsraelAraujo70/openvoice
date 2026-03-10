---
name: iced-rust-best-practices
description: Use when building, refactoring, or reviewing Rust desktop apps with Iced. Covers The Elm Architecture in Iced, project structure, Task vs Subscription usage, window/runtime patterns, Linux/X11 vs Wayland caveats, and how to use official Iced docs, book chapters, and upstream examples before making architectural decisions.
license: Apache 2.0
---

Use this skill when the task involves `iced`, `iced::application`, `Task`, `Subscription`, `window::*`, Rust GUI architecture, or refactoring an Iced codebase for readability and scale.

## First Principles

Start from the official Iced architecture:

- `State`: application data
- `Message`: user interactions or meaningful runtime events
- `update`: how messages mutate state and produce `Task`s
- `view`: how state becomes widgets

In Iced, these map directly to The Elm Architecture. Keep this boundary clear even in small apps.

## Default Structure

Prefer process and responsibility boundaries over dumping everything into `main.rs`.

```text
src/
├── main.rs
├── app/
│   ├── mod.rs
│   ├── bootstrap.rs
│   ├── message.rs
│   ├── state.rs
│   └── update.rs
├── ui/
│   ├── mod.rs
│   ├── theme.rs
│   ├── overlay.rs
│   └── components/
├── platform/
│   ├── mod.rs
│   ├── window.rs
│   ├── monitors.rs
│   ├── hotkeys.rs
│   └── audio.rs
├── modules/
│   ├── overlay/
│   ├── meetings/
│   ├── obsidian/
│   ├── audio/
│   └── dictation/
└── support/
    ├── config.rs
    ├── error.rs
    └── logging.rs
```

Use these rules:

- `app/` owns runtime wiring, top-level state, and message routing.
- `ui/` renders only. Keep it dumb.
- `platform/` owns Linux/X11/Wayland behavior, subprocesses, OS integration, hotkeys, audio capture, and monitor detection.
- `modules/` owns product logic. Split each module into `application`, `domain`, and `infrastructure` once it grows.
- `support/` holds config, logging, and shared errors.

## Update Loop Rules

Treat `update` as orchestration, not a dumping ground.

- Match on `Message`.
- Mutate state minimally.
- Return `Task`s for side effects and runtime operations.
- Delegate feature-specific behavior to helper functions once the match arms get long.
- Keep pure business rules outside the Iced shell whenever possible.

Good pattern:

```rust
match message {
    Message::Overlay(msg) => overlay::update(&mut state.overlay, msg),
    Message::Meetings(msg) => meetings::update(&mut state.meetings, msg),
    Message::Quit => close_main_window(state),
}
```

Bad pattern:

- filesystem writes directly in `view`
- parsing CLI output inside widget builders
- hundreds of lines of mixed UI, OS, and domain logic in one `update`

## View Rules

Keep `view` pure and predictable.

- Build widgets from state only.
- Avoid hidden side effects.
- Prefer small view functions per screen or component.
- Move styling into `ui/theme.rs` or dedicated component helpers.
- Pass data in; avoid making view code discover runtime state on its own.

## Task vs Subscription

Use the right primitive:

- `Task`: for requested work
  - window commands
  - async calls
  - triggered side effects after a message
- `Subscription`: for passive streams
  - keyboard events
  - timers
  - sockets
  - runtime/window event streams

Heuristic:

- “Do this now because of a message” => `Task`
- “Keep listening while this state says so” => `Subscription`

Never use `Subscription` as a disguised command queue.

## Window And Runtime Patterns

For windowed Linux apps:

- keep initial `window::Settings` in one place
- store `window::Id` in app state after `window::open_events()`
- treat `window::*` calls as runtime tasks, not direct imperative calls
- isolate platform-specific geometry or monitor detection in `platform/`

If behavior is platform-sensitive, verify the underlying backend before promising support. `window::set_level`, monitor APIs, fullscreen, passthrough, and move semantics may differ between X11 and Wayland.

## Linux Guidance

Be explicit about Linux backend assumptions.

- If the product depends on window-manager behavior, choose `x11` and `wayland` features intentionally in `Cargo.toml`.
- For GNOME/Wayland, verify backend support before relying on `always-on-top`, exact positioning, or special window stacking.
- If the app depends on X11-only behavior, keep that logic in `platform/` and document the decision.
- If the app needs monitor geometry and Iced does not expose enough, isolate external probes like `xrandr` in small testable functions.

## Refactoring Heuristics

Refactor when you see:

- `main.rs` doing everything
- `Message` mixing unrelated domains without sub-messages
- `update` arms doing parsing, IO, and UI logic together
- platform shell and product logic tightly coupled
- view files owning business rules

Refactor into:

- feature message enums
- pure domain helpers
- platform adapters
- tiny parsing functions with tests

## Testing Priorities

Test pure code first.

- parsers
- config loading
- domain rules
- reducers/update helpers that do not require runtime integration

Do not over-invest early in screenshot or widget-level tests unless the project already has that culture.

## Workflow

When using this skill:

1. Read the current Iced entrypoints and identify `State`, `Message`, `update`, `view`, and window setup.
2. Decide whether the task is UI-only, runtime/windowing, or domain logic.
3. Keep runtime shell in `app/`, rendering in `ui/`, and Linux-specific behavior in `platform/`.
4. If version-specific behavior matters, read `references/official-sources.md` and then consult the matching upstream docs or release notes.
5. For platform-sensitive window behavior, validate backend support before implementing or promising a feature.

## Read Next

Read `references/official-sources.md` when:

- deciding architecture or folder structure
- using `Task`, `Subscription`, or window APIs
- debugging Linux/X11/Wayland behavior
- checking whether an API is new in recent Iced releases
