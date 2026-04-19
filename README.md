# coomer

macOS agent: frozen full-screen capture of the display under the cursor, with zoom, bounded pan while zoomed, and a toggleable flashlight overlay. Intended to show up in screen shares because it draws its own fullscreen window instead of using Accessibility zoom.

## Controls


| Control                   | Description                                |
| ------------------------- | ------------------------------------------ |
| `0`                       | Reset application state.                   |
| `q` or `Esc`              | Quit the application.                      |
| `f`                       | Toggle the flashlight effect.              |
| Left mouse drag           | Pan the screenshot (only while zoomed in). |
| Scroll wheel or `=` / `-` | Smooth continuous zoom in or out.          |
| `Command` + scroll wheel  | Smooth continuous flashlight radius.       |


## Build

```sh
cargo build --release
```

## App bundle (Dock-less agent)

```sh
./scripts/package-app.sh
```

Install `packaging/Coomer.app` wherever you like, then point Raycast at `Contents/MacOS/coomer`.

`Info.plist` sets `LSUIElement` so there is no Dock icon when launched from the bundle.

## Permissions

On first capture, macOS prompts for **Screen Recording** for the binary. Allow it in **System Settings → Privacy & Security → Screen Recording**.

## Raycast

Import the script command from `raycast/` (see `raycast/README.md`) or run the binary from a “Script Command” / “Run shell script” action.

## Repo layout

- `src/capture.rs` — display under cursor + `CGDisplayCreateImage`
- `src/overlay.rs` — `NSWindow` / `NSView`, event monitor, cursor hide
- `src/render.rs` — zoom + ring drawing
- `src/app.rs` — `NSApplication` run loop, single-instance pid file under `~/.coomer/pid`

