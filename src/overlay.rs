use core_graphics::image::CGImage;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObjectProtocol};
use objc2::{define_class, msg_send, ClassType, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSAutoresizingMaskOptions, NSBackingStoreType, NSColor, NSCursor, NSEvent,
    NSEventModifierFlags, NSEventType, NSGraphicsContext, NSScreen, NSView, NSWindow,
    NSWindowCollectionBehavior, NSWindowStyleMask, NSScreenSaverWindowLevel,
};
use objc2_core_foundation::CGPoint;
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect};
use std::cell::RefCell;

use crate::render;

pub const ZOOM_LEVELS: [f64; 7] = [1.0, 1.25, 1.5, 2.0, 2.5, 3.0, 4.0];

pub struct DrawState {
    pub image: CGImage,
    pub zoom_index: usize,
    pub center_view: NSPoint,
    pub command_down: bool,
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
                let cg_ctx = ns_ctx.CGContext();
                let bounds = self.bounds();
                let zi = st.zoom_index.min(ZOOM_LEVELS.len() - 1);
                let zoom = ZOOM_LEVELS[zi];
                let center = CGPoint {
                    x: st.center_view.x as _,
                    y: st.center_view.y as _,
                };
                render::draw_session(
                    &cg_ctx,
                    bounds,
                    &st.image,
                    zoom,
                    center,
                    st.command_down,
                );
            });
        }
    }

    unsafe impl NSObjectProtocol for CoomerView {}
);

thread_local! {
    static MONITOR: RefCell<Option<Retained<AnyObject>>> = const { RefCell::new(None) };
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
            screen: Some(screen),
        ]
    };
    let window = window.ok_or("initWithContentRect failed for CoomerWindow")?;

    let w = window.as_super();
    w.setLevel(NSScreenSaverWindowLevel);
    w.setOpaque(true);
    w.setBackgroundColor(Some(&NSColor::blackColor()));
    w.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces | NSWindowCollectionBehavior::FullScreenAuxiliary,
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
        if ty == NSEventType::LeftMouseDown
            || ty == NSEventType::RightMouseDown
            || ty == NSEventType::OtherMouseDown
        {
            MONITOR.with(|c| {
                if let Some(m) = c.borrow_mut().take() {
                    unsafe {
                        NSEvent::removeMonitor(&m);
                    }
                }
            });
            NSCursor::unhide();
            clear_session();
            let app = NSApplication::sharedApplication(mtm_for);
            window.as_super().orderOut(None);
            app.stop(None);
            return event.as_ptr();
        }
        if ty == NSEventType::ScrollWheel {
            let dy = ev.scrollingDeltaY();
            let step = if dy > 0.0 {
                1i32
            } else if dy < 0.0 {
                -1
            } else {
                0
            };
            if step != 0 {
                with_session_mut(|st| {
                    let n = ZOOM_LEVELS.len();
                    let zi = st.zoom_index as i32 + step;
                    st.zoom_index = zi.clamp(0, (n - 1) as i32) as usize;
                });
                view.setNeedsDisplay(true);
            }
            return event.as_ptr();
        }
        if ty == NSEventType::FlagsChanged || ty == NSEventType::MouseMoved {
            let flags = ev.modifierFlags();
            let cmd = flags.contains(NSEventModifierFlags::Command);
            with_session_mut(|st| {
                st.command_down = cmd;
            });
            let mouse = NSEvent::mouseLocation();
            let wp = window.as_super().convertPointFromScreen(mouse);
            with_session_mut(|st| {
                let vp = view.as_super().convertPoint_fromView(wp, None);
                st.center_view = vp;
            });
            view.setNeedsDisplay(true);
            return event.as_ptr();
        }
        event.as_ptr()
    });

    let monitor = unsafe {
        NSEvent::addLocalMonitorForEventsMatchingMask_handler(mask, &block)
    }
    .expect("addLocalMonitorForEventsMatchingMask");
    MONITOR.with(|c| {
        *c.borrow_mut() = Some(monitor.clone());
    });
    monitor
}

pub fn hide_cursor() {
    NSCursor::hide();
}
