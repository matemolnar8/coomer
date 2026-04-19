# Screen share validation (manual)

`coomer` draws a borderless fullscreen `NSWindow` at `NSScreenSaverWindowLevel` with a frozen `CGDisplay` snapshot. It should appear in captures that include normal desktop pixels (Zoom “Screen”, QuickTime Player New Screen Recording, etc.).

## Zoom

1. Start a meeting, share **Screen** (not “Window” only).
2. Run `coomer` (or trigger from Raycast).
3. Confirm participants see the dimmed/zoomed overlay, not only your local view.

## Second app

Repeat with **QuickTime Player → File → New Screen Recording** preview or another desktop capture tool to confirm the overlay is not Zoom-specific.

## Notes

- If capture returns black or empty until you grant permission, re-run after enabling **Screen Recording** for `coomer` in System Settings.
- A second launch terminates the previous `coomer` process (pid file in `~/.coomer/pid`) so Raycast re-triggers do not stack sessions.
