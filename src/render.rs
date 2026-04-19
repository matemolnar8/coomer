use core_graphics::image::CGImage as SysCgImage;
use foreign_types::ForeignType;
use objc2_core_foundation::{CGFloat, CGPoint, CGRect, CGSize};
use objc2_core_graphics::{CGContext, CGImage};
use objc2_foundation::NSRect;

fn nsrect_to_cgrect(r: NSRect) -> CGRect {
    CGRect::new(
        CGPoint::new(r.origin.x as CGFloat, r.origin.y as CGFloat),
        CGSize::new(r.size.width as CGFloat, r.size.height as CGFloat),
    )
}

pub fn draw_session(
    cg_ctx: &CGContext,
    view_bounds: NSRect,
    image: &SysCgImage,
    zoom: f64,
    center: CGPoint,
    show_ring: bool,
) {
    let bounds = nsrect_to_cgrect(view_bounds);
    let objc_img: &CGImage = unsafe { &*(ForeignType::as_ptr(image).cast::<CGImage>()) };
    let c = Some(cg_ctx);
    CGContext::save_g_state(c);
    CGContext::translate_ctm(c, center.x, center.y);
    CGContext::scale_ctm(c, zoom as CGFloat, zoom as CGFloat);
    CGContext::translate_ctm(c, -center.x, -center.y);
    CGContext::draw_image(c, bounds, Some(objc_img));
    CGContext::restore_g_state(c);
    if show_ring {
        let radius: CGFloat = 72.0;
        let ring = CGRect::new(
            CGPoint::new(center.x - radius, center.y - radius),
            CGSize::new(radius * 2.0, radius * 2.0),
        );
        CGContext::save_g_state(c);
        CGContext::set_line_width(c, 4.0);
        CGContext::set_rgb_stroke_color(c, 1.0, 0.85, 0.2, 1.0);
        CGContext::stroke_ellipse_in_rect(c, ring);
        CGContext::restore_g_state(c);
    }
}
