use core_graphics::image::CGImage;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObjectProtocol};
use objc2::{ClassType, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSAutoresizingMaskOptions, NSBackingStoreType, NSColor, NSCursor, NSEvent,
    NSEventModifierFlags, NSEventType, NSGraphicsContext, NSScreen, NSScreenSaverWindowLevel,
    NSView, NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_core_foundation::CGPoint;
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSRunLoop, NSRunLoopCommonModes};
use objc2_quartz_core::{CADisplayLink, CAFrameRateRange};
use std::cell::RefCell;

use crate::{hud, render};

mod config {
    pub(super) mod zoom {
        pub const DEFAULT: f64 = 1.0;
        pub const MIN: f64 = 1.0;
        pub const MAX: f64 = 4.0;
        pub const SCROLL_FACTOR_PRECISE: f64 = 0.004;
        pub const SCROLL_FACTOR_LINE: f64 = 0.07;
        pub const KEYBOARD_MULTIPLIER: f64 = 1.08;
        pub const EPSILON: f64 = 1e-9;
    }

    pub(super) mod flashlight {
        pub const DEFAULT_RADIUS: f64 = 144.0;
        pub const MIN_RADIUS: f64 = 24.0;
        pub const MAX_RADIUS: f64 = 320.0;
        pub const SCROLL_FACTOR_PRECISE: f64 = 2.5;
        pub const SCROLL_FACTOR_LINE: f64 = 12.0;
        pub const TOGGLE_DURATION_SECS: f64 = 0.18;
    }

    pub(super) mod fade_in {
        pub const DURATION_SECS: f64 = 0.8;
    }

    pub(super) mod key {
        pub const FLASHLIGHT_TOGGLE: u16 = 3;
        pub const QUIT: u16 = 12;
        pub const ZOOM_IN: u16 = 24;
        pub const ZOOM_OUT: u16 = 27;
        pub const RESET: u16 = 29;
        pub const ESCAPE: u16 = 53;
    }
}

pub const DEFAULT_ZOOM: f64 = config::zoom::DEFAULT;
pub const DEFAULT_FLASHLIGHT_RADIUS: f64 = config::flashlight::DEFAULT_RADIUS;

pub struct DrawState {
    pub image: CGImage,
    pub zoom: f64,
    pub pointer_view: NSPoint,
    pub image_origin: NSPoint,
    pub drag_anchor_view: Option<NSPoint>,
    pub last_frame_timestamp: Option<f64>,

    pub fade_in_progress: f64,
    pub fade_in_elapsed_secs: f64,
    pub fade_in_animating: bool,

    pub flashlight_enabled: bool,
    pub flashlight_progress: f64,
    pub flashlight_radius: f64,
    pub flashlight_animation_from: f64,
    pub flashlight_animation_elapsed_secs: f64,
    pub flashlight_animating: bool,
}

thread_local! {
    static SESSION: RefCell<Option<DrawState>> = const { RefCell::new(None) };
}

pub fn set_session(state: DrawState) {
    SESSION.with(|c| {
        *c.borrow_mut() = Some(state);
    });
}

pub fn clear_session() {
    SESSION.with(|c| {
        *c.borrow_mut() = None;
    });
}

pub fn with_session_mut<R>(f: impl FnOnce(&mut DrawState) -> R) -> Option<R> {
    SESSION.with(|c| c.borrow_mut().as_mut().map(f))
}

fn flashlight_target_progress(st: &DrawState) -> f64 {
    if st.flashlight_enabled { 1.0 } else { 0.0 }
}

fn ease_in_out(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn advance_animation(elapsed_secs: &mut f64, duration_secs: f64, frame_delta_secs: f64) -> f64 {
    *elapsed_secs = (*elapsed_secs + frame_delta_secs).min(duration_secs);
    (*elapsed_secs / duration_secs).clamp(0.0, 1.0)
}

fn update_flashlight_animation(st: &mut DrawState, frame_delta_secs: f64) -> bool {
    if !st.flashlight_animating {
        return false;
    }

    let target = flashlight_target_progress(st);
    let t = advance_animation(
        &mut st.flashlight_animation_elapsed_secs,
        config::flashlight::TOGGLE_DURATION_SECS,
        frame_delta_secs,
    );
    st.flashlight_progress =
        st.flashlight_animation_from + (target - st.flashlight_animation_from) * ease_in_out(t);

    if t >= 1.0 {
        st.flashlight_progress = target;
        st.flashlight_animation_elapsed_secs = 0.0;
        st.flashlight_animating = false;
        return false;
    }

    true
}

fn update_fade_in_animation(st: &mut DrawState, frame_delta_secs: f64) -> bool {
    if !st.fade_in_animating {
        return false;
    }

    let t = advance_animation(
        &mut st.fade_in_elapsed_secs,
        config::fade_in::DURATION_SECS,
        frame_delta_secs,
    );
    st.fade_in_progress = ease_in_out(t);

    if t >= 1.0 {
        st.fade_in_progress = 1.0;
        st.fade_in_elapsed_secs = 0.0;
        st.fade_in_animating = false;
        return false;
    }

    true
}

fn step_overlay_animations(
    st: &mut DrawState,
    frame_timestamp: f64,
    fallback_delta_secs: f64,
) -> bool {
    let frame_delta_secs = st
        .last_frame_timestamp
        .map(|prev| frame_timestamp - prev)
        .unwrap_or(fallback_delta_secs)
        .clamp(0.0, 0.25);
    st.last_frame_timestamp = Some(frame_timestamp);

    let animating_flashlight = update_flashlight_animation(st, frame_delta_secs);
    let animating_fade_in = update_fade_in_animation(st, frame_delta_secs);
    animating_flashlight || animating_fade_in
}

fn start_flashlight_animation(st: &mut DrawState, enabled: bool) {
    st.flashlight_enabled = enabled;
    st.flashlight_animation_from = st.flashlight_progress;
    st.flashlight_animation_elapsed_secs = 0.0;
    st.flashlight_animating = true;
    st.last_frame_timestamp = None;
}

fn reset_state(st: &mut DrawState) {
    st.zoom = DEFAULT_ZOOM;
    st.pointer_view = NSPoint { x: 0.0, y: 0.0 };
    st.image_origin = NSPoint { x: 0.0, y: 0.0 };
    st.drag_anchor_view = None;
    st.flashlight_enabled = false;
    st.flashlight_progress = 0.0;
    st.flashlight_radius = DEFAULT_FLASHLIGHT_RADIUS;
    st.flashlight_animation_from = 0.0;
    st.flashlight_animation_elapsed_secs = 0.0;
    st.flashlight_animating = false;
}

fn clamp_image_origin(origin: NSPoint, bounds: NSRect, zoom: f64) -> NSPoint {
    let z = zoom.clamp(config::zoom::MIN, config::zoom::MAX);
    if z <= 1.0 + config::zoom::EPSILON {
        return NSPoint { x: 0.0, y: 0.0 };
    }
    let w = bounds.size.width;
    let h = bounds.size.height;
    let sw = w * z;
    let sh = h * z;
    let min_ox = w - sw;
    let min_oy = h - sh;
    NSPoint {
        x: origin.x.clamp(min_ox, 0.0),
        y: origin.y.clamp(min_oy, 0.0),
    }
}

fn anchor_zoom_to_cursor(st: &mut DrawState, bounds: NSRect, px: f64, py: f64, new_zoom: f64) {
    let new_zoom = new_zoom.clamp(config::zoom::MIN, config::zoom::MAX);
    if new_zoom <= 1.0 + config::zoom::EPSILON {
        st.zoom = config::zoom::MIN;
        st.image_origin = NSPoint { x: 0.0, y: 0.0 };
        return;
    }
    let z0 = st.zoom;
    if z0 <= 1.0 + config::zoom::EPSILON {
        st.image_origin = NSPoint {
            x: px * (1.0 - new_zoom),
            y: py * (1.0 - new_zoom),
        };
        st.zoom = new_zoom;
        st.image_origin = clamp_image_origin(st.image_origin, bounds, st.zoom);
        return;
    }
    let ratio = new_zoom / z0;
    st.image_origin = NSPoint {
        x: px - (px - st.image_origin.x) * ratio,
        y: py - (py - st.image_origin.y) * ratio,
    };
    st.zoom = new_zoom;
    st.image_origin = clamp_image_origin(st.image_origin, bounds, st.zoom);
}

fn zoom_keyboard_anchored(st: &mut DrawState, bounds: NSRect, px: f64, py: f64, direction: i32) {
    let factor = if direction > 0 {
        config::zoom::KEYBOARD_MULTIPLIER
    } else {
        1.0 / config::zoom::KEYBOARD_MULTIPLIER
    };
    let new_zoom = (st.zoom * factor).clamp(config::zoom::MIN, config::zoom::MAX);
    anchor_zoom_to_cursor(st, bounds, px, py, new_zoom);
}

fn event_point_in_view(ev: &NSEvent, view: &CoomerView) -> NSPoint {
    view.as_super()
        .convertPoint_fromView(ev.locationInWindow(), None)
}

fn point_delta(from: NSPoint, to: NSPoint) -> NSPoint {
    NSPoint {
        x: to.x - from.x,
        y: to.y - from.y,
    }
}

fn stop_overlay(mtm: MainThreadMarker, window: &CoomerWindow) {
    MONITOR.with(|c| {
        if let Some(m) = c.borrow_mut().take() {
            unsafe {
                NSEvent::removeMonitor(&m);
            }
        }
    });
    DISPLAY_LINK.with(|c| {
        if let Some(display_link) = c.borrow_mut().take() {
            display_link.invalidate();
        }
    });
    NSCursor::unhide();
    hud::clear();
    clear_session();
    let app = NSApplication::sharedApplication(mtm);
    window.as_super().orderOut(None);
    app.stop(None);
}

define_class!(
    #[unsafe(super(NSWindow))]
    #[thread_kind = MainThreadOnly]
    #[ivars = ()]
    pub struct CoomerWindow;

    impl CoomerWindow {
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key_window(&self) -> bool {
            true
        }

        #[unsafe(method(canBecomeMainWindow))]
        fn can_become_main_window(&self) -> bool {
            false
        }
    }

    unsafe impl NSObjectProtocol for CoomerWindow {}
);

define_class!(
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[ivars = ()]
    pub struct CoomerView;

    impl CoomerView {
        #[unsafe(method(isOpaque))]
        fn is_opaque(&self) -> bool {
            false
        }

        #[unsafe(method(acceptsFirstResponder))]
        fn accepts_first_responder(&self) -> bool {
            true
        }

        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, _rect: NSRect) {
            SESSION.with(|c| {
                let b = c.borrow();
                let Some(st) = b.as_ref() else {
                    return;
                };
                let Some(ns_ctx) = NSGraphicsContext::currentContext() else {
                    return;
                };
                let bounds = self.bounds();
                let cg_ctx = ns_ctx.CGContext();
                let zoom = st.zoom.clamp(config::zoom::MIN, config::zoom::MAX);
                let pointer = CGPoint {
                    x: st.pointer_view.x as _,
                    y: st.pointer_view.y as _,
                };
                let image_origin = CGPoint {
                    x: st.image_origin.x as _,
                    y: st.image_origin.y as _,
                };
                render::draw_session(
                    &cg_ctx,
                    bounds,
                    &st.image,
                    zoom,
                    pointer,
                    image_origin,
                    st.flashlight_progress,
                    st.flashlight_radius,
                    st.fade_in_progress,
                );
            });
        }

        #[unsafe(method(stepAnimation:))]
        fn step_animation(&self, display_link: &CADisplayLink) {
            let frame_timestamp = display_link.targetTimestamp();
            let fallback_delta_secs = (display_link.targetTimestamp() - display_link.timestamp())
                .clamp(0.0, 0.25);
            let animating = with_session_mut(|st| {
                step_overlay_animations(st, frame_timestamp, fallback_delta_secs)
            })
            .unwrap_or(false);
            self.setNeedsDisplay(true);
            if !animating {
                pause_display_link();
            }
        }
    }

    unsafe impl NSObjectProtocol for CoomerView {}
);

thread_local! {
    static MONITOR: RefCell<Option<Retained<AnyObject>>> = const { RefCell::new(None) };
    static DISPLAY_LINK: RefCell<Option<Retained<CADisplayLink>>> = const { RefCell::new(None) };
}

fn pause_display_link() {
    DISPLAY_LINK.with(|c| {
        if let Some(display_link) = c.borrow().as_ref() {
            display_link.setPaused(true);
        }
    });
}

fn ensure_display_link(view: &CoomerView) {
    DISPLAY_LINK.with(|c| {
        if let Some(display_link) = c.borrow().as_ref() {
            display_link.setPaused(false);
            return;
        }

        let display_link = unsafe {
            view.as_super()
                .displayLinkWithTarget_selector(view.as_super(), sel!(stepAnimation:))
        };
        display_link.setPreferredFrameRateRange(CAFrameRateRange::new(60.0, 120.0, 120.0));
        unsafe {
            display_link.addToRunLoop_forMode(&NSRunLoop::currentRunLoop(), NSRunLoopCommonModes);
        }
        *c.borrow_mut() = Some(display_link);
    });
}

pub fn spawn_window(
    mtm: MainThreadMarker,
    screen: &NSScreen,
    window_frame: NSRect,
) -> Result<(Retained<CoomerWindow>, Retained<CoomerView>), String> {
    let this = CoomerWindow::alloc(mtm).set_ivars(());
    let window: Option<Retained<CoomerWindow>> = unsafe {
        msg_send![
            super(this),
            initWithContentRect: window_frame,
            styleMask: NSWindowStyleMask::Borderless,
            backing: NSBackingStoreType::Buffered,
            defer: false,
        ]
    };
    let window = window.ok_or("initWithContentRect failed for CoomerWindow")?;

    let w = window.as_super();
    w.setFrame_display(window_frame, false);
    w.setLevel(NSScreenSaverWindowLevel);
    w.setOpaque(false);
    w.setBackgroundColor(Some(&NSColor::clearColor()));
    w.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary,
    );
    w.setIgnoresMouseEvents(false);
    w.setAcceptsMouseMovedEvents(true);

    let content_size = window_frame.size;
    let view_frame = NSRect {
        origin: NSPoint { x: 0.0, y: 0.0 },
        size: content_size,
    };

    let v_this = CoomerView::alloc(mtm).set_ivars(());
    let view: Option<Retained<CoomerView>> =
        unsafe { msg_send![super(v_this), initWithFrame: view_frame] };
    let view = view.ok_or("CoomerView init failed")?;

    let v = view.as_super();
    v.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
    );
    w.setContentView(Some(v));
    hud::mount(mtm, v, screen);
    w.makeFirstResponder(Some(v));
    ensure_display_link(&view);
    Ok((window, view))
}

pub fn install_local_monitor(
    mtm: MainThreadMarker,
    view: Retained<CoomerView>,
    window: Retained<CoomerWindow>,
) -> Retained<AnyObject> {
    let mask = crate::input::local_monitor_mask();
    let mtm_for = mtm;
    let block = block2::RcBlock::new(move |event: core::ptr::NonNull<NSEvent>| -> *mut NSEvent {
        let ev = unsafe { event.as_ref() };
        let ty = ev.r#type();

        if ty == NSEventType::KeyDown {
            match ev.keyCode() {
                config::key::QUIT | config::key::ESCAPE => {
                    stop_overlay(mtm_for, &window);
                    return core::ptr::null_mut();
                }
                config::key::FLASHLIGHT_TOGGLE => {
                    with_session_mut(|st| {
                        start_flashlight_animation(st, !st.flashlight_enabled);
                    });
                    ensure_display_link(&view);
                    view.setNeedsDisplay(true);
                    return core::ptr::null_mut();
                }
                config::key::RESET => {
                    let pointer = event_point_in_view(ev, &view);
                    with_session_mut(|st| {
                        reset_state(st);
                        st.pointer_view = pointer;
                    });
                    view.setNeedsDisplay(true);
                    return core::ptr::null_mut();
                }
                config::key::ZOOM_IN => {
                    let bounds = view.as_super().bounds();
                    with_session_mut(|st| {
                        let p = st.pointer_view;
                        zoom_keyboard_anchored(st, bounds, p.x, p.y, 1);
                    });
                    view.setNeedsDisplay(true);
                    return core::ptr::null_mut();
                }
                config::key::ZOOM_OUT => {
                    let bounds = view.as_super().bounds();
                    with_session_mut(|st| {
                        let p = st.pointer_view;
                        zoom_keyboard_anchored(st, bounds, p.x, p.y, -1);
                    });
                    view.setNeedsDisplay(true);
                    return core::ptr::null_mut();
                }
                _ => {}
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
            let point = event_point_in_view(ev, &view);
            let precise = ev.hasPreciseScrollingDeltas();
            let bounds = view.as_super().bounds();
            with_session_mut(|st| {
                st.pointer_view = point;
                let cmd = ev.modifierFlags().contains(NSEventModifierFlags::Command);
                if cmd && (st.flashlight_enabled || st.flashlight_progress > 0.0) {
                    let k = if precise {
                        config::flashlight::SCROLL_FACTOR_PRECISE
                    } else {
                        config::flashlight::SCROLL_FACTOR_LINE
                    };
                    st.flashlight_radius = (st.flashlight_radius + dy * k).clamp(
                        config::flashlight::MIN_RADIUS,
                        config::flashlight::MAX_RADIUS,
                    );
                } else {
                    let line_factor = 1.0 + dy * config::zoom::SCROLL_FACTOR_LINE;
                    let factor = if precise {
                        1.0 + dy * config::zoom::SCROLL_FACTOR_PRECISE
                    } else {
                        line_factor
                    };
                    let new_zoom = (st.zoom * factor).clamp(config::zoom::MIN, config::zoom::MAX);
                    anchor_zoom_to_cursor(st, bounds, point.x, point.y, new_zoom);
                }
            });
            view.setNeedsDisplay(true);
            return event.as_ptr();
        }

        if ty == NSEventType::LeftMouseDown {
            let point = event_point_in_view(ev, &view);
            with_session_mut(|st| {
                st.pointer_view = point;
                st.drag_anchor_view = Some(point);
            });
            view.setNeedsDisplay(true);
            return event.as_ptr();
        }

        if ty == NSEventType::LeftMouseDragged {
            let point = event_point_in_view(ev, &view);
            let bounds = view.as_super().bounds();
            with_session_mut(|st| {
                if let Some(anchor) = st.drag_anchor_view {
                    if st.zoom > 1.0 + config::zoom::EPSILON {
                        let d = point_delta(anchor, point);
                        st.image_origin.x += d.x;
                        st.image_origin.y += d.y;
                        st.image_origin = clamp_image_origin(st.image_origin, bounds, st.zoom);
                    }
                    st.drag_anchor_view = Some(point);
                }
                st.pointer_view = point;
            });
            view.setNeedsDisplay(true);
            return event.as_ptr();
        }

        if ty == NSEventType::LeftMouseUp {
            let point = event_point_in_view(ev, &view);
            with_session_mut(|st| {
                st.pointer_view = point;
                st.drag_anchor_view = None;
            });
            view.setNeedsDisplay(true);
            return event.as_ptr();
        }

        if ty == NSEventType::MouseMoved {
            let point = event_point_in_view(ev, &view);
            with_session_mut(|st| {
                st.pointer_view = point;
            });
            view.setNeedsDisplay(true);
            return event.as_ptr();
        }

        event.as_ptr()
    });

    let monitor = unsafe { NSEvent::addLocalMonitorForEventsMatchingMask_handler(mask, &block) }
        .expect("addLocalMonitorForEventsMatchingMask");
    MONITOR.with(|c| {
        *c.borrow_mut() = Some(monitor.clone());
    });
    monitor
}
