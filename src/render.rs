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
    pointer: CGPoint,
    image_origin: CGPoint,
    flashlight_progress: f64,
    flashlight_radius: f64,
) {
    let bounds = nsrect_to_cgrect(view_bounds);
    let objc_img: &CGImage = unsafe { &*(ForeignType::as_ptr(image).cast::<CGImage>()) };
    let c = Some(cg_ctx);
    let scaled = CGRect::new(
        CGPoint::new(image_origin.x as CGFloat, image_origin.y as CGFloat),
        CGSize::new(
            bounds.size.width * zoom as CGFloat,
            bounds.size.height * zoom as CGFloat,
        ),
    );
    CGContext::draw_image(c, scaled, Some(objc_img));
    let progress = flashlight_progress.clamp(0.0, 1.0) as CGFloat;
    if progress > 0.0 {
        let radius = flashlight_radius as CGFloat * (0.9 + 0.1 * progress);
        let hole = CGRect::new(
            CGPoint::new(pointer.x - radius, pointer.y - radius),
            CGSize::new(radius * 2.0, radius * 2.0),
        );
        CGContext::save_g_state(c);
        CGContext::set_rgb_fill_color(c, 0.0, 0.0, 0.0, 0.65 * progress);
        CGContext::begin_path(c);
        CGContext::add_rect(c, bounds);
        CGContext::add_ellipse_in_rect(c, hole);
        CGContext::eo_fill_path(c);
        CGContext::restore_g_state(c);
    }
}
