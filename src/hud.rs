use objc2::MainThreadOnly;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSAnimatablePropertyContainer, NSAnimationContext, NSAutoresizingMaskOptions, NSColor, NSFont,
    NSGlassEffectView, NSGlassEffectViewStyle, NSImage, NSImageView, NSScreen, NSTextField, NSView,
    NSWorkspace,
};
use objc2_foundation::{
    MainThreadMarker, NSEdgeInsets, NSPoint, NSRect, NSSize, NSString, NSTimer,
};
use std::cell::RefCell;

mod config {
    pub(super) const ANIMATION_DELAY_SECS: f64 = 1.0;
    pub(super) const ANIMATION_DURATION_SECS: f64 = 0.6;
    pub(super) const VISIBILITY_SLIDE_SECS: f64 = 0.3;
    pub(super) const VISIBILITY_FADE_SECS: f64 = 0.3;
    pub(super) const FLASHLIGHT_OVERLAP_PADDING: f64 = 6.0;
    pub(super) const TOP_CORRIDOR_PADDING_X: f64 = 18.0;
    pub(super) const OFFSCREEN_GAP: f64 = 2.0;
    pub(super) const LAUNCH_WIDTH: f64 = 288.0;
    pub(super) const SETTLED_WIDTH: f64 = 144.0;
    pub(super) const LAUNCH_HEIGHT: f64 = 58.0;
    pub(super) const SETTLED_HEIGHT: f64 = 46.0;
    pub(super) const CORNER_RADIUS: f64 = SETTLED_HEIGHT / 2.0;
    pub(super) const TOP_MARGIN_LAUNCH: f64 = 20.0;
    pub(super) const TOP_MARGIN_SETTLED: f64 = 14.0;
    pub(super) const LAUNCH_PADDING_X: f64 = 20.0;
    pub(super) const SETTLED_PADDING_X: f64 = 12.0;
    pub(super) const LAUNCH_GAP: f64 = 10.0;
    pub(super) const SETTLED_GAP: f64 = 8.0;
    pub(super) const LAUNCH_HINT_WIDTH: f64 = 86.0;
    pub(super) const SETTLED_HINT_WIDTH: f64 = 80.0;
    pub(super) const NOTCH_CLEARANCE: f64 = 6.0;
}

struct OverlayHud {
    glass: Retained<NSGlassEffectView>,
    content: Retained<NSView>,
    icon: Option<Retained<NSImageView>>,
    title: Retained<NSTextField>,
    hint: Retained<NSTextField>,
    settled: bool,
    hidden: bool,
    reduce_motion: bool,
    notch_top_offset: f64,
}

struct HudLayout {
    glass_frame: NSRect,
    icon_frame: Option<NSRect>,
    title_frame: NSRect,
    hint_frame: NSRect,
    title_alpha: f64,
}

thread_local! {
    static HUD: RefCell<Option<OverlayHud>> = const { RefCell::new(None) };
    static HUD_ANIMATION_TIMER: RefCell<Option<Retained<NSTimer>>> = const { RefCell::new(None) };
}

fn notch_top_offset(screen: &NSScreen, launch_top_margin: f64) -> f64 {
    let left = screen.auxiliaryTopLeftArea();
    let right = screen.auxiliaryTopRightArea();
    let notch_width = right.origin.x - (left.origin.x + left.size.width);
    if notch_width <= 1.0 {
        return 0.0;
    }

    let frame = screen.frame();
    let screen_top = frame.origin.y + frame.size.height;
    let notch_bottom = left.origin.y.min(right.origin.y);
    let notch_depth = (screen_top - notch_bottom).max(0.0);

    (notch_depth + config::NOTCH_CLEARANCE - launch_top_margin).max(0.0)
}

fn fallback_notch_offset(screen: &NSScreen, launch_top_margin: f64) -> f64 {
    let NSEdgeInsets { top, .. } = screen.safeAreaInsets();
    (top + config::NOTCH_CLEARANCE - launch_top_margin).max(0.0)
}

fn point_in_rect(point: NSPoint, rect: NSRect) -> bool {
    point.x >= rect.origin.x
        && point.x <= rect.origin.x + rect.size.width
        && point.y >= rect.origin.y
        && point.y <= rect.origin.y + rect.size.height
}

fn expand_rect(rect: NSRect, dx: f64, dy: f64) -> NSRect {
    NSRect {
        origin: NSPoint {
            x: rect.origin.x - dx,
            y: rect.origin.y - dy,
        },
        size: NSSize {
            width: rect.size.width + dx * 2.0,
            height: rect.size.height + dy * 2.0,
        },
    }
}

fn circle_intersects_rect(center: NSPoint, radius: f64, rect: NSRect) -> bool {
    let closest_x = center
        .x
        .clamp(rect.origin.x, rect.origin.x + rect.size.width);
    let closest_y = center
        .y
        .clamp(rect.origin.y, rect.origin.y + rect.size.height);
    let dx = center.x - closest_x;
    let dy = center.y - closest_y;
    dx * dx + dy * dy <= radius * radius
}

fn top_corridor_rect(bounds: NSRect, home_frame: NSRect) -> Option<NSRect> {
    let top = bounds.origin.y + bounds.size.height;
    let bottom = home_frame.origin.y + home_frame.size.height;
    let height = top - bottom;
    if height <= 0.0 {
        return None;
    }

    Some(NSRect {
        origin: NSPoint {
            x: home_frame.origin.x - config::TOP_CORRIDOR_PADDING_X,
            y: bottom,
        },
        size: NSSize {
            width: home_frame.size.width + config::TOP_CORRIDOR_PADDING_X * 2.0,
            height,
        },
    })
}

fn hidden_frame(bounds: NSRect, home_frame: NSRect) -> NSRect {
    NSRect {
        origin: NSPoint {
            x: home_frame.origin.x,
            y: (bounds.origin.y + bounds.size.height + config::OFFSCREEN_GAP).round(),
        },
        size: home_frame.size,
    }
}

fn target_frame(bounds: NSRect, home_frame: NSRect, hidden: bool) -> NSRect {
    if hidden {
        hidden_frame(bounds, home_frame)
    } else {
        home_frame
    }
}

fn presentation_frame(
    bounds: NSRect,
    home_frame: NSRect,
    hidden: bool,
    reduce_motion: bool,
) -> NSRect {
    if reduce_motion {
        home_frame
    } else {
        target_frame(bounds, home_frame, hidden)
    }
}

fn presentation_alpha(hidden: bool) -> f64 {
    if hidden { 0.0 } else { 1.0 }
}

fn reduce_motion_enabled() -> bool {
    NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion()
}

fn hud_layout(bounds: NSRect, settled: bool, notch_top_offset: f64) -> HudLayout {
    let width = if settled {
        config::SETTLED_WIDTH
    } else {
        config::LAUNCH_WIDTH
    };
    let height = if settled {
        config::SETTLED_HEIGHT
    } else {
        config::LAUNCH_HEIGHT
    };
    let top_margin = if settled {
        config::TOP_MARGIN_SETTLED
    } else {
        config::TOP_MARGIN_LAUNCH
    } + notch_top_offset;
    let glass_frame = NSRect {
        origin: NSPoint {
            x: ((bounds.size.width - width) * 0.5).round(),
            y: (bounds.size.height - top_margin - height).round(),
        },
        size: NSSize { width, height },
    };
    let content_bounds = NSRect {
        origin: NSPoint { x: 0.0, y: 0.0 },
        size: glass_frame.size,
    };
    let pad_x = if settled {
        config::SETTLED_PADDING_X
    } else {
        config::LAUNCH_PADDING_X
    };
    let icon_size = if settled { 18.0 } else { 22.0 };
    let gap = if settled {
        config::SETTLED_GAP
    } else {
        config::LAUNCH_GAP
    };
    let hint_width = if settled {
        config::SETTLED_HINT_WIDTH
    } else {
        config::LAUNCH_HINT_WIDTH
    };
    let baseline_y = ((content_bounds.size.height - 18.0) * 0.5).round();
    let mut text_x = pad_x;
    let icon_frame = Some(NSRect {
        origin: NSPoint {
            x: pad_x.round(),
            y: ((content_bounds.size.height - icon_size) * 0.5).round(),
        },
        size: NSSize {
            width: icon_size,
            height: icon_size,
        },
    });
    text_x += icon_size + gap;

    let hint_frame = NSRect {
        origin: NSPoint {
            x: (content_bounds.size.width - pad_x - hint_width).round(),
            y: baseline_y,
        },
        size: NSSize {
            width: hint_width,
            height: 18.0,
        },
    };
    let title_frame = NSRect {
        origin: NSPoint {
            x: text_x.round(),
            y: baseline_y,
        },
        size: NSSize {
            width: (content_bounds.size.width - text_x - hint_width - pad_x - 8.0).max(96.0),
            height: 18.0,
        },
    };

    HudLayout {
        glass_frame,
        icon_frame,
        title_frame,
        hint_frame,
        title_alpha: if settled { 0.0 } else { 1.0 },
    }
}

fn make_hud_label(
    mtm: MainThreadMarker,
    text: &str,
    font_size: f64,
    emphasized: bool,
) -> Retained<NSTextField> {
    let label = NSTextField::labelWithString(&NSString::from_str(text), mtm);
    let font = if emphasized {
        NSFont::boldSystemFontOfSize(font_size)
    } else {
        NSFont::systemFontOfSize(font_size)
    };
    let text_color = if emphasized {
        NSColor::labelColor()
    } else {
        NSColor::secondaryLabelColor()
    };
    label.setFont(Some(&font));
    label.setUsesSingleLineMode(true);
    label.setTextColor(Some(&text_color));
    label
}

pub(crate) fn mount(mtm: MainThreadMarker, host_view: &NSView, screen: &NSScreen) {
    clear();

    let bounds = host_view.bounds();
    let notch_offset = notch_top_offset(screen, config::TOP_MARGIN_LAUNCH);
    let notch_offset = if notch_offset > 0.0 {
        notch_offset
    } else {
        fallback_notch_offset(screen, config::TOP_MARGIN_LAUNCH)
    };
    let glass = NSGlassEffectView::initWithFrame(
        NSGlassEffectView::alloc(mtm),
        hud_layout(bounds, false, notch_offset).glass_frame,
    );
    glass.setStyle(NSGlassEffectViewStyle::Regular);
    glass.setCornerRadius(config::CORNER_RADIUS);
    glass.setTintColor(None);
    glass.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewMinXMargin
            | NSAutoresizingMaskOptions::ViewMaxXMargin
            | NSAutoresizingMaskOptions::ViewMinYMargin,
    );

    let content = NSView::initWithFrame(NSView::alloc(mtm), glass.bounds());
    content.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
    );
    glass.setContentView(Some(&content));

    let icon = NSImage::imageWithSystemSymbolName_accessibilityDescription(
        &NSString::from_str("record.circle.fill"),
        Some(&NSString::from_str("Overlay active")),
    )
    .or_else(|| {
        NSImage::imageWithSystemSymbolName_accessibilityDescription(
            &NSString::from_str("circle.fill"),
            Some(&NSString::from_str("Overlay active")),
        )
    })
    .map(|image| {
        let view = NSImageView::imageViewWithImage(&image, mtm);
        view.setContentTintColor(Some(&NSColor::controlAccentColor()));
        view
    });

    let title = make_hud_label(mtm, "Coomer", 13.0, true);
    let hint = make_hud_label(mtm, "Esc to close", 12.0, false);

    if let Some(icon) = &icon {
        content.addSubview(icon);
    }
    content.addSubview(&title);
    content.addSubview(&hint);

    host_view.addSubview(&glass);
    glass.setAlphaValue(1.0);

    HUD.with(|slot| {
        *slot.borrow_mut() = Some(OverlayHud {
            glass,
            content,
            icon,
            title,
            hint,
            settled: false,
            hidden: false,
            reduce_motion: reduce_motion_enabled(),
            notch_top_offset: notch_offset,
        });
    });

    apply_layout(bounds, false);
    schedule_settle();
}

fn apply_layout(bounds: NSRect, settled: bool) {
    HUD.with(|slot| {
        let hud_slot = slot.borrow();
        let Some(hud) = hud_slot.as_ref() else {
            return;
        };

        let layout = hud_layout(bounds, settled, hud.notch_top_offset);
        let reduce_motion = reduce_motion_enabled();
        hud.glass.setFrame(presentation_frame(
            bounds,
            layout.glass_frame,
            hud.hidden,
            reduce_motion,
        ));
        hud.glass.setAlphaValue(presentation_alpha(hud.hidden));
        let content_bounds = hud.glass.bounds();
        hud.content.setFrame(content_bounds);

        if let Some(icon) = &hud.icon {
            if let Some(icon_frame) = layout.icon_frame {
                icon.setFrame(icon_frame);
            }
        }

        hud.hint.setFrame(layout.hint_frame);
        let title_view = &hud.title;
        if !title_view.isDescendantOf(&hud.content) {
            hud.content.addSubview(title_view);
        }
        title_view.setHidden(settled);
        title_view.setAlphaValue(layout.title_alpha);
        title_view.setFrame(layout.title_frame);
    });
}

pub(crate) fn update_visibility(
    pointer_view: NSPoint,
    flashlight_radius: f64,
    flashlight_progress: f64,
) {
    let Some((glass, target, alpha, reduce_motion)) = HUD.with(|slot| {
        let mut hud_slot = slot.borrow_mut();
        let hud = hud_slot.as_mut()?;
        let bounds = unsafe { hud.glass.superview() }?.bounds();
        let home_frame = hud_layout(bounds, hud.settled, hud.notch_top_offset).glass_frame;
        let reduce_motion = reduce_motion_enabled();
        let hover = point_in_rect(pointer_view, home_frame);
        let flashlight_visible = flashlight_progress > 0.0;
        let flashlight_radius =
            flashlight_radius * (0.9 + 0.1 * flashlight_progress.clamp(0.0, 1.0));
        let flashlight_overlap = flashlight_visible
            && circle_intersects_rect(
                pointer_view,
                flashlight_radius,
                expand_rect(
                    home_frame,
                    config::FLASHLIGHT_OVERLAP_PADDING,
                    config::FLASHLIGHT_OVERLAP_PADDING,
                ),
            );
        let in_top_corridor = top_corridor_rect(bounds, home_frame)
            .map(|rect| point_in_rect(pointer_view, rect))
            .unwrap_or(false);
        let hidden = hover || flashlight_overlap || in_top_corridor;
        if hud.hidden == hidden && hud.reduce_motion == reduce_motion {
            return None;
        }

        hud.hidden = hidden;
        hud.reduce_motion = reduce_motion;
        Some((
            hud.glass.clone(),
            presentation_frame(bounds, home_frame, hidden, reduce_motion),
            presentation_alpha(hidden),
            reduce_motion,
        ))
    }) else {
        return;
    };

    if reduce_motion {
        glass.setFrame(target);
        let changes = block2::RcBlock::new(move |ctx: core::ptr::NonNull<NSAnimationContext>| {
            let ctx = unsafe { ctx.as_ref() };
            ctx.setDuration(config::VISIBILITY_FADE_SECS);
            ctx.setAllowsImplicitAnimation(true);
            glass.animator().setAlphaValue(alpha);
        });
        NSAnimationContext::runAnimationGroup(&changes);
    } else {
        glass.setAlphaValue(1.0);
        let changes = block2::RcBlock::new(move |ctx: core::ptr::NonNull<NSAnimationContext>| {
            let ctx = unsafe { ctx.as_ref() };
            ctx.setDuration(config::VISIBILITY_SLIDE_SECS);
            ctx.setAllowsImplicitAnimation(true);
            glass.animator().setFrame(target);
            glass.animator().setAlphaValue(alpha);
        });
        NSAnimationContext::runAnimationGroup(&changes);
    }
}

fn animate_to_settled() {
    let hud = HUD.with(|slot| {
        let mut hud_slot = slot.borrow_mut();
        let hud = hud_slot.as_mut()?;
        if hud.settled {
            return None;
        }
        hud.settled = true;
        hud.reduce_motion = reduce_motion_enabled();
        Some((
            hud.glass.clone(),
            hud.icon.clone(),
            hud.title.clone(),
            hud.hint.clone(),
            hud.hidden,
            hud.reduce_motion,
        ))
    });
    let Some((glass, icon, title, hint, hidden, reduce_motion)) = hud else {
        return;
    };

    let bounds = unsafe { glass.superview() }
        .map(|view| view.bounds())
        .unwrap_or_else(|| glass.frame());
    let notch_top_offset = HUD.with(|slot| {
        slot.borrow()
            .as_ref()
            .map(|hud| hud.notch_top_offset)
            .unwrap_or(0.0)
    });
    let layout = hud_layout(bounds, true, notch_top_offset);
    let glass_frame = presentation_frame(bounds, layout.glass_frame, hidden, reduce_motion);
    let glass_alpha = presentation_alpha(hidden);
    title.setHidden(false);
    title.setAlphaValue(1.0);

    let title_for_layout = title.clone();
    let title_for_fade = title.clone();
    let changes = block2::RcBlock::new(move |ctx: core::ptr::NonNull<NSAnimationContext>| {
        let ctx = unsafe { ctx.as_ref() };
        ctx.setDuration(config::ANIMATION_DURATION_SECS);
        ctx.setAllowsImplicitAnimation(true);

        glass.animator().setFrame(glass_frame);
        glass.animator().setAlphaValue(glass_alpha);
        if let Some(icon) = &icon {
            if let Some(icon_frame) = layout.icon_frame {
                icon.animator().setFrame(icon_frame);
            }
        }
        hint.animator().setFrame(layout.hint_frame);
        title_for_layout.animator().setFrame(layout.title_frame);
        title_for_fade.animator().setAlphaValue(0.0);
    });
    let title_for_hide = title.clone();
    let completion = block2::RcBlock::new(move || {
        title_for_hide.setHidden(true);
    });
    NSAnimationContext::runAnimationGroup_completionHandler(&changes, Some(&completion));
}

fn clear_timer() {
    HUD_ANIMATION_TIMER.with(|slot| {
        if let Some(timer) = slot.borrow_mut().take() {
            timer.invalidate();
        }
    });
}

fn schedule_settle() {
    clear_timer();
    let block = block2::RcBlock::new(move |_timer: core::ptr::NonNull<NSTimer>| {
        clear_timer();
        animate_to_settled();
    });
    let timer = unsafe {
        NSTimer::scheduledTimerWithTimeInterval_repeats_block(
            config::ANIMATION_DELAY_SECS,
            false,
            &block,
        )
    };
    HUD_ANIMATION_TIMER.with(|slot| {
        *slot.borrow_mut() = Some(timer);
    });
}

pub(crate) fn clear() {
    clear_timer();
    HUD.with(|slot| {
        if let Some(hud) = slot.borrow_mut().take() {
            hud.glass.removeFromSuperview();
        }
    });
}
