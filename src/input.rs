use objc2_app_kit::NSEventMask;

pub fn local_monitor_mask() -> NSEventMask {
    NSEventMask::ScrollWheel
        | NSEventMask::LeftMouseDown
        | NSEventMask::RightMouseDown
        | NSEventMask::OtherMouseDown
        | NSEventMask::MouseMoved
        | NSEventMask::FlagsChanged
}
