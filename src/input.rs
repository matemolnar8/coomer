use core::ptr::NonNull;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::{NSEvent, NSEventMask, NSEventModifierFlags, NSEventType, NSView};
use objc2_foundation::{MainThreadMarker, NSPoint};
use std::cell::RefCell;
use std::rc::Rc;

mod key {
    pub const FLASHLIGHT_TOGGLE: u16 = 3;
    pub const QUIT: u16 = 12;
    pub const ZOOM_IN: u16 = 24;
    pub const ZOOM_OUT: u16 = 27;
    pub const RESET: u16 = 29;
    pub const ESCAPE: u16 = 53;
}

pub fn local_monitor_mask() -> NSEventMask {
    NSEventMask::ScrollWheel
        | NSEventMask::KeyDown
        | NSEventMask::KeyUp
        | NSEventMask::LeftMouseDown
        | NSEventMask::LeftMouseDragged
        | NSEventMask::LeftMouseUp
        | NSEventMask::MouseMoved
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OverlayIntent {
    Quit,
    ToggleFlashlight,
    Reset {
        pointer_view: NSPoint,
    },
    ZoomIn,
    ZoomOut,
    ScrollWheel {
        pointer_view: NSPoint,
        dy: f64,
        precise: bool,
        command: bool,
    },
    LeftMouseDown {
        pointer_view: NSPoint,
    },
    LeftMouseDragged {
        pointer_view: NSPoint,
    },
    LeftMouseUp {
        pointer_view: NSPoint,
    },
    PointerMoved {
        pointer_view: NSPoint,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentResult {
    Consume,
    PassThrough,
    StopOverlay,
}

pub trait OverlayIntentSink {
    fn handle(&mut self, intent: OverlayIntent) -> IntentResult;
}

thread_local! {
    static ACTIVE_MONITOR: RefCell<Option<Retained<AnyObject>>> = const { RefCell::new(None) };
}

pub fn remove_overlay_monitor() {
    ACTIVE_MONITOR.with(|c| {
        if let Some(m) = c.borrow_mut().take() {
            unsafe {
                NSEvent::removeMonitor(&m);
            }
        }
    });
}

pub struct LocalMonitorGuard;

impl Drop for LocalMonitorGuard {
    fn drop(&mut self) {
        remove_overlay_monitor();
    }
}

fn event_point_in_view(ev: &NSEvent, view: &NSView) -> NSPoint {
    view.convertPoint_fromView(ev.locationInWindow(), None)
}

fn overlay_intent_for_key_down(key_code: u16, pointer_view: NSPoint) -> Option<OverlayIntent> {
    match key_code {
        key::QUIT | key::ESCAPE => Some(OverlayIntent::Quit),
        key::FLASHLIGHT_TOGGLE => Some(OverlayIntent::ToggleFlashlight),
        key::RESET => Some(OverlayIntent::Reset { pointer_view }),
        key::ZOOM_IN => Some(OverlayIntent::ZoomIn),
        key::ZOOM_OUT => Some(OverlayIntent::ZoomOut),
        _ => None,
    }
}

pub fn install_overlay_monitor<S: OverlayIntentSink + 'static>(
    _mtm: MainThreadMarker,
    view: Retained<NSView>,
    sink: S,
) -> LocalMonitorGuard {
    remove_overlay_monitor();

    let sink = Rc::new(RefCell::new(sink));
    let view_for_block = view.clone();
    let block = block2::RcBlock::new(move |event: NonNull<NSEvent>| -> *mut NSEvent {
        let ev = unsafe { event.as_ref() };
        let ty = ev.r#type();

        if ty == NSEventType::KeyDown {
            let pointer_view = event_point_in_view(ev, &view_for_block);
            let intent = overlay_intent_for_key_down(ev.keyCode(), pointer_view);
            if let Some(intent) = intent {
                match sink.borrow_mut().handle(intent) {
                    IntentResult::PassThrough => return event.as_ptr(),
                    IntentResult::Consume | IntentResult::StopOverlay => {
                        return core::ptr::null_mut();
                    }
                }
            }
            return core::ptr::null_mut();
        }

        if ty == NSEventType::KeyUp {
            return core::ptr::null_mut();
        }

        if ty == NSEventType::ScrollWheel {
            let dy = ev.scrollingDeltaY();
            if dy == 0.0 {
                return event.as_ptr();
            }
            let point = event_point_in_view(ev, &view_for_block);
            let precise = ev.hasPreciseScrollingDeltas();
            let command = ev.modifierFlags().contains(NSEventModifierFlags::Command);
            return match sink.borrow_mut().handle(OverlayIntent::ScrollWheel {
                pointer_view: point,
                dy,
                precise,
                command,
            }) {
                IntentResult::PassThrough => event.as_ptr(),
                IntentResult::Consume | IntentResult::StopOverlay => core::ptr::null_mut(),
            };
        }
        if ty == NSEventType::LeftMouseDown {
            let point = event_point_in_view(ev, &view_for_block);
            return match sink.borrow_mut().handle(OverlayIntent::LeftMouseDown {
                pointer_view: point,
            }) {
                IntentResult::PassThrough => event.as_ptr(),
                IntentResult::Consume | IntentResult::StopOverlay => core::ptr::null_mut(),
            };
        }
        if ty == NSEventType::LeftMouseDragged {
            let point = event_point_in_view(ev, &view_for_block);
            return match sink.borrow_mut().handle(OverlayIntent::LeftMouseDragged {
                pointer_view: point,
            }) {
                IntentResult::PassThrough => event.as_ptr(),
                IntentResult::Consume | IntentResult::StopOverlay => core::ptr::null_mut(),
            };
        }
        if ty == NSEventType::LeftMouseUp {
            let point = event_point_in_view(ev, &view_for_block);
            return match sink.borrow_mut().handle(OverlayIntent::LeftMouseUp {
                pointer_view: point,
            }) {
                IntentResult::PassThrough => event.as_ptr(),
                IntentResult::Consume | IntentResult::StopOverlay => core::ptr::null_mut(),
            };
        }
        if ty == NSEventType::MouseMoved {
            let point = event_point_in_view(ev, &view_for_block);
            return match sink.borrow_mut().handle(OverlayIntent::PointerMoved {
                pointer_view: point,
            }) {
                IntentResult::PassThrough => event.as_ptr(),
                IntentResult::Consume | IntentResult::StopOverlay => core::ptr::null_mut(),
            };
        }
        event.as_ptr()
    });

    let monitor = unsafe {
        NSEvent::addLocalMonitorForEventsMatchingMask_handler(local_monitor_mask(), &block)
    }
    .expect("addLocalMonitorForEventsMatchingMask");

    ACTIVE_MONITOR.with(|c| {
        *c.borrow_mut() = Some(monitor.clone());
    });

    LocalMonitorGuard
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intent_result_distinct() {
        assert_ne!(IntentResult::Consume, IntentResult::PassThrough);
        assert_ne!(IntentResult::Consume, IntentResult::StopOverlay);
    }

    #[test]
    fn key_down_maps_quit_and_escape() {
        let p = NSPoint { x: 1.0, y: 2.0 };
        assert_eq!(
            overlay_intent_for_key_down(key::QUIT, p),
            Some(OverlayIntent::Quit)
        );
        assert_eq!(
            overlay_intent_for_key_down(key::ESCAPE, p),
            Some(OverlayIntent::Quit)
        );
    }

    #[test]
    fn key_down_reset_carries_pointer() {
        let p = NSPoint { x: 10.0, y: 20.0 };
        assert_eq!(
            overlay_intent_for_key_down(key::RESET, p),
            Some(OverlayIntent::Reset { pointer_view: p })
        );
    }

    #[test]
    fn key_down_unknown_yields_none() {
        assert_eq!(
            overlay_intent_for_key_down(0xFFFF, NSPoint { x: 0.0, y: 0.0 }),
            None
        );
    }
}
