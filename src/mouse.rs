use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use evdev::{Device, InputEventKind, RelativeAxisType};
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};

use crate::render::RenderMetrics;

const BTN_LEFT: u16 = 272;
const BTN_RIGHT: u16 = 273;
const BTN_MIDDLE: u16 = 274;
const BTN_SIDE: u16 = 275;
const BTN_EXTRA: u16 = 276;

const SCROLL_EXPIRE: Duration = Duration::from_millis(400);

#[derive(Clone)]
pub struct MouseState {
    pub lmb: bool,
    pub rmb: bool,
    pub mmb: bool,
    pub scroll_up: bool,
    pub scroll_down: bool,
    pub side_back: bool,
    pub side_forward: bool,
    pub cursor_x: f32,
    pub cursor_y: f32,
    scroll_up_time: Option<Instant>,
    scroll_down_time: Option<Instant>,
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            lmb: false,
            rmb: false,
            mmb: false,
            scroll_up: false,
            scroll_down: false,
            side_back: false,
            side_forward: false,
            cursor_x: 0.0,
            cursor_y: 0.0,
            scroll_up_time: None,
            scroll_down_time: None,
        }
    }
}

pub fn calculate_mouse_height(metrics: &RenderMetrics) -> f32 {
    let ku = metrics.key_unit;
    let pad = metrics.padding;
    pad + ku + pad
}

pub fn find_mice() -> Vec<Device> {
    let mut mice = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/dev/input") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("event") {
                if let Ok(dev) = Device::open(entry.path()) {
                    if dev
                        .supported_events()
                        .contains(evdev::EventType::RELATIVE)
                    {
                        mice.push(dev);
                    }
                }
            }
        }
    }
    mice
}

pub fn mouse_input_thread(
    mouse_state: Arc<Mutex<MouseState>>,
    needs_render: Arc<AtomicBool>,
) {
    let mut devices = find_mice();
    if devices.is_empty() {
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
                    let mut changed = false;
                    match ev.kind() {
                        InputEventKind::RelAxis(axis) => {
                            let mut state = mouse_state.lock().unwrap();
                            if axis == RelativeAxisType::REL_X {
                                state.cursor_x += ev.value() as f32 / 8.0;
                                changed = true;
                            } else if axis == RelativeAxisType::REL_Y {
                                state.cursor_y += ev.value() as f32 / 8.0;
                                changed = true;
                            } else if axis == RelativeAxisType::REL_WHEEL {
                                if ev.value() > 0 {
                                    state.scroll_up = true;
                                    state.scroll_up_time = Some(Instant::now());
                                    changed = true;
                                } else if ev.value() < 0 {
                                    state.scroll_down = true;
                                    state.scroll_down_time = Some(Instant::now());
                                    changed = true;
                                }
                            }
                        }
                        InputEventKind::Key(key) => {
                            let mut state = mouse_state.lock().unwrap();
                            let pressed = ev.value() > 0;
                            match key.0 {
                                BTN_LEFT => {
                                    state.lmb = pressed;
                                    changed = true;
                                }
                                BTN_RIGHT => {
                                    state.rmb = pressed;
                                    changed = true;
                                }
                                BTN_MIDDLE => {
                                    state.mmb = pressed;
                                    changed = true;
                                }
                                BTN_SIDE => {
                                    state.side_forward = pressed;
                                    changed = true;
                                }
                                BTN_EXTRA => {
                                    state.side_back = pressed;
                                    changed = true;
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                    if changed {
                        needs_render.store(true, Ordering::Relaxed);
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(8));
    }
}

pub fn decay_mouse_state(state: &mut MouseState) {
    state.cursor_x *= 0.6;
    state.cursor_y *= 0.6;
    if state.cursor_x.abs() < 0.3 {
        state.cursor_x = 0.0;
    }
    if state.cursor_y.abs() < 0.3 {
        state.cursor_y = 0.0;
    }

    let now = Instant::now();
    if let Some(t) = state.scroll_up_time {
        if now.duration_since(t) > SCROLL_EXPIRE {
            state.scroll_up = false;
            state.scroll_up_time = None;
        }
    }
    if let Some(t) = state.scroll_down_time {
        if now.duration_since(t) > SCROLL_EXPIRE {
            state.scroll_down = false;
            state.scroll_down_time = None;
        }
    }
}

pub fn render_mouse(
    pixmap: &mut Pixmap,
    mouse_state: &MouseState,
    font: &fontdue::Font,
    metrics: &RenderMetrics,
    y_start: f32,
) {
    let ku = metrics.key_unit;
    let pad = metrics.padding;
    let alpha = metrics.alpha;
    let scale = |base: u8| -> u8 { (base as f32 * alpha).round() as u8 };

    let bg = Color::from_rgba8(45, 45, 45, scale(220));
    let border = Color::from_rgba8(85, 85, 85, scale(255));
    let active_bg = Color::from_rgba8(74, 158, 255, scale(200));
    let active_border = Color::from_rgba8(100, 180, 255, scale(255));
    let text_col = Color::from_rgba8(200, 200, 200, scale(255));
    let dot_col = Color::from_rgba8(120, 120, 120, scale(200));

    let btn_h = ku;
    let side_w = ku * 0.65;
    let main_w = ku * 2.8;
    let scroll_w = ku * 0.35;
    let half_main = (main_w - scroll_w) / 2.0;
    let widget_size = ku * 1.0;
    let gap = ku * 0.15;
    let total_w = side_w * 2.0 + gap + main_w + gap + widget_size;

    let base_x = pixmap.width() as f32 - pad - total_w;
    let base_y = y_start + pad;

    let lmb_x = base_x + side_w * 2.0 + gap;
    let lmb_bg = if mouse_state.lmb { active_bg } else { bg };
    let lmb_bd = if mouse_state.lmb { active_border } else { border };
    fill_rect(pixmap, lmb_x, base_y, half_main, btn_h, lmb_bg);
    stroke_rect(pixmap, lmb_x, base_y, half_main, btn_h, 1.0, lmb_bd);
    draw_text_centered(
        pixmap,
        "L",
        lmb_x + half_main / 2.0,
        base_y + btn_h / 2.0,
        font,
        metrics.font_size * 0.55,
        text_col,
    );

    let scr_x = lmb_x + half_main;
    let scr_bg = if mouse_state.mmb { active_bg } else { bg };
    let scr_bd = if mouse_state.mmb { active_border } else { border };
    fill_rect(pixmap, scr_x, base_y, scroll_w, btn_h, scr_bg);
    stroke_rect(pixmap, scr_x, base_y, scroll_w, btn_h, 1.0, scr_bd);

    if mouse_state.scroll_up {
        fill_rect(
            pixmap,
            scr_x + 1.0,
            base_y + 1.0,
            scroll_w - 2.0,
            btn_h / 2.0 - 1.0,
            active_bg,
        );
    }
    if mouse_state.scroll_down {
        fill_rect(
            pixmap,
            scr_x + 1.0,
            base_y + btn_h / 2.0,
            scroll_w - 2.0,
            btn_h / 2.0 - 1.0,
            active_bg,
        );
    }

    let rmb_x = scr_x + scroll_w;
    let rmb_bg = if mouse_state.rmb { active_bg } else { bg };
    let rmb_bd = if mouse_state.rmb { active_border } else { border };
    fill_rect(pixmap, rmb_x, base_y, half_main, btn_h, rmb_bg);
    stroke_rect(pixmap, rmb_x, base_y, half_main, btn_h, 1.0, rmb_bd);
    draw_text_centered(
        pixmap,
        "R",
        rmb_x + half_main / 2.0,
        base_y + btn_h / 2.0,
        font,
        metrics.font_size * 0.55,
        text_col,
    );

    let b_bg = if mouse_state.side_back { active_bg } else { bg };
    let b_bd = if mouse_state.side_back { active_border } else { border };
    fill_rect(pixmap, base_x, base_y, side_w, btn_h, b_bg);
    stroke_rect(pixmap, base_x, base_y, side_w, btn_h, 1.0, b_bd);
    draw_text_centered(
        pixmap,
        "B",
        base_x + side_w / 2.0,
        base_y + btn_h / 2.0,
        font,
        metrics.font_size * 0.45,
        text_col,
    );

    let f_x = base_x + side_w + gap * 0.5;
    let f_bg = if mouse_state.side_forward { active_bg } else { bg };
    let f_bd = if mouse_state.side_forward { active_border } else { border };
    fill_rect(pixmap, f_x, base_y, side_w, btn_h, f_bg);
    stroke_rect(pixmap, f_x, base_y, side_w, btn_h, 1.0, f_bd);
    draw_text_centered(
        pixmap,
        "F",
        f_x + side_w / 2.0,
        base_y + btn_h / 2.0,
        font,
        metrics.font_size * 0.45,
        text_col,
    );

    let wgt_x = rmb_x + half_main + gap;
    let wgt_y = base_y;
    fill_rect(pixmap, wgt_x, wgt_y, widget_size, widget_size, bg);
    stroke_rect(pixmap, wgt_x, wgt_y, widget_size, widget_size, 1.0, border);

    let cx = wgt_x + widget_size / 2.0;
    let cy = wgt_y + widget_size / 2.0;

    let max_r = widget_size * 0.35;
    let dot_r = ku * 0.18;
    let dx = mouse_state.cursor_x.clamp(-max_r, max_r);
    let dy = mouse_state.cursor_y.clamp(-max_r, max_r);
    let dot_active =
        mouse_state.cursor_x.abs() > 0.5 || mouse_state.cursor_y.abs() > 0.5;
    let dot_color = if dot_active { active_bg } else { dot_col };

    let dot_x = cx + dx - dot_r;
    let dot_y = cy + dy - dot_r;
    fill_rect(
        pixmap,
        dot_x,
        dot_y,
        dot_r * 2.0,
        dot_r * 2.0,
        dot_color,
    );
}

fn fill_rect(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: Color,
) {
    let rect = match Rect::from_xywh(x, y, w, h) {
        Some(r) => r,
        None => return,
    };
    let paint = Paint {
        shader: tiny_skia::Shader::SolidColor(color),
        ..Default::default()
    };
    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
}

fn stroke_rect(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    width: f32,
    color: Color,
) {
    let rect = match Rect::from_xywh(x, y, w, h) {
        Some(r) => r,
        None => return,
    };
    let paint = Paint {
        shader: tiny_skia::Shader::SolidColor(color),
        ..Default::default()
    };
    let stroke = Stroke {
        width,
        ..Default::default()
    };
    let mut pb = PathBuilder::new();
    pb.push_rect(rect);
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

fn draw_text_centered(
    pixmap: &mut Pixmap,
    label: &str,
    center_x: f32,
    center_y: f32,
    font: &fontdue::Font,
    font_size: f32,
    color: Color,
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

    let cr = color.red() * 255.0;
    let cg = color.green() * 255.0;
    let cb = color.blue() * 255.0;

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
                let a = bitmap[py * metrics.width + px];
                if a == 0 {
                    continue;
                }
                let draw_x = char_x as i32 + px as i32;
                let draw_y = char_y as i32 + py as i32;
                if draw_x < 0 || draw_y < 0 || draw_x as u32 >= pw || draw_y as u32 >= ph {
                    continue;
                }
                let idx = (draw_y as u32 * pw + draw_x as u32) as usize;
                let src_a = a as u32;
                let src_r = cr as u32 * src_a / 255;
                let src_g = cg as u32 * src_a / 255;
                let src_b = cb as u32 * src_a / 255;
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
