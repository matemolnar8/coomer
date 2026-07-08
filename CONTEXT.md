# coomer

coomer is a macOS agent that presents a frozen, shareable zoom overlay for presentations and screen shares. This context defines the domain language for the overlay experience, visual controls, launch flow, and screen-sharing behavior.

## Language

### Overlay experience

**coomer**:
The macOS agent that presents a frozen, shareable zoom overlay for presentations and screen shares.
_Avoid_: App when the product identity matters

**Overlay**:
The full-screen visual layer that displays the frozen **Screenshot** and its visual effects.
_Avoid_: HUD, window, view

**Overlay Session**:
One active run from launch until the overlay quits or is replaced by a new launch.
_Avoid_: Run, instance, session

**Screenshot**:
The frozen pixels captured from the display under the pointer at launch.
_Avoid_: Snapshot, frozen capture, image

**Display Capture**:
The act of acquiring the **Screenshot** from the display under the pointer.
_Avoid_: Screenshot when referring to the act

**HUD**:
The small top glass pill that shows the app identity and keyboard hints.
_Avoid_: Overlay, controls, toolbar

**Fade In**:
The brief launch animation that clears the initial white wash from the **Overlay**.
_Avoid_: Flash, intro

### Visual controls

**Zoom**:
The feature that magnifies the **Screenshot** inside the **Overlay**.
_Avoid_: Magnification, scale

**Zoom Factor**:
The numeric multiplier applied to the **Screenshot** during **Zoom**.
_Avoid_: Zoom, scale

**Panning**:
Moving the zoomed **Screenshot** within the **Overlay** by dragging.
_Avoid_: Dragging, scrolling

**Flashlight**:
The toggleable effect that dims the **Screenshot** except for a circular clear area around the pointer.
_Avoid_: Spotlight, mask, aperture

**Flashlight Radius**:
The size of the clear circular area used by the **Flashlight**.
_Avoid_: Flashlight size, radius

**Dimmed Region**:
The darkened part of the **Screenshot** outside the **Flashlight** circle.
_Avoid_: Shadow, overlay tint

### Input and launch

**Pointer**:
The current mouse location inside the **Overlay**.
_Avoid_: Cursor when referring to geometry

**Pointer Anchor**:
The point that stays visually stable while changing the **Zoom Factor**.
_Avoid_: Zoom center, cursor point

**Reset**:
The command that restores default **Zoom**, **Panning**, and **Flashlight** state.
_Avoid_: Restart, clear

**Quit**:
The command that ends the current **Overlay Session**.
_Avoid_: Close, stop

**Raycast Trigger**:
A Raycast script command or action that launches `coomer`.
_Avoid_: Shortcut, launcher

**Replacement Launch**:
A launch that terminates the previous **Overlay Session** before starting a new one.
_Avoid_: Relaunch, duplicate launch

### Screen sharing and permissions

**Screen Share**:
A meeting or recording capture of normal desktop pixels that should include the **Overlay**.
_Avoid_: Zoom when referring to screen sharing generally

**Zoom Meeting App**:
The video meeting app used as one possible **Screen Share** target.
_Avoid_: Zoom when referring to the coomer feature

**Screen Recording Permission**:
The macOS privacy permission required for **Display Capture**.
_Avoid_: Capture permission, accessibility permission

**Dock-less Agent**:
The app bundle mode that runs without a Dock icon.
_Avoid_: Background app, daemon
