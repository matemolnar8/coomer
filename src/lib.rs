//! coomer: frozen display capture + fullscreen zoom overlay.

#![cfg_attr(target_os = "macos", deny(clashing_extern_declarations))]

#[cfg(target_os = "macos")]
mod app;
#[cfg(target_os = "macos")]
mod capture;
#[cfg(target_os = "macos")]
mod hud;
#[cfg(target_os = "macos")]
mod input;
#[cfg(target_os = "macos")]
mod overlay;
#[cfg(target_os = "macos")]
mod permissions;
#[cfg(target_os = "macos")]
mod render;

#[cfg(target_os = "macos")]
pub fn run() -> Result<(), String> {
    app::run()
}

#[cfg(not(target_os = "macos"))]
pub fn run() -> Result<(), String> {
    Err("coomer only runs on macOS".into())
}
