use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn start_layout_watcher(interval: Duration) -> Arc<Mutex<String>> {
    let current = Arc::new(Mutex::new(detect_layout()));
    let shared = Arc::clone(&current);
    thread::spawn(move || {
        let mut prev = shared.lock().unwrap().clone();
        loop {
            thread::sleep(interval);
            let detected = detect_layout();
            if detected != prev {
                eprintln!("Layout changed: {prev} -> {detected}");
                *shared.lock().unwrap() = detected.clone();
                prev = detected;
            }
        }
    });
    current
}

pub fn detect_layout() -> String {
    if let Some(layout) = detect_sway() {
        return layout;
    }
    if let Some(layout) = detect_hyprland() {
        return layout;
    }
    if let Some(layout) = detect_xkb_env() {
        return layout;
    }
    "qwerty".to_string()
}

fn detect_sway() -> Option<String> {
    std::env::var("SWAYSOCK").ok()?;
    let output = Command::new("swaymsg")
        .args(["-t", "get_inputs"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    find_layout_in_json(&text)
}

fn detect_hyprland() -> Option<String> {
    std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    let output = Command::new("hyprctl").args(["devices", "-j"]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for name in extract_json_string_values(&text, "active_keymap") {
        if let Some(layout) = map_xkb_name(&name) {
            return Some(layout.to_string());
        }
    }
    None
}

fn detect_xkb_env() -> Option<String> {
    for var in ["XKB_DEFAULT_LAYOUT", "XKB_LAYOUT"] {
        if let Ok(val) = std::env::var(var) {
            let primary = val.split(',').next()?.trim();
            if let Some(layout) = map_xkb_name(primary) {
                return Some(layout.to_string());
            }
        }
    }
    None
}

fn find_layout_in_json(text: &str) -> Option<String> {
    let mut in_keyboard = false;
    let mut depth = 0i32;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.contains("\"type\"") && trimmed.contains("keyboard") {
            in_keyboard = true;
            depth = 0;
        }
        if in_keyboard {
            depth += trimmed.matches('{').count() as i32;
            depth -= trimmed.matches('}').count() as i32;

            if let Some(name) = extract_json_string(trimmed, "xkb_active_layout_name") {
                if let Some(layout) = map_xkb_name(&name) {
                    return Some(layout.to_string());
                }
            }

            if depth <= 0 && trimmed.contains('}') {
                in_keyboard = false;
            }
        }
    }

    for name in extract_json_string_values(text, "xkb_active_layout_name") {
        if let Some(layout) = map_xkb_name(&name) {
            return Some(layout.to_string());
        }
    }
    None
}

fn extract_json_string(line: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{key}\":");
    let start = line.find(&pattern)? + pattern.len();
    let rest = line[start..].trim_start();
    if !rest.starts_with('"') {
        return None;
    }
    let rest = &rest[1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_json_string_values(text: &str, key: &str) -> Vec<String> {
    let pattern = format!("\"{key}\":");
    let mut values = Vec::new();
    let mut search_from = 0;
    while let Some(found) = text[search_from..].find(&pattern) {
        let pos = search_from + found + pattern.len();
        let rest = text[pos..].trim_start();
        if rest.starts_with('"') {
            let rest = &rest[1..];
            if let Some(end) = rest.find('"') {
                values.push(rest[..end].to_string());
            }
        }
        search_from = pos + 1;
    }
    values
}

fn map_xkb_name(name: &str) -> Option<&'static str> {
    let n = name.to_lowercase();
    if n.contains("dvorak") {
        Some("dvorak")
    } else if n.contains("colemak") {
        Some("colemak")
    } else if n.contains("canary") {
        Some("canary")
    } else if n.contains("us")
        || n.contains("english")
        || n.contains("en")
        || n.contains("qwerty")
        || n.contains("intl")
        || n.contains("basic")
    {
        Some("qwerty")
    } else {
        None
    }
}
