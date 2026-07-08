use objc2::rc::Retained;
use objc2::runtime::NSObjectProtocol;
use objc2::{MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSAutoresizingMaskOptions, NSBackingStoreType, NSColor, NSCursor,
    NSGraphicsContext, NSScreen, NSScreenSaverWindowLevel, NSView, NSWindow,
    NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_core_foundation::CGPoint;
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSRunLoop, NSRunLoopCommonModes};
use objc2_quartz_core::{CADisplayLink, CAFrameRateRange};
use std::cell::RefCell;

use crate::session::{OverlaySession, SessionOutcome};
use crate::{hud, input, render};

thread_local! {
    static SESSION: RefCell<Option<OverlaySession>> = const { RefCell::new(None) };
}

pub fn start_session(session: OverlaySession) {
    SESSION.with(|c| {
        *c.borrow_mut() = Some(session);
    });
}

pub fn clear_session() {
    SESSION.with(|c| {
        *c.borrow_mut() = None;
    });
}

fn with_session<R>(f: impl FnOnce(&OverlaySession) -> R) -> Option<R> {
    SESSION.with(|c| c.borrow().as_ref().map(f))
}

fn with_session_mut<R>(f: impl FnOnce(&mut OverlaySession) -> R) -> Option<R> {
    SESSION.with(|c| c.borrow_mut().as_mut().map(f))
}

pub(crate) fn refresh_hud_visibility() {
    let _ = with_session(|session| {
        let snapshot = session.hud_snapshot();
        hud::update_visibility(
            snapshot.pointer_view,
            snapshot.flashlight_radius,
            snapshot.flashlight_progress,
        );
    });
}

fn stop_overlay(mtm: MainThreadMarker, window: &CoomerWindow) {
    input::remove_overlay_monitor();
    DISPLAY_LINK.with(|c| {
        if let Some(display_link) = c.borrow_mut().take() {
            display_link.invalidate();
        }
    });
    NSCursor::unhide();
    hud::clear();
    clear_session();
    let app = NSApplication::sharedApplication(mtm);
    window.orderOut(None);
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
                let Some(session) = b.as_ref() else {
                    return;
                };
                let Some(ns_ctx) = NSGraphicsContext::currentContext() else {
                    return;
                };
                let bounds = self.bounds();
                let cg_ctx = ns_ctx.CGContext();
                let snapshot = session.render_snapshot();
                let pointer = CGPoint {
                    x: snapshot.pointer_view.x as _,
                    y: snapshot.pointer_view.y as _,
                };
                let image_origin = CGPoint {
                    x: snapshot.image_origin.x as _,
                    y: snapshot.image_origin.y as _,
                };
                render::draw_session(
                    &cg_ctx,
                    bounds,
                    snapshot.image,
                    snapshot.zoom,
                    pointer,
                    image_origin,
                    snapshot.flashlight_progress,
                    snapshot.flashlight_radius,
                    snapshot.fade_in_progress,
                );
            });
        }

        #[unsafe(method(stepAnimation:))]
        fn step_animation(&self, display_link: &CADisplayLink) {
            let frame_timestamp = display_link.targetTimestamp();
            let fallback_delta_secs = (display_link.targetTimestamp() - display_link.timestamp())
                .clamp(0.0, 0.25);
            let animating = with_session_mut(|session| {
                let animating = session.step_animation(frame_timestamp, fallback_delta_secs);
                let snapshot = session.hud_snapshot();
                hud::update_visibility(
                    snapshot.pointer_view,
                    snapshot.flashlight_radius,
                    snapshot.flashlight_progress,
                );
                animating
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

        let display_link =
            unsafe { view.displayLinkWithTarget_selector(view, sel!(stepAnimation:)) };
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

    window.setFrame_display(window_frame, false);
    window.setLevel(NSScreenSaverWindowLevel);
    window.setOpaque(false);
    window.setBackgroundColor(Some(&NSColor::clearColor()));
    window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary,
    );
    window.setIgnoresMouseEvents(false);
    window.setAcceptsMouseMovedEvents(true);

    let content_size = window_frame.size;
    let view_frame = NSRect {
        origin: NSPoint { x: 0.0, y: 0.0 },
        size: content_size,
    };

    let v_this = CoomerView::alloc(mtm).set_ivars(());
    let view: Option<Retained<CoomerView>> =
        unsafe { msg_send![super(v_this), initWithFrame: view_frame] };
    let view = view.ok_or("CoomerView init failed")?;

    view.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
    );
    window.setContentView(Some(&view));
    hud::mount(mtm, &view, screen);
    window.makeFirstResponder(Some(&view));
    ensure_display_link(&view);
    Ok((window, view))
}

pub fn retained_content_ns_view(view: &Retained<CoomerView>) -> Retained<NSView> {
    view.clone().into_super()
}

pub struct OverlayInputSink {
    mtm: MainThreadMarker,
    window: Retained<CoomerWindow>,
    view: Retained<CoomerView>,
}

impl OverlayInputSink {
    pub fn new(
        mtm: MainThreadMarker,
        window: Retained<CoomerWindow>,
        view: Retained<CoomerView>,
    ) -> Self {
        Self { mtm, window, view }
    }

    fn apply_outcome(&self, outcome: SessionOutcome) -> input::IntentResult {
        if outcome.quit {
            stop_overlay(self.mtm, &self.window);
            return input::IntentResult::StopOverlay;
        }

        if outcome.needs_animation {
            ensure_display_link(&self.view);
        }
        if outcome.needs_redraw {
            refresh_hud_visibility();
            self.view.setNeedsDisplay(true);
        }

        outcome.intent_result
    }
}

impl input::OverlayIntentSink for OverlayInputSink {
    fn handle(&mut self, intent: input::OverlayIntent) -> input::IntentResult {
        let bounds = self.view.bounds();
        with_session_mut(|session| session.handle(intent, bounds))
            .map(|outcome| self.apply_outcome(outcome))
            .unwrap_or(input::IntentResult::PassThrough)
    }
}
