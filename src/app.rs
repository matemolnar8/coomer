use crate::capture;
use crate::overlay::{self, DrawState};
use crate::permissions;
use objc2::ClassType;
use objc2_app_kit::NSApplication;
use objc2_foundation::MainThreadMarker;
use std::path::PathBuf;

fn coomer_data_dir() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME").ok_or("HOME not set")?;
    let dir = PathBuf::from(home).join(".coomer");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn replace_running_instance() -> Result<(), String> {
    let dir = coomer_data_dir()?;
    let pidfile = dir.join("pid");
    if pidfile.exists() {
        let txt = std::fs::read_to_string(&pidfile).unwrap_or_default();
        if let Ok(pid) = txt.trim().parse::<libc::pid_t>() {
            if pid > 0 {
                let alive = unsafe { ::libc::kill(pid, 0) } == 0;
                if alive {
                    unsafe {
                        ::libc::kill(pid, ::libc::SIGTERM);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(120));
                }
            }
        }
        let _ = std::fs::remove_file(&pidfile);
    }
    std::fs::write(&pidfile, format!("{}\n", std::process::id())).map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn cleanup_pidfile() {
    if let Ok(dir) = coomer_data_dir() {
        let _ = std::fs::remove_file(dir.join("pid"));
    }
}

pub fn run() -> Result<(), String> {
    replace_running_instance()?;

    let mtm = MainThreadMarker::new().ok_or("coomer must run on the main thread")?;

    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(objc2_app_kit::NSApplicationActivationPolicy::Accessory);
    permissions::ensure_screen_capture_access(mtm, &app);

    let cap = capture::capture_under_cursor(mtm)?;
    let mouse = objc2_app_kit::NSEvent::mouseLocation();

    overlay::set_session(DrawState {
        image: cap.cg_image,
        zoom: overlay::DEFAULT_ZOOM,
        pointer_view: objc2_foundation::NSPoint { x: 0.0, y: 0.0 },
        image_origin: objc2_foundation::NSPoint { x: 0.0, y: 0.0 },
        drag_anchor_view: None,
        last_frame_timestamp: None,
        flashlight_enabled: false,
        flashlight_progress: 0.0,
        flashlight_radius: overlay::DEFAULT_FLASHLIGHT_RADIUS,
        flashlight_animation_from: 0.0,
        flashlight_animation_elapsed_secs: 0.0,
        flashlight_animating: false,
        fade_in_progress: 0.0,
        fade_in_elapsed_secs: 0.0,
        fade_in_animating: true,
    });

    let (window, view) = overlay::spawn_window(mtm, &cap.screen, cap.window_frame)?;

    {
        let wp = window.as_super().convertPointFromScreen(mouse);
        let vp = view.as_super().convertPoint_fromView(wp, None);
        overlay::with_session_mut(|st| {
            st.pointer_view = vp;
        });
    }

    let _monitor = overlay::install_local_monitor(mtm, view.clone(), window.clone());

    #[allow(deprecated)]
    {
        app.activateIgnoringOtherApps(true);
    }
    window.as_super().makeKeyAndOrderFront(None);
    let _ = window.as_super().makeFirstResponder(Some(&*view));
    view.setNeedsDisplay(true);

    app.run();

    overlay::clear_session();
    cleanup_pidfile();
    Ok(())
}
