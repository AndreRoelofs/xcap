use image::{RgbImage, RgbaImage};
use xcb::{
    x::{Drawable, GetImage, ImageFormat, ImageOrder, Window},
    Connection,
};

use crate::error::{XCapError, XCapResult};

fn get_pixel8_rgba(
    bytes: &[u8],
    x: u32,
    y: u32,
    width: u32,
    bits_per_pixel: u32,
    bit_order: ImageOrder,
) -> (u8, u8, u8, u8) {
    let (r, g, b) = get_pixel8_rgb(bytes, x, y, width, bits_per_pixel, bit_order);
    (r, g, b, 255)
}

fn get_pixel8_rgb(
    bytes: &[u8],
    x: u32,
    y: u32,
    width: u32,
    bits_per_pixel: u32,
    bit_order: ImageOrder,
) -> (u8, u8, u8) {
    let index = ((y * width + x) * bits_per_pixel / 8) as usize;

    let pixel = if bit_order == ImageOrder::LsbFirst {
        bytes[index]
    } else {
        bytes[index] & (7 << 4) | (bytes[index] >> 4)
    };

    // Fast integer scaling instead of floating point math
    // 8-bit pixel format: RRR GGG BB
    // R: bits 6-7, G: bits 3-5, B: bits 0-1
    let r = ((pixel >> 6) * 85) & 0xFF; // 85 = 255/3, multiply instead of division
    let g = (((pixel >> 3) & 7) * 36) & 0xFF; // 36 ~= 255/7
    let b = ((pixel & 3) * 85) & 0xFF; // 85 = 255/3

    (r, g, b)
}

fn get_pixel16_rgba(
    bytes: &[u8],
    x: u32,
    y: u32,
    width: u32,
    bits_per_pixel: u32,
    bit_order: ImageOrder,
) -> (u8, u8, u8, u8) {
    let (r, g, b) = get_pixel16_rgb(bytes, x, y, width, bits_per_pixel, bit_order);
    (r, g, b, 255)
}

fn get_pixel16_rgb(
    bytes: &[u8],
    x: u32,
    y: u32,
    width: u32,
    bits_per_pixel: u32,
    bit_order: ImageOrder,
) -> (u8, u8, u8) {
    let index = ((y * width + x) * bits_per_pixel / 8) as usize;

    let pixel = if bit_order == ImageOrder::LsbFirst {
        bytes[index] as u16 | ((bytes[index + 1] as u16) << 8)
    } else {
        ((bytes[index] as u16) << 8) | bytes[index + 1] as u16
    };

    // Fast integer scaling using bit shifting
    // 16-bit pixel format: RRRRR GGGGGG BBBBB
    // R: bits 11-15, G: bits 5-10, B: bits 0-4
    let r = ((pixel >> 11) * 8) as u8 & 0xFF; // Multiply by 8 ~= 255/31
    let g = (((pixel >> 5) & 63) * 4) as u8 & 0xFF; // Multiply by 4 ~= 255/63
    let b = ((pixel & 31) * 8) as u8 & 0xFF; // Multiply by 8 ~= 255/31

    (r, g, b)
}

fn get_pixel24_32_rgba(
    bytes: &[u8],
    x: u32,
    y: u32,
    width: u32,
    bits_per_pixel: u32,
    bit_order: ImageOrder,
) -> (u8, u8, u8, u8) {
    let index = ((y * width + x) * bits_per_pixel / 8) as usize;

    if bit_order == ImageOrder::LsbFirst {
        (bytes[index + 2], bytes[index + 1], bytes[index], 255)
    } else {
        (bytes[index], bytes[index + 1], bytes[index + 2], 255)
    }
}

fn get_pixel24_32_rgb(
    bytes: &[u8],
    x: u32,
    y: u32,
    width: u32,
    bits_per_pixel: u32,
    bit_order: ImageOrder,
) -> (u8, u8, u8) {
    let index = ((y * width + x) * bits_per_pixel / 8) as usize;

    if bit_order == ImageOrder::LsbFirst {
        (bytes[index + 2], bytes[index + 1], bytes[index])
    } else {
        (bytes[index], bytes[index + 1], bytes[index + 2])
    }
}

pub fn xorg_capture(
    window: Window,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> XCapResult<RgbaImage> {
    let (conn, _) = Connection::connect(None)?;

    let setup = conn.get_setup();

    let get_image_cookie = conn.send_request(&GetImage {
        format: ImageFormat::ZPixmap,
        drawable: Drawable::Window(window),
        x: x as i16,
        y: y as i16,
        width: width as u16,
        height: height as u16,
        plane_mask: u32::MAX,
    });

    let get_image_reply = conn.wait_for_reply(get_image_cookie)?;
    let bytes = get_image_reply.data();
    let depth = get_image_reply.depth();

    let pixmap_format = setup
        .pixmap_formats()
        .iter()
        .find(|item| item.depth() == depth)
        .ok_or(XCapError::new("Not found pixmap format"))?;

    let bits_per_pixel = pixmap_format.bits_per_pixel() as u32;
    let bit_order = setup.bitmap_format_bit_order();

    let get_pixel_rgba = match depth {
        8 => get_pixel8_rgba,
        16 => get_pixel16_rgba,
        24 => get_pixel24_32_rgba,
        32 => get_pixel24_32_rgba,
        _ => return Err(XCapError::new(format!("Unsupported {} depth", depth))),
    };

    let mut rgba = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let index = ((y * width + x) * 4) as usize;
            let (r, g, b, a) = get_pixel_rgba(bytes, x, y, width, bits_per_pixel, bit_order);

            rgba[index] = r;
            rgba[index + 1] = g;
            rgba[index + 2] = b;
            rgba[index + 3] = a;
        }
    }

    RgbaImage::from_raw(width, height, rgba)
        .ok_or_else(|| XCapError::new("RgbaImage::from_raw failed"))
}

/// Capture a window's content directly to an RgbImage for better performance when alpha channel is not needed
pub fn xorg_capture_rgb(
    window: Window,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> XCapResult<RgbImage> {
    // Setup connection to X server
    let (conn, _) = Connection::connect(None)?;
    let setup = conn.get_setup();

    // Request image data
    let get_image_cookie = conn.send_request(&GetImage {
        format: ImageFormat::ZPixmap,
        drawable: Drawable::Window(window),
        x: x as i16,
        y: y as i16,
        width: width as u16,
        height: height as u16,
        plane_mask: u32::MAX,
    });

    // Get image data
    let get_image_reply = conn.wait_for_reply(get_image_cookie)?;
    let bytes = get_image_reply.data();
    let depth = get_image_reply.depth();

    // Get pixmap format information
    let pixmap_format = setup
        .pixmap_formats()
        .iter()
        .find(|item| item.depth() == depth)
        .ok_or(XCapError::new("Not found pixmap format"))?;

    let bits_per_pixel = pixmap_format.bits_per_pixel() as u32;
    let bit_order = setup.bitmap_format_bit_order();

    // Get appropriate pixel conversion function based on depth
    let get_pixel_rgb = match depth {
        8 => get_pixel8_rgb,
        16 => get_pixel16_rgb,
        24 => get_pixel24_32_rgb,
        32 => get_pixel24_32_rgb,
        _ => return Err(XCapError::new(format!("Unsupported {} depth", depth))),
    };
    let mut rgb = vec![0u8; (width * height * 3) as usize];

    for y in 0..height {
        for x in 0..width {
            let index = ((y * width + x) * 3) as usize;
            let (r, g, b) = get_pixel_rgb(bytes, x, y, width, bits_per_pixel, bit_order);

            rgb[index] = r;
            rgb[index + 1] = g;
            rgb[index + 2] = b;
        }
    }

    RgbImage::from_raw(width, height, rgb)
        .ok_or_else(|| XCapError::new("RgbImage::from_raw failed"))
}
