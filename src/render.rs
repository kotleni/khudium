use std::collections::HashSet;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use evdev::{Device, InputEventKind};
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};

use crate::layout::KeyboardLayout;
use crate::mouse::MouseState;
use crate::stats::TypingStats;

const BASE_KEY_UNIT: f32 = 36.0;
const BASE_FONT_SIZE: f32 = 12.0;
const STATS_GAP: f32 = 8.0;

pub struct RenderMetrics {
    pub key_unit: f32,
    pub font_size: f32,
    pub padding: f32,
    pub alpha: f32,
}

impl RenderMetrics {
    pub fn new(scale: f32, padding: u32, alpha: f32) -> Self {
        Self {
            key_unit: BASE_KEY_UNIT * scale,
            font_size: BASE_FONT_SIZE * scale,
            padding: padding as f32,
            alpha,
        }
    }
}

pub fn load_font() -> fontdue::Font {
    let paths = [
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/google-noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/TTF/LiberationSans-Regular.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/croscore/Arimo-Regular.ttf",
        "/usr/share/fonts/roboto/Roboto-Regular.ttf",
        "/usr/share/fonts/truetype/roboto/Roboto-Regular.ttf",
        "/usr/share/fonts/ubuntu-font-family/Ubuntu-R.ttf",
        "/usr/share/fonts/gnu-free/FreeSans.ttf",
        "/usr/share/fonts/cascadia-code/CascadiaCode-Regular.ttf",
        "/usr/share/fonts/TTF/FiraCode-Regular.ttf",
        "/usr/share/fonts/fira-code-fonts/FiraCode-Regular.ttf",
        "/usr/share/fonts/jetbrains-mono/JetBrainsMono-Regular.ttf",
        "/usr/share/fonts/truetype/jetbrains/JetBrainsMono-Regular.ttf",
        "/usr/share/fonts/hack/TTF/Hack-Regular.ttf",
        "/usr/share/fonts/truetype/hack/Hack-Regular.ttf",
        "/usr/share/fonts/adwaita/AdwaitaSans-Regular.ttf",
    ];
    for path in &paths {
        if let Ok(data) = std::fs::read(path) {
            return fontdue::Font::from_bytes(&data[..], fontdue::FontSettings::default()).unwrap();
        }
    }
    panic!("No font found. Install a font: pacman -S ttf-dejavu ttf-liberation noto-fonts");
}

pub fn calculate_dimensions(
    layout: &KeyboardLayout,
    metrics: &RenderMetrics,
    mouse_enabled: bool,
) -> (u32, u32) {
    let pad = metrics.padding;
    let max_width = layout
        .rows
        .iter()
        .map(|row| {
            row.iter().map(|k| k.width * metrics.key_unit).sum::<f32>()
                + (row.len() as f32 - 1.0) * pad
        })
        .fold(0.0f32, f32::max);
    let keyboard_height = layout.rows.len() as f32 * metrics.key_unit
        + (layout.rows.len() as f32 - 1.0) * pad;
    let stats_gap = STATS_GAP * (metrics.key_unit / BASE_KEY_UNIT);
    let mouse_height = if mouse_enabled {
        crate::mouse::calculate_mouse_height(metrics)
    } else {
        0.0
    };
    let total_height = pad + metrics.font_size + stats_gap + mouse_height + keyboard_height + pad;
    (
        (max_width + 2.0 * pad) as u32,
        (total_height + 2.0 * pad) as u32,
    )
}

pub fn render_to_buffer(
    pixmap: &mut Pixmap,
    layout: &KeyboardLayout,
    pressed_keys: &HashSet<u16>,
    stats: &TypingStats,
    font: &fontdue::Font,
    metrics: &RenderMetrics,
    mouse_state: Option<&MouseState>,
) {
    pixmap.fill(tiny_skia::Color::TRANSPARENT);
    render_stats_bar(pixmap, stats, font, metrics);

    let stats_gap = STATS_GAP * (metrics.key_unit / BASE_KEY_UNIT);
    let mut keyboard_y = metrics.padding + metrics.font_size + stats_gap;

    if let Some(ms) = mouse_state {
        let mouse_height = crate::mouse::calculate_mouse_height(metrics);
        crate::mouse::render_mouse(pixmap, ms, font, metrics, keyboard_y);
        keyboard_y += mouse_height;
    }

    render_keyboard(pixmap, layout, pressed_keys, font, metrics, keyboard_y);
}

pub fn write_pixmap_to_file(pixmap: &Pixmap, file: &File) {
    let mut file = file.try_clone().unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    file.write_all(pixmap.data()).unwrap();
    file.flush().unwrap();
}

fn render_keyboard(
    pixmap: &mut Pixmap,
    layout: &KeyboardLayout,
    pressed_keys: &HashSet<u16>,
    font: &fontdue::Font,
    metrics: &RenderMetrics,
    y_start: f32,
) {
    let pad = metrics.padding;
    let mut y = y_start;

    for row in &layout.rows {
        let mut x = pad;
        for key in row {
            let key_width = key.width * metrics.key_unit;
            let key_height = metrics.key_unit;

            if !key.label.is_empty() {
                let is_pressed = key.code != 0 && pressed_keys.contains(&key.code);
                let style = KeyStyle {
                    font,
                    font_size: metrics.font_size,
                    alpha: metrics.alpha,
                };
                let key_rect = Rect::from_xywh(x, y, key_width, key_height).unwrap();
                draw_key(pixmap, key_rect, key.label, is_pressed, &style);
            }

            x += key_width + pad;
        }
        y += metrics.key_unit + pad;
    }
}

fn render_stats_bar(
    pixmap: &mut Pixmap,
    stats: &TypingStats,
    font: &fontdue::Font,
    metrics: &RenderMetrics,
) {
    let pad = metrics.padding;
    let text = stats.display_text();
    let a = (180.0 * metrics.alpha).round() as u8;

    let text_width: f32 = text
        .chars()
        .map(|c| font.rasterize(c, metrics.font_size).0.width as f32)
        .sum();

    draw_text(
        pixmap,
        &text,
        pixmap.width() as f32 - pad - text_width / 2.0,
        pad + metrics.font_size / 2.0,
        font,
        metrics.font_size,
        Color::from_rgba8(180, 180, 180, a),
    );
}

struct KeyStyle<'a> {
    font: &'a fontdue::Font,
    font_size: f32,
    alpha: f32,
}

fn draw_key(
    pixmap: &mut Pixmap,
    rect: Rect,
    label: &str,
    pressed: bool,
    style: &KeyStyle,
) {
    let scale = |base: u8| -> u8 { (base as f32 * style.alpha).round() as u8 };
    let (bg, border, text) = if pressed {
        (
            Color::from_rgba8(74, 158, 255, scale(200)),
            Color::from_rgba8(100, 180, 255, scale(255)),
            Color::from_rgba8(255, 255, 255, scale(255)),
        )
    } else {
        (
            Color::from_rgba8(45, 45, 45, scale(220)),
            Color::from_rgba8(85, 85, 85, scale(255)),
            Color::from_rgba8(200, 200, 200, scale(255)),
        )
    };

    let bg_paint = Paint {
        shader: tiny_skia::Shader::SolidColor(bg),
        ..Default::default()
    };
    pixmap.fill_rect(rect, &bg_paint, Transform::identity(), None);

    let stroke = Stroke {
        width: 1.0,
        ..Default::default()
    };
    let border_paint = Paint {
        shader: tiny_skia::Shader::SolidColor(border),
        ..Default::default()
    };
    let mut pb = PathBuilder::new();
    pb.push_rect(rect);
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
    }

    draw_text(
        pixmap,
        label,
        rect.x() + rect.width() / 2.0,
        rect.y() + rect.height() / 2.0,
        style.font,
        style.font_size,
        text,
    );
}

fn draw_text(
    pixmap: &mut Pixmap,
    label: &str,
    center_x: f32,
    center_y: f32,
    font: &fontdue::Font,
    font_size: f32,
    text_color: Color,
) {
    let total_width: f32 = label
        .chars()
        .map(|c| font.rasterize(c, font_size).0.width as f32)
        .sum();
    let max_height: f32 = label
        .chars()
        .map(|c| font.rasterize(c, font_size).0.height as f32)
        .fold(0.0f32, f32::max);

    let mut text_x = center_x - total_width / 2.0;
    let text_y = center_y - max_height / 2.0;

    let text_color_r = text_color.red() * 255.0;
    let text_color_g = text_color.green() * 255.0;
    let text_color_b = text_color.blue() * 255.0;

    let pw = pixmap.width();
    let ph = pixmap.height();
    let pixels = pixmap.pixels_mut();

    for ch in label.chars() {
        let (metrics, bitmap) = font.rasterize(ch, font_size);
        if metrics.width == 0 || metrics.height == 0 {
            continue;
        }

        let char_x = text_x;
        let char_y = text_y + (max_height - metrics.height as f32) / 2.0;

        for py in 0..metrics.height {
            for px in 0..metrics.width {
                let alpha = bitmap[py * metrics.width + px];
                if alpha == 0 {
                    continue;
                }
                let draw_x = char_x as i32 + px as i32;
                let draw_y = char_y as i32 + py as i32;
                if draw_x < 0 || draw_y < 0 || draw_x as u32 >= pw || draw_y as u32 >= ph {
                    continue;
                }
                let idx = (draw_y as u32 * pw + draw_x as u32) as usize;
                let src_a = alpha as u32;
                let src_r = text_color_r as u32 * src_a / 255;
                let src_g = text_color_g as u32 * src_a / 255;
                let src_b = text_color_b as u32 * src_a / 255;
                let inv = 255 - src_a;
                let dst = pixels[idx];
                let out_r = (src_r + dst.red() as u32 * inv / 255) as u8;
                let out_g = (src_g + dst.green() as u32 * inv / 255) as u8;
                let out_b = (src_b + dst.blue() as u32 * inv / 255) as u8;
                let out_a = (src_a + dst.alpha() as u32 * inv / 255) as u8;
                pixels[idx] =
                    tiny_skia::PremultipliedColorU8::from_rgba(out_r, out_g, out_b, out_a)
                        .unwrap_or(pixels[idx]);
            }
        }

        text_x += metrics.width as f32;
    }
}

pub fn find_keyboards() -> Vec<Device> {
    let mut keyboards = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/dev/input") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("event") {
                if let Ok(dev) = Device::open(entry.path()) {
                    if dev
                        .supported_events()
                        .contains(evdev::EventType::KEY)
                    {
                        keyboards.push(dev);
                    }
                }
            }
        }
    }
    keyboards
}

pub fn input_thread(
    pressed_keys: Arc<Mutex<HashSet<u16>>>,
    stats: Arc<Mutex<TypingStats>>,
    needs_render: Arc<AtomicBool>,
) {
    let mut devices = find_keyboards();
    if devices.is_empty() {
        eprintln!("No keyboard devices found. Run with root or add user to input group.");
        return;
    }

    for dev in &mut devices {
        use nix::fcntl::{fcntl, FcntlArg, OFlag};
        use std::os::unix::io::AsRawFd;
        let fd = dev.as_raw_fd();
        let _ = fcntl(fd, FcntlArg::F_SETFL(OFlag::O_NONBLOCK));
    }

    loop {
        for dev in &mut devices {
            if let Ok(events) = dev.fetch_events() {
                for ev in events {
                    if let InputEventKind::Key(key) = ev.kind() {
                        let code = key.0;
                        let pressed = ev.value() > 0;
                        let mut keys = pressed_keys.lock().unwrap();
                        if pressed {
                            keys.insert(code);
                            stats.lock().unwrap().on_key_press(code);
                        } else {
                            keys.remove(&code);
                        }
                        drop(keys);
                        needs_render.store(true, Ordering::Relaxed);
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(8));
    }
}
