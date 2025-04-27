use image::{RgbImage, RgbaImage};

use crate::error::{XCapError, XCapResult};

use super::{
    impl_monitor::ImplMonitor,
    impl_window::ImplWindow,
    utils::{get_current_screen_buf, get_monitor_info_buf, wayland_detect},
    wayland_capture::{wayland_capture, wayland_capture_rgb},
    xorg_capture::{xorg_capture, xorg_capture_rgb},
};

pub fn capture_monitor(impl_monitor: &ImplMonitor) -> XCapResult<RgbaImage> {
    let monitor_info_buf = get_monitor_info_buf(impl_monitor.output)?;

    if wayland_detect() {
        wayland_capture(
            monitor_info_buf.x() as i32,
            monitor_info_buf.y() as i32,
            monitor_info_buf.width() as i32,
            monitor_info_buf.height() as i32,
        )
    } else {
        let screen_buf = get_current_screen_buf()?;

        xorg_capture(
            screen_buf.root(),
            monitor_info_buf.x() as i32,
            monitor_info_buf.y() as i32,
            monitor_info_buf.width() as u32,
            monitor_info_buf.height() as u32,
        )
    }
}

pub fn capture_region(
    impl_monitor: &ImplMonitor,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> XCapResult<RgbaImage> {
    let monitor_info_buf = get_monitor_info_buf(impl_monitor.output)?;

    if wayland_detect() {
        wayland_capture(x, y, width as i32, height as i32)
    } else {
        let screen_buf = get_current_screen_buf()?;

        xorg_capture(
            screen_buf.root(),
            monitor_info_buf.x() as i32 + x,
            monitor_info_buf.y() as i32 + y,
            width,
            height,
        )
    }
}

pub fn capture_region_rgb(
    impl_monitor: &ImplMonitor,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> XCapResult<RgbImage> {
    let monitor_info_buf = get_monitor_info_buf(impl_monitor.output)?;

    // First capture as RGBA
    if wayland_detect() {
        wayland_capture_rgb(x, y, width as i32, height as i32)
    } else {
        let screen_buf = get_current_screen_buf()?;

        xorg_capture_rgb(
            screen_buf.root(),
            monitor_info_buf.x() as i32 + x,
            monitor_info_buf.y() as i32 + y,
            width,
            height,
        )
    }
}

/// Capture a window's content as an RGBA image
pub fn capture_window(impl_window: &ImplWindow) -> XCapResult<RgbaImage> {
    let width = impl_window.width()?;
    let height = impl_window.height()?;

    xorg_capture(impl_window.window, 0, 0, width, height)
}

/// Capture a window's content as an RGB image (more efficient when alpha is not needed)
pub fn capture_window_rgb(impl_window: &ImplWindow) -> XCapResult<RgbImage> {
    let width = impl_window.width()?;
    let height = impl_window.height()?;

    xorg_capture_rgb(impl_window.window, 0, 0, width, height)
}
