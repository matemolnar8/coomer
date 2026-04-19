use core_graphics::display::CGDisplay;
use core_graphics::geometry::CGPoint;
use objc2_app_kit::{NSEvent, NSScreen};
use objc2_foundation::{MainThreadMarker, NSNumber, NSPoint, NSRect, NSString};

pub struct CapturedDisplay {
    pub cg_image: core_graphics::image::CGImage,
    pub screen: objc2::rc::Retained<NSScreen>,
    pub window_frame: NSRect,
}

fn ns_point_in_rect(p: NSPoint, r: NSRect) -> bool {
    p.x >= r.origin.x
        && p.y >= r.origin.y
        && p.x <= r.origin.x + r.size.width
        && p.y <= r.origin.y + r.size.height
}

fn screen_for_mouse(mtm: MainThreadMarker, mouse: NSPoint) -> Option<objc2::rc::Retained<NSScreen>> {
    let screens = NSScreen::screens(mtm);
    let n = screens.count();
    for i in 0..n {
        let s = screens.objectAtIndex(i);
        let f = s.frame();
        if ns_point_in_rect(mouse, f) {
            return Some(s);
        }
    }
    Some(screens.objectAtIndex(0))
}

fn display_id_for_screen(screen: &NSScreen) -> Option<u32> {
    let dict = screen.deviceDescription();
    let key = NSString::from_str("NSScreenNumber");
    let obj = dict.objectForKey(&key)?;
    let n = obj.downcast_ref::<NSNumber>()?;
    Some(n.as_u32())
}

pub fn capture_under_cursor(mtm: MainThreadMarker) -> Result<CapturedDisplay, String> {
    let mouse = NSEvent::mouseLocation();
    let screen = screen_for_mouse(mtm, mouse).ok_or("no NSScreen")?;
    let did = display_id_for_screen(&screen).ok_or("no display id in NSScreen deviceDescription")?;

    let cg_point = CGPoint {
        x: mouse.x,
        y: mouse.y,
    };
    let _ = CGDisplay::displays_with_point(cg_point, 16).ok();

    let d = CGDisplay::new(did);
    let cg_image = d
        .image()
        .ok_or("CGDisplayCreateImage returned null (grant Screen Recording to coomer in System Settings)")?;

    let window_frame = screen.frame();
    Ok(CapturedDisplay {
        cg_image,
        screen,
        window_frame,
    })
}
