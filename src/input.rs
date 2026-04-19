use objc2_app_kit::NSEventMask;

pub fn local_monitor_mask() -> NSEventMask {
    NSEventMask::ScrollWheel
        | NSEventMask::KeyDown
        | NSEventMask::KeyUp
        | NSEventMask::LeftMouseDown
        | NSEventMask::LeftMouseDragged
        | NSEventMask::LeftMouseUp
        | NSEventMask::MouseMoved
}
