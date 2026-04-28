# coomer

This is a Rust app providing a frozen full-screen zoom overlay for macOS.
Uses AppKit through rust bindings.

## Ubiquitous language

Domain terminology (Overlay, HUD, Screenshot, Zoom vs meeting app, and related terms) lives in [UBIQUITOUS_LANGUAGE.md](UBIQUITOUS_LANGUAGE.md). Prefer those names in code, comments, and discussions.

## Conventions

- Prefer AppKit views for native UI affordances; keep Core Graphics rendering focused on the frozen image/effects.
- In `objc2` code, avoid `as_super()` unless a real superclass coercion or owned upcast is needed.
- Derive HUD/layout geometry from named constants instead of unexplained magic numbers.
- When AppKit docs suggest event APIs, first decide whether the feature is really event-driven or just geometry/state.
