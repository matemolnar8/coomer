# coomer

coomer is a frozen full-screen zoom overlay for macOS screen shares. A Rust binary drives AppKit through the `objc2` crates; Core Graphics paints the frozen **Screenshot**. It renders as a normal window, so meeting apps capture it, unlike macOS Accessibility Zoom.

Prefer simple systems and small diffs. Fight scope creep; honor the dev's intent minimally.

## Glossary

Read [CONTEXT.md](./CONTEXT.md) before naming anything. It is the ubiquitous language (Overlay, Overlay Session, Screenshot, HUD, Flashlight, Zoom vs Zoom Meeting App, ...) and lists the words to avoid. Use those terms in code, comments, issues, and PRs.

When this file speaks:

- `you` is the agent reading this file and changing coomer.
- `user` is the person presenting with coomer during a **Screen Share**.

## How it works

One launch is one **Overlay Session**. `app::run` terminates the previous session through `~/.coomer/pid` (**Replacement Launch**), checks **Screen Recording Permission**, captures the display under the pointer, and opens a borderless screen-saver-level window on that screen. A local `NSEvent` monitor in `input` maps events to `OverlayIntent`s; `OverlaySession` applies them and returns a `SessionOutcome` (redraw, animate, quit). A `CADisplayLink` drives the Fade In and Flashlight animations. `render` draws a session snapshot with Core Graphics; the HUD is AppKit glass views mounted on top.

## Where code lives

- `src/app.rs` - launch flow, pidfile, `NSApplication` setup
- `src/session.rs` - `OverlaySession`: Zoom, Panning, Flashlight, animation state and tuning constants. No AppKit views.
- `src/overlay.rs` - `CoomerWindow` / `CoomerView`, display link, session ownership on the main thread
- `src/input.rs` - key codes, event monitor, `OverlayIntent`; `src/render.rs` - Core Graphics drawing
- `src/hud.rs` - HUD layout, notch offset, hide-when-obstructing, settle animation
- `src/capture.rs` - **Display Capture**; `src/permissions.rs` - Screen Recording Permission flow
- `packaging/Info.plist` + `scripts/package-app.sh` - **Dock-less Agent** bundle; `raycast/` - **Raycast Trigger**
- `.agents/skills/` - `comprehende`, installed from `matemolnar8/comprehende` via the `skills` CLI (`skills-lock.json`), so update it with `npx skills update`, never by hand; and `pages`, a repo-owned skill with its own publish script. `.claude/skills` symlinks here.

## Footguns

1. **Launching takes over the screen.** The Overlay covers the display at screen-saver level and grabs input. A Replacement Launch also kills whatever session the user has open. Ask before you run the binary or the bundle. If you started one, end it with `q`/`Esc` or by the PID you tracked.
2. **Every exit path must clean up.** `app::cleanup_pidfile` runs on normal quit and on the permission-denied exit. A new exit path that skips it leaves a stale pidfile behind.
3. **Main thread only.** AppKit objects need a `MainThreadMarker`. Session state lives in a main-thread `thread_local!` in `overlay.rs`; do not reach for `Mutex`, `static mut`, or threads to share it.

## Hit every surface

The common defect is a change that works on the path you tested and breaks another. Before calling work done, say which applied:

- **Displays.** Multiple monitors, notched and non-notched screens, different backing scale factors. Capture and window frame must agree on the screen under the pointer.
- **Inputs.** Keyboard and pointer, trackpad (precise) and wheel (line) scrolling, `Command`+scroll. Keep the README controls table in sync with `input.rs`.
- **Reverse states.** Flashlight on needs off, Zoom in needs out, and **Reset** restores defaults for everything you added.
- **Motion.** Respect Reduce Motion (see `hud.rs`). No continuous repaint when nothing animates; pause the display link.
- **Screen Share.** The Overlay must stay capturable by meeting apps. Do not switch to window types or APIs that exclude it from capture.

## Taste

- Prefer AppKit views for native UI affordances; keep Core Graphics rendering focused on the frozen image and its effects.
- In `objc2` code, avoid `as_super()` unless a real superclass coercion or owned upcast is needed.
- Derive HUD and layout geometry from named constants, not unexplained magic numbers.
- When AppKit docs suggest event APIs, first decide whether the feature is really event-driven or just geometry and state.
- Keep logic that can be pure in `session.rs` or small functions, and unit test it there.

## Verifying

- Smallest proof the change works: `cargo fmt`, `cargo clippy --all-targets`, `cargo test`, `cargo build --release`.
- For a visible change, do one pass in the real Overlay after asking (see Footguns). The packaged app needs its own Screen Recording grant.

## Pull requests

- Open a PR only when asked, or when requested feature work is ready under an approved plan.
- Conventional titles, plain language: `fix(hud): hide pill when flashlight covers it`.
- Body: the problem in a sentence or two, then how you fixed it, then which checks passed. End with the model and harness that did the work.
- Review the diff with the `comprehende` skill (base is the merge-base with `main`, head is the branch tip), `export` it, then host the export with the `pages` skill using `--pr <number>`. Put the printed URL in the PR body. When the PR gets new commits, refresh the review and publish again to the same URL.
- Visual changes need before/after screenshots; motion needs a short video. Upload evidence to GitHub; do not commit PR-only images.
- One request is one PR.

## Documentation

Most code changes do not need a docs change. Agents can read the code.

- `README.md` is for users: controls, build, permissions, Raycast. Change it when user-visible behavior changes.
- `CONTEXT.md` is the glossary. Add a term only when a new user-facing concept appears; keep the "Avoid" lines current.
- Do not enumerate fields, narrate control flow, or keep file catalogs. When a documented decision changes, rewrite it in place; do not append a second account.

## Plans and work artifacts

- Do not commit plans, research notes, or scratch files. Keep them outside the worktree.
- GitHub Issues on `matemolnar8/coomer` are the tracker; use `gh`. A merged PR is the implementation record.
