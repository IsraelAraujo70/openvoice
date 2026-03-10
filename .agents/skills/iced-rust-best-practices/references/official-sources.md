# Official Sources

Use these sources in this order.

## 1. Architecture

- Iced book, Architecture:
  - https://book.iced.rs/architecture.html
  - Use for `State`, `Message`, `update`, `view`, and Elm-style boundaries.

- Iced book, The Runtime:
  - https://book.iced.rs/the-runtime.html
  - Use to reason about initialization, the feedback loop, and what the runtime owns.

## 2. API Surface

- `iced::window` docs:
  - https://docs.rs/iced/latest/iced/window/index.html
  - Use for window tasks, events, ids, levels, position, monitor size, and runtime window operations.

- `enable_mouse_passthrough` docs:
  - https://docs.rs/iced/latest/iced/window/fn.enable_mouse_passthrough.html
  - Use when implementing click-through overlays.

- `Subscription` docs:
  - https://docs.rs/iced/latest/iced/struct.Subscription.html
  - Use when deciding whether a passive stream belongs in `subscription`.

## 3. Upstream Repo

- Main repo:
  - https://github.com/iced-rs/iced
  - Check `examples/`, the top-level README, and current repo layout before inventing patterns.

- Releases:
  - https://github.com/iced-rs/iced/releases
  - Check for version-specific APIs and recent changes. Important examples from `0.14.0` include:
    - mouse passthrough tasks
    - `override_redirect` for X11 windows
    - `winit` 0.30 upgrade

## Practical Notes

- Do not assume feature parity across Linux backends just because the `iced` API exists.
- For window behavior, the underlying backend is often `winit`; backend limitations can invalidate an otherwise valid Iced API call.
- Prefer explicit Cargo features for Linux desktop apps when X11/Wayland behavior matters.
- When a pattern is unclear, inspect the official `examples/` tree upstream before inventing your own architecture.
