use objc2::rc::Retained;
use objc2::MainThreadOnly;
use objc2_app_kit::{
    NSAnimatablePropertyContainer, NSAnimationContext, NSAutoresizingMaskOptions, NSColor, NSFont,
    NSGlassEffectView, NSGlassEffectViewStyle, NSImage, NSImageView, NSScreen, NSTextField, NSView,
};
use objc2_foundation::{
    MainThreadMarker, NSEdgeInsets, NSPoint, NSRect, NSSize, NSString, NSTimer,
};
use std::cell::RefCell;

mod config {
    pub(super) const ANIMATION_DELAY_SECS: f64 = 1.0;
    pub(super) const ANIMATION_DURATION_SECS: f64 = 0.6;
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
        hud.glass.setFrame(layout.glass_frame);
        hud.glass.setAlphaValue(1.0);
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

fn animate_to_settled() {
    let hud = HUD.with(|slot| {
        let mut hud_slot = slot.borrow_mut();
        let hud = hud_slot.as_mut()?;
        if hud.settled {
            return None;
        }
        hud.settled = true;
        Some((
            hud.glass.clone(),
            hud.icon.clone(),
            hud.title.clone(),
            hud.hint.clone(),
        ))
    });
    let Some((glass, icon, title, hint)) = hud else {
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
    title.setHidden(false);
    title.setAlphaValue(1.0);

    let title_for_layout = title.clone();
    let title_for_fade = title.clone();
    let changes = block2::RcBlock::new(move |ctx: core::ptr::NonNull<NSAnimationContext>| {
        let ctx = unsafe { ctx.as_ref() };
        ctx.setDuration(config::ANIMATION_DURATION_SECS);
        ctx.setAllowsImplicitAnimation(true);

        glass.animator().setFrame(layout.glass_frame);
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
