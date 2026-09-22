mod common;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::LinuxBrowserDetector as SystemBrowserDetector;
#[cfg(target_os = "macos")]
pub use macos::MacOsBrowserDetector as SystemBrowserDetector;
#[cfg(target_os = "windows")]
pub use windows::WindowsBrowserDetector as SystemBrowserDetector;

pub fn system_detector() -> SystemBrowserDetector {
    SystemBrowserDetector::new()
}
