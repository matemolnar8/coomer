# Ubiquitous Language

## Overlay experience

| Term | Definition | Aliases to avoid |
| --- | --- | --- |
| **coomer** | The macOS agent that presents a frozen, shareable zoom overlay for presentations and screen shares. | App when the product identity matters |
| **Overlay** | The full-screen visual layer that displays the frozen **Screenshot** and its visual effects. | HUD, window, view |
| **Overlay Session** | One active run from launch until the overlay quits or is replaced by a new launch. | Run, instance, session |
| **Screenshot** | The frozen pixels captured from the display under the pointer at launch. | Snapshot, frozen capture, image |
| **Display Capture** | The act of acquiring the **Screenshot** from the display under the pointer. | Screenshot when referring to the act |
| **HUD** | The small top glass pill that shows the app identity and keyboard hints. | Overlay, controls, toolbar |
| **Fade In** | The brief launch animation that clears the initial white wash from the **Overlay**. | Flash, intro |

## Visual controls

| Term | Definition | Aliases to avoid |
| --- | --- | --- |
| **Zoom** | The feature that magnifies the **Screenshot** inside the **Overlay**. | Magnification, scale |
| **Zoom Factor** | The numeric multiplier applied to the **Screenshot** during **Zoom**. | Zoom, scale |
| **Panning** | Moving the zoomed **Screenshot** within the **Overlay** by dragging. | Dragging, scrolling |
| **Flashlight** | The toggleable effect that dims the **Screenshot** except for a circular clear area around the pointer. | Spotlight, mask, aperture |
| **Flashlight Radius** | The size of the clear circular area used by the **Flashlight**. | Flashlight size, radius |
| **Dimmed Region** | The darkened part of the **Screenshot** outside the **Flashlight** circle. | Shadow, overlay tint |

## Input and launch

| Term | Definition | Aliases to avoid |
| --- | --- | --- |
| **Pointer** | The current mouse location inside the **Overlay**. | Cursor when referring to geometry |
| **Pointer Anchor** | The point that stays visually stable while changing the **Zoom Factor**. | Zoom center, cursor point |
| **Reset** | The command that restores default **Zoom**, **Panning**, and **Flashlight** state. | Restart, clear |
| **Quit** | The command that ends the current **Overlay Session**. | Close, stop |
| **Raycast Trigger** | A Raycast script command or action that launches `coomer`. | Shortcut, launcher |
| **Replacement Launch** | A launch that terminates the previous **Overlay Session** before starting a new one. | Relaunch, duplicate launch |

## Screen sharing and permissions

| Term | Definition | Aliases to avoid |
| --- | --- | --- |
| **Screen Share** | A meeting or recording capture of normal desktop pixels that should include the **Overlay**. | Zoom when referring to screen sharing generally |
| **Zoom Meeting App** | The video meeting app used as one possible **Screen Share** target. | Zoom when referring to the coomer feature |
| **Screen Recording Permission** | The macOS privacy permission required for **Display Capture**. | Capture permission, accessibility permission |
| **Dock-less Agent** | The app bundle mode that runs without a Dock icon. | Background app, daemon |

## Relationships

- A **Display Capture** produces exactly one **Screenshot** for an **Overlay Session**.
- An **Overlay Session** owns exactly one active **Overlay**.
- An **Overlay** displays exactly one **Screenshot**.
- An **Overlay** may show zero or one **HUD**.
- **Zoom** changes the **Zoom Factor** applied to the **Screenshot**.
- **Panning** is available only when the **Zoom Factor** is greater than `1.0`.
- **Flashlight** has exactly one **Flashlight Radius** while enabled or animating.
- A **Replacement Launch** ends at most one previous **Overlay Session** before starting a new one.
- A **Screen Share** should include the **Overlay** because coomer draws normal desktop pixels.

## Example dialogue

> **Dev:** "When the user launches **coomer**, do we create a new **Overlay Session** every time?"
> **Domain expert:** "Yes, but a **Replacement Launch** first ends the previous **Overlay Session** so two overlays do not stack."
> **Dev:** "During the session, is the **HUD** the whole full-screen layer?"
> **Domain expert:** "No. The **Overlay** is the full-screen layer; the **HUD** is only the small top glass pill."
> **Dev:** "So **Zoom** changes the **Zoom Factor** on the frozen **Screenshot**, while **Panning** moves that screenshot when zoomed?"
> **Domain expert:** "Exactly. **Flashlight** is separate: it dims the screenshot outside the pointer-centered circle, whose size is the **Flashlight Radius**."

## Flagged ambiguities

- "HUD" was previously easy to read as all overlay UI; use **HUD** only for the top glass pill and **Overlay** for the full-screen visual layer.
- "Flashlight" can mean the mode, mask, or clear circle; use **Flashlight** for the dimming effect/mode and **Flashlight Radius** for the clear circle size.
- "Zoom" can mean either the coomer feature or the video meeting app; use **Zoom** for the coomer feature and **Zoom Meeting App** for the app.
- "Screenshot", "snapshot", "capture", and "image" can refer to the same frozen pixels; use **Screenshot** for the frozen pixels and **Display Capture** for acquiring them.
