use image::{ImageBuffer, Rgba};

const ICON_SIZE: u32 = 22;
const RETINA_ICON_SIZE: u32 = 44;
const BAR_HEIGHT: u32 = 9;
const GAP: u32 = 2;
const BAR_LEFT: u32 = 2;
const BAR_RIGHT_MARGIN: u32 = 2;

const GREEN: [u8; 4] = [0x4a, 0xde, 0x80, 255];
const BG: [u8; 4] = [0x3a, 0x3a, 0x4a, 255];

pub fn render_tray_icon(
    session_util: f64,
    weekly_util: f64,
    session_color: Option<[u8; 4]>,
    weekly_color: Option<[u8; 4]>,
) -> Vec<u8> {
    render_at_size(
        RETINA_ICON_SIZE,
        session_util,
        weekly_util,
        session_color.unwrap_or(GREEN),
        weekly_color.unwrap_or(GREEN),
    )
}

pub fn render_default_icon() -> Vec<u8> {
    render_tray_icon(0.0, 0.0, None, None)
}

fn render_at_size(
    size: u32,
    session_util: f64,
    weekly_util: f64,
    session_color: [u8; 4],
    weekly_color: [u8; 4],
) -> Vec<u8> {
    let scale = size as f64 / ICON_SIZE as f64;
    let bar_h = (BAR_HEIGHT as f64 * scale) as u32;
    let gap = (GAP as f64 * scale) as u32;
    let left = (BAR_LEFT as f64 * scale) as u32;
    let right_margin = (BAR_RIGHT_MARGIN as f64 * scale) as u32;
    let bar_width = size - left - right_margin;

    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(size, size);

    // Top bar (session) - starts at top with some padding
    let top_y = (size - bar_h * 2 - gap) / 2;
    draw_bar(&mut img, left, top_y, bar_width, bar_h, session_util, session_color);

    // Bottom bar (weekly)
    let bottom_y = top_y + bar_h + gap;
    draw_bar(&mut img, left, bottom_y, bar_width, bar_h, weekly_util, weekly_color);

    img.into_raw()
}

fn draw_bar(
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    utilization: f64,
    color: [u8; 4],
) {
    let fill_width = ((width as f64) * utilization.clamp(0.0, 1.0)) as u32;

    for dy in 0..height {
        for dx in 0..width {
            let px = if dx < fill_width {
                Rgba(color)
            } else {
                Rgba(BG)
            };
            img.put_pixel(x + dx, y + dy, px);
        }
    }
}
