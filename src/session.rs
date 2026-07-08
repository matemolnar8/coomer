use core_graphics::image::CGImage;
use objc2_foundation::{NSPoint, NSRect};

use crate::input::{IntentResult, OverlayIntent};

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
}

pub struct OverlaySession {
    image: CGImage,
    zoom: f64,
    pointer_view: NSPoint,
    image_origin: NSPoint,
    drag_anchor_view: Option<NSPoint>,
    last_frame_timestamp: Option<f64>,

    fade_in_progress: f64,
    fade_in_elapsed_secs: f64,
    fade_in_animating: bool,

    flashlight_enabled: bool,
    flashlight_progress: f64,
    flashlight_radius: f64,
    flashlight_animation_from: f64,
    flashlight_animation_elapsed_secs: f64,
    flashlight_animating: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionOutcome {
    pub intent_result: IntentResult,
    pub needs_redraw: bool,
    pub needs_animation: bool,
    pub quit: bool,
}

pub struct RenderSnapshot<'a> {
    pub image: &'a CGImage,
    pub zoom: f64,
    pub pointer_view: NSPoint,
    pub image_origin: NSPoint,
    pub flashlight_progress: f64,
    pub flashlight_radius: f64,
    pub fade_in_progress: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HudSnapshot {
    pub pointer_view: NSPoint,
    pub flashlight_radius: f64,
    pub flashlight_progress: f64,
}

impl OverlaySession {
    pub fn new(image: CGImage, pointer_view: NSPoint) -> Self {
        Self {
            image,
            zoom: config::zoom::DEFAULT,
            pointer_view,
            image_origin: NSPoint { x: 0.0, y: 0.0 },
            drag_anchor_view: None,
            last_frame_timestamp: None,
            fade_in_progress: 0.0,
            fade_in_elapsed_secs: 0.0,
            fade_in_animating: true,
            flashlight_enabled: false,
            flashlight_progress: 0.0,
            flashlight_radius: config::flashlight::DEFAULT_RADIUS,
            flashlight_animation_from: 0.0,
            flashlight_animation_elapsed_secs: 0.0,
            flashlight_animating: false,
        }
    }

    pub fn handle(&mut self, intent: OverlayIntent, bounds: NSRect) -> SessionOutcome {
        match intent {
            OverlayIntent::Quit => SessionOutcome {
                intent_result: IntentResult::StopOverlay,
                needs_redraw: false,
                needs_animation: false,
                quit: true,
            },
            OverlayIntent::ToggleFlashlight => {
                self.start_flashlight_animation(!self.flashlight_enabled);
                SessionOutcome::changed(IntentResult::Consume).with_animation()
            }
            OverlayIntent::Reset { pointer_view } => {
                self.reset(pointer_view);
                SessionOutcome::changed(IntentResult::Consume)
            }
            OverlayIntent::ZoomIn => {
                let p = self.pointer_view;
                self.zoom_keyboard_anchored(bounds, p.x, p.y, 1);
                SessionOutcome::changed(IntentResult::Consume)
            }
            OverlayIntent::ZoomOut => {
                let p = self.pointer_view;
                self.zoom_keyboard_anchored(bounds, p.x, p.y, -1);
                SessionOutcome::changed(IntentResult::Consume)
            }
            OverlayIntent::ScrollWheel {
                pointer_view,
                dy,
                precise,
                command,
            } => {
                self.pointer_view = pointer_view;
                if command && (self.flashlight_enabled || self.flashlight_progress > 0.0) {
                    self.adjust_flashlight_radius(dy, precise);
                } else {
                    self.zoom_scroll_anchored(bounds, pointer_view, dy, precise);
                }
                SessionOutcome::changed(IntentResult::PassThrough)
            }
            OverlayIntent::LeftMouseDown { pointer_view } => {
                self.pointer_view = pointer_view;
                self.drag_anchor_view = Some(pointer_view);
                SessionOutcome::changed(IntentResult::PassThrough)
            }
            OverlayIntent::LeftMouseDragged { pointer_view } => {
                self.pan_to(bounds, pointer_view);
                SessionOutcome::changed(IntentResult::PassThrough)
            }
            OverlayIntent::LeftMouseUp { pointer_view } => {
                self.pointer_view = pointer_view;
                self.drag_anchor_view = None;
                SessionOutcome::changed(IntentResult::PassThrough)
            }
            OverlayIntent::PointerMoved { pointer_view } => {
                self.pointer_view = pointer_view;
                SessionOutcome::changed(IntentResult::PassThrough)
            }
        }
    }

    pub fn step_animation(&mut self, frame_timestamp: f64, fallback_delta_secs: f64) -> bool {
        let frame_delta_secs = self
            .last_frame_timestamp
            .map(|prev| frame_timestamp - prev)
            .unwrap_or(fallback_delta_secs)
            .clamp(0.0, 0.25);
        self.last_frame_timestamp = Some(frame_timestamp);

        let animating_flashlight = self.update_flashlight_animation(frame_delta_secs);
        let animating_fade_in = self.update_fade_in_animation(frame_delta_secs);
        animating_flashlight || animating_fade_in
    }

    pub fn render_snapshot(&self) -> RenderSnapshot<'_> {
        RenderSnapshot {
            image: &self.image,
            zoom: self.zoom,
            pointer_view: self.pointer_view,
            image_origin: self.image_origin,
            flashlight_progress: self.flashlight_progress,
            flashlight_radius: self.flashlight_radius,
            fade_in_progress: self.fade_in_progress,
        }
    }

    pub fn hud_snapshot(&self) -> HudSnapshot {
        HudSnapshot {
            pointer_view: self.pointer_view,
            flashlight_radius: self.flashlight_radius,
            flashlight_progress: self.flashlight_progress,
        }
    }

    fn reset(&mut self, pointer_view: NSPoint) {
        self.zoom = config::zoom::DEFAULT;
        self.pointer_view = pointer_view;
        self.image_origin = NSPoint { x: 0.0, y: 0.0 };
        self.drag_anchor_view = None;
        self.flashlight_enabled = false;
        self.flashlight_progress = 0.0;
        self.flashlight_radius = config::flashlight::DEFAULT_RADIUS;
        self.flashlight_animation_from = 0.0;
        self.flashlight_animation_elapsed_secs = 0.0;
        self.flashlight_animating = false;
    }

    fn start_flashlight_animation(&mut self, enabled: bool) {
        self.flashlight_enabled = enabled;
        self.flashlight_animation_from = self.flashlight_progress;
        self.flashlight_animation_elapsed_secs = 0.0;
        self.flashlight_animating = true;
        self.last_frame_timestamp = None;
    }

    fn update_flashlight_animation(&mut self, frame_delta_secs: f64) -> bool {
        if !self.flashlight_animating {
            return false;
        }

        let target = self.flashlight_target_progress();
        let t = advance_animation(
            &mut self.flashlight_animation_elapsed_secs,
            config::flashlight::TOGGLE_DURATION_SECS,
            frame_delta_secs,
        );
        self.flashlight_progress = self.flashlight_animation_from
            + (target - self.flashlight_animation_from) * ease_in_out(t);

        if t >= 1.0 {
            self.flashlight_progress = target;
            self.flashlight_animation_elapsed_secs = 0.0;
            self.flashlight_animating = false;
            return false;
        }

        true
    }

    fn update_fade_in_animation(&mut self, frame_delta_secs: f64) -> bool {
        if !self.fade_in_animating {
            return false;
        }

        let t = advance_animation(
            &mut self.fade_in_elapsed_secs,
            config::fade_in::DURATION_SECS,
            frame_delta_secs,
        );
        self.fade_in_progress = ease_in_out(t);

        if t >= 1.0 {
            self.fade_in_progress = 1.0;
            self.fade_in_elapsed_secs = 0.0;
            self.fade_in_animating = false;
            return false;
        }

        true
    }

    fn flashlight_target_progress(&self) -> f64 {
        if self.flashlight_enabled { 1.0 } else { 0.0 }
    }

    fn adjust_flashlight_radius(&mut self, dy: f64, precise: bool) {
        let k = if precise {
            config::flashlight::SCROLL_FACTOR_PRECISE
        } else {
            config::flashlight::SCROLL_FACTOR_LINE
        };
        self.flashlight_radius = (self.flashlight_radius + dy * k).clamp(
            config::flashlight::MIN_RADIUS,
            config::flashlight::MAX_RADIUS,
        );
    }

    fn zoom_scroll_anchored(
        &mut self,
        bounds: NSRect,
        pointer_view: NSPoint,
        dy: f64,
        precise: bool,
    ) {
        let line_factor = 1.0 + dy * config::zoom::SCROLL_FACTOR_LINE;
        let factor = if precise {
            1.0 + dy * config::zoom::SCROLL_FACTOR_PRECISE
        } else {
            line_factor
        };
        let new_zoom = (self.zoom * factor).clamp(config::zoom::MIN, config::zoom::MAX);
        self.anchor_zoom_to_pointer(bounds, pointer_view.x, pointer_view.y, new_zoom);
    }

    fn zoom_keyboard_anchored(&mut self, bounds: NSRect, px: f64, py: f64, direction: i32) {
        let factor = if direction > 0 {
            config::zoom::KEYBOARD_MULTIPLIER
        } else {
            1.0 / config::zoom::KEYBOARD_MULTIPLIER
        };
        let new_zoom = (self.zoom * factor).clamp(config::zoom::MIN, config::zoom::MAX);
        self.anchor_zoom_to_pointer(bounds, px, py, new_zoom);
    }

    fn anchor_zoom_to_pointer(&mut self, bounds: NSRect, px: f64, py: f64, new_zoom: f64) {
        let new_zoom = new_zoom.clamp(config::zoom::MIN, config::zoom::MAX);
        if new_zoom <= 1.0 + config::zoom::EPSILON {
            self.zoom = config::zoom::MIN;
            self.image_origin = NSPoint { x: 0.0, y: 0.0 };
            return;
        }

        let z0 = self.zoom;
        if z0 <= 1.0 + config::zoom::EPSILON {
            self.image_origin = NSPoint {
                x: px * (1.0 - new_zoom),
                y: py * (1.0 - new_zoom),
            };
            self.zoom = new_zoom;
            self.image_origin = clamp_image_origin(self.image_origin, bounds, self.zoom);
            return;
        }

        let ratio = new_zoom / z0;
        self.image_origin = NSPoint {
            x: px - (px - self.image_origin.x) * ratio,
            y: py - (py - self.image_origin.y) * ratio,
        };
        self.zoom = new_zoom;
        self.image_origin = clamp_image_origin(self.image_origin, bounds, self.zoom);
    }

    fn pan_to(&mut self, bounds: NSRect, pointer_view: NSPoint) {
        if let Some(anchor) = self.drag_anchor_view {
            if self.zoom > 1.0 + config::zoom::EPSILON {
                let d = point_delta(anchor, pointer_view);
                self.image_origin.x += d.x;
                self.image_origin.y += d.y;
                self.image_origin = clamp_image_origin(self.image_origin, bounds, self.zoom);
            }
            self.drag_anchor_view = Some(pointer_view);
        }
        self.pointer_view = pointer_view;
    }
}

impl SessionOutcome {
    fn changed(intent_result: IntentResult) -> Self {
        Self {
            intent_result,
            needs_redraw: true,
            needs_animation: false,
            quit: false,
        }
    }

    fn with_animation(mut self) -> Self {
        self.needs_animation = true;
        self
    }
}

fn ease_in_out(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn advance_animation(elapsed_secs: &mut f64, duration_secs: f64, frame_delta_secs: f64) -> f64 {
    *elapsed_secs = (*elapsed_secs + frame_delta_secs).min(duration_secs);
    (*elapsed_secs / duration_secs).clamp(0.0, 1.0)
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

fn point_delta(from: NSPoint, to: NSPoint) -> NSPoint {
    NSPoint {
        x: to.x - from.x,
        y: to.y - from.y,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_graphics::base::{kCGImageAlphaPremultipliedLast, kCGRenderingIntentDefault};
    use core_graphics::color_space::CGColorSpace;
    use core_graphics::data_provider::CGDataProvider;
    use objc2_foundation::NSSize;
    use std::sync::Arc;

    fn test_image() -> CGImage {
        let colorspace = CGColorSpace::create_device_rgb();
        let data = Arc::new(vec![0_u8, 0, 0, 255]);
        let provider = CGDataProvider::from_buffer(data);
        CGImage::new(
            1,
            1,
            8,
            32,
            4,
            &colorspace,
            kCGImageAlphaPremultipliedLast,
            &provider,
            false,
            kCGRenderingIntentDefault,
        )
    }

    fn point(x: f64, y: f64) -> NSPoint {
        NSPoint { x, y }
    }

    fn bounds(width: f64, height: f64) -> NSRect {
        NSRect {
            origin: point(0.0, 0.0),
            size: NSSize { width, height },
        }
    }

    fn session_at(pointer_view: NSPoint) -> OverlaySession {
        OverlaySession::new(test_image(), pointer_view)
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "actual {actual} did not match expected {expected}"
        );
    }

    fn assert_point_close(actual: NSPoint, expected: NSPoint) {
        assert_close(actual.x, expected.x);
        assert_close(actual.y, expected.y);
    }

    #[test]
    fn new_session_uses_initial_pointer_without_half_initialized_state() {
        let session = session_at(point(10.0, 20.0));
        let render = session.render_snapshot();

        assert_eq!(render.pointer_view, point(10.0, 20.0));
        assert_close(render.zoom, config::zoom::DEFAULT);
        assert_close(render.flashlight_radius, config::flashlight::DEFAULT_RADIUS);
        assert_close(render.fade_in_progress, 0.0);
    }

    #[test]
    fn scroll_zoom_keeps_pointer_anchor_stable() {
        let mut session = session_at(point(50.0, 40.0));
        session.handle(
            OverlayIntent::ScrollWheel {
                pointer_view: point(50.0, 40.0),
                dy: 10.0,
                precise: false,
                command: false,
            },
            bounds(100.0, 100.0),
        );

        let render = session.render_snapshot();
        assert_close(render.zoom, 1.7);
        assert_point_close(render.image_origin, point(-35.0, -28.0));
    }

    #[test]
    fn panning_clamps_image_origin_to_zoomed_bounds() {
        let mut session = session_at(point(50.0, 50.0));
        session.handle(OverlayIntent::ZoomIn, bounds(100.0, 100.0));
        session.handle(
            OverlayIntent::LeftMouseDown {
                pointer_view: point(50.0, 50.0),
            },
            bounds(100.0, 100.0),
        );
        session.handle(
            OverlayIntent::LeftMouseDragged {
                pointer_view: point(500.0, 500.0),
            },
            bounds(100.0, 100.0),
        );

        assert_eq!(session.render_snapshot().image_origin, point(0.0, 0.0));

        session.handle(
            OverlayIntent::LeftMouseDragged {
                pointer_view: point(-500.0, -500.0),
            },
            bounds(100.0, 100.0),
        );

        let render = session.render_snapshot();
        assert_eq!(render.image_origin, point(-8.0, -8.0));
    }

    #[test]
    fn reset_restores_zoom_panning_and_flashlight_state() {
        let mut session = session_at(point(20.0, 20.0));
        session.handle(OverlayIntent::ToggleFlashlight, bounds(100.0, 100.0));
        session.step_animation(1.0, 0.18);
        session.handle(
            OverlayIntent::ScrollWheel {
                pointer_view: point(20.0, 20.0),
                dy: 10.0,
                precise: false,
                command: true,
            },
            bounds(100.0, 100.0),
        );
        session.handle(OverlayIntent::ZoomIn, bounds(100.0, 100.0));

        session.handle(
            OverlayIntent::Reset {
                pointer_view: point(3.0, 4.0),
            },
            bounds(100.0, 100.0),
        );

        let render = session.render_snapshot();
        assert_close(render.zoom, config::zoom::DEFAULT);
        assert_eq!(render.pointer_view, point(3.0, 4.0));
        assert_eq!(render.image_origin, point(0.0, 0.0));
        assert_close(render.flashlight_progress, 0.0);
        assert_close(render.flashlight_radius, config::flashlight::DEFAULT_RADIUS);
    }

    #[test]
    fn command_scroll_adjusts_flashlight_radius_when_flashlight_is_visible() {
        let mut session = session_at(point(10.0, 10.0));
        let outcome = session.handle(OverlayIntent::ToggleFlashlight, bounds(100.0, 100.0));
        assert!(outcome.needs_animation);
        session.step_animation(1.0, 0.18);

        session.handle(
            OverlayIntent::ScrollWheel {
                pointer_view: point(10.0, 10.0),
                dy: 2.0,
                precise: false,
                command: true,
            },
            bounds(100.0, 100.0),
        );

        assert_close(
            session.hud_snapshot().flashlight_radius,
            config::flashlight::DEFAULT_RADIUS + 24.0,
        );
    }

    #[test]
    fn animation_step_reports_when_animation_finishes() {
        let mut session = session_at(point(10.0, 10.0));
        session.handle(OverlayIntent::ToggleFlashlight, bounds(100.0, 100.0));

        assert!(session.step_animation(1.0, 0.09));
        assert!(session.step_animation(1.25, 0.09));
        assert!(session.step_animation(1.5, 0.09));
        assert!(!session.step_animation(1.75, 0.09));

        let render = session.render_snapshot();
        assert_close(render.flashlight_progress, 1.0);
        assert_close(render.fade_in_progress, 1.0);
    }

    #[test]
    fn quit_returns_adapter_action_without_redraw() {
        let mut session = session_at(point(0.0, 0.0));

        let outcome = session.handle(OverlayIntent::Quit, bounds(100.0, 100.0));

        assert_eq!(outcome.intent_result, IntentResult::StopOverlay);
        assert!(outcome.quit);
        assert!(!outcome.needs_redraw);
        assert!(!outcome.needs_animation);
    }
}
