use core_graphics::access::ScreenCaptureAccess;
use objc2_app_kit::{NSAlert, NSAlertStyle, NSApplication};
use objc2_foundation::{MainThreadMarker, NSString};

use crate::app::cleanup_pidfile;

pub fn ensure_screen_capture_access(mtm: MainThreadMarker, app: &NSApplication) {
    let access = ScreenCaptureAccess;
    if access.preflight() {
        return;
    }

    #[allow(deprecated)]
    {
        app.activateIgnoringOtherApps(true);
    }

    let alert = NSAlert::new(mtm);
    alert.setMessageText(&NSString::from_str("Screen Recording required"));
    alert.setInformativeText(&NSString::from_str(
        "Coomer will now request screen recording permission. If you chose Deny in the system prompt, enable Screen Recording for this app in \
         System Settings → Privacy & Security → Screen Recording.",
    ));
    alert.setAlertStyle(NSAlertStyle::Warning);
    alert.addButtonWithTitle(&NSString::from_str("OK"));

    let _ = alert.runModal();

    if access.request() {
        return;
    }

    println!("Screen Recording permission not granted");

    cleanup_pidfile();
    std::process::exit(0);
}
