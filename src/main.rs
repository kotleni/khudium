mod cli;
mod layout;
mod layout_detect;
mod render;
mod stats;

use std::collections::HashSet;
use std::fs::File;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use clap::Parser;
use tiny_skia::Pixmap;
use wayland_client::globals::registry_queue_init;
use wayland_client::protocol::{wl_buffer, wl_callback, wl_compositor, wl_registry, wl_region, wl_shm, wl_shm_pool, wl_surface};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, ZwlrLayerSurfaceV1},
};

use cli::{parse_anchor, Args};
use layout::LayoutOptions;
use render::{calculate_dimensions, load_font, render_to_buffer, write_pixmap_to_file, RenderMetrics};
use stats::TypingStats;

struct App {
    running: bool,
    configured: bool,
    _compositor: Option<wl_compositor::WlCompositor>,
    _shm: Option<wl_shm::WlShm>,
    surface: Option<wl_surface::WlSurface>,
    _input_region: Option<wl_region::WlRegion>,
    _layer_surface: Option<ZwlrLayerSurfaceV1>,
    _pool: Option<wl_shm_pool::WlShmPool>,
    buffer: Option<wl_buffer::WlBuffer>,
    pool_file: Option<File>,
    layout: layout::KeyboardLayout,
    pressed_keys: HashSet<u16>,
    stats: TypingStats,
    width: u32,
    height: u32,
    font: fontdue::Font,
    metrics: RenderMetrics,
    needs_render: bool,
    layout_watcher: Arc<Mutex<String>>,
    last_detected_layout: String,
    fx_keys: bool,
    qh: QueueHandle<App>,
}

macro_rules! delegate_noop {
    ($ty:ty) => {
        impl Dispatch<$ty, ()> for App {
            fn event(
                _: &mut Self,
                _: &$ty,
                _: <$ty as wayland_client::Proxy>::Event,
                _: &(),
                _: &Connection,
                _: &QueueHandle<Self>,
            ) {}
        }
    };
}

impl Dispatch<wl_registry::WlRegistry, wayland_client::globals::GlobalListContents> for App {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &wayland_client::globals::GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

delegate_noop!(wl_compositor::WlCompositor);
delegate_noop!(wl_shm::WlShm);
delegate_noop!(wl_surface::WlSurface);
delegate_noop!(wl_shm_pool::WlShmPool);
delegate_noop!(wl_buffer::WlBuffer);
delegate_noop!(wl_region::WlRegion);

impl Dispatch<ZwlrLayerShellV1, ()> for App {
    fn event(
        _: &mut Self,
        _: &ZwlrLayerShellV1,
        _: zwlr_layer_shell_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure { serial, width, height } => {
                proxy.ack_configure(serial);
                if width > 0 && height > 0 {
                    state.width = width;
                    state.height = height;
                }
                state.configured = true;
                state.needs_render = true;
            }
            zwlr_layer_surface_v1::Event::Closed => {
                state.running = false;
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_callback::WlCallback, ()> for App {
    fn event(
        state: &mut Self,
        _: &wl_callback::WlCallback,
        _: wl_callback::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        state.needs_render = true;
    }
}

fn main() {
    let args = Args::parse();
    let layout_watcher = layout_detect::start_layout_watcher(Duration::from_millis(500));
    let initial_layout_name = layout_watcher.lock().unwrap().clone();
    let layout_opts = LayoutOptions {
        fx_keys: args.fx_keys,
    };
    let kb_layout = layout::Layouts::get(&initial_layout_name, layout_opts);
    let metrics = RenderMetrics::new(args.scale, args.paddings, args.alpha);
    let font = load_font();
    let (width, height) = calculate_dimensions(&kb_layout, &metrics);

    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
    let (globals, mut event_queue) =
        registry_queue_init::<App>(&conn).expect("Failed to initialize Wayland");
    let qh = event_queue.handle();

    let compositor: wl_compositor::WlCompositor = globals.bind(&qh, 4..=5, ()).unwrap();
    let shm: wl_shm::WlShm = globals.bind(&qh, 1..=1, ()).unwrap();
    let layer_shell: ZwlrLayerShellV1 = globals.bind(&qh, 1..=1, ()).unwrap();

    let surface = compositor.create_surface(&qh, ());
    let input_region = compositor.create_region(&qh, ());
    surface.set_input_region(Some(&input_region));
    let layer_surface = layer_shell.get_layer_surface(
        &surface,
        None,
        zwlr_layer_shell_v1::Layer::Overlay,
        "wkboverlay".to_string(),
        &qh,
        (),
    );

    let anchor = parse_anchor(&args.place);
    layer_surface.set_anchor(anchor);
    layer_surface.set_size(width, height);
    layer_surface
        .set_keyboard_interactivity(zwlr_layer_surface_v1::KeyboardInteractivity::None);
    layer_surface.set_exclusive_zone(-1);
    layer_surface.set_margin(0, 0, 0, 0);
    surface.commit();

    let pool_size = (width * height * 4) as usize;
    let pool_file = tempfile::tempfile().unwrap();
    pool_file.set_len(pool_size as u64).unwrap();

    use std::os::unix::io::AsFd;
    let pool = shm.create_pool(pool_file.as_fd(), pool_size as i32, &qh, ());
    let buffer = pool.create_buffer(
        0,
        width as i32,
        height as i32,
        (width * 4) as i32,
        wl_shm::Format::Argb8888,
        &qh,
        (),
    );

    let pressed_keys = Arc::new(Mutex::new(HashSet::new()));
    let stats = Arc::new(Mutex::new(TypingStats::default()));
    let needs_render = Arc::new(AtomicBool::new(true));

    let pk = pressed_keys.clone();
    let st = stats.clone();
    let nr = needs_render.clone();
    std::thread::spawn(move || render::input_thread(pk, st, nr));

    let mut app = App {
        running: true,
        configured: false,
        _compositor: Some(compositor),
        _shm: Some(shm),
        surface: Some(surface),
        _input_region: Some(input_region),
        _layer_surface: Some(layer_surface),
        _pool: Some(pool),
        buffer: Some(buffer),
        pool_file: Some(pool_file),
        layout: kb_layout,
        pressed_keys: HashSet::new(),
        stats: TypingStats::default(),
        width,
        height,
        font,
        metrics,
        needs_render: true,
        layout_watcher,
        last_detected_layout: initial_layout_name,
        fx_keys: args.fx_keys,
        qh: qh.clone(),
    };

    while app.running {
        event_queue.blocking_dispatch(&mut app).unwrap();

        let new_layout_name = app.layout_watcher.lock().unwrap().clone();
        if new_layout_name != app.last_detected_layout {
            let layout_opts = LayoutOptions {
                fx_keys: app.fx_keys,
            };
            app.layout = layout::Layouts::get(&new_layout_name, layout_opts);
            let (w, h) = calculate_dimensions(&app.layout, &app.metrics);
            app.width = w;
            app.height = h;
            app.last_detected_layout = new_layout_name;

            if let Some(layer_surface) = &app._layer_surface {
                layer_surface.set_size(app.width, app.height);
            }

            let pool_size = (app.width * app.height * 4) as usize;
            let new_pool_file = tempfile::tempfile().unwrap();
            new_pool_file.set_len(pool_size as u64).unwrap();

            use std::os::unix::io::AsFd;
            let shm = app._shm.as_ref().unwrap();
            let new_pool = shm.create_pool(new_pool_file.as_fd(), pool_size as i32, &app.qh, ());
            let new_buffer = new_pool.create_buffer(
                0,
                app.width as i32,
                app.height as i32,
                (app.width * 4) as i32,
                wl_shm::Format::Argb8888,
                &app.qh,
                (),
            );

            app._pool = Some(new_pool);
            app.buffer = Some(new_buffer);
            app.pool_file = Some(new_pool_file);
            app.needs_render = true;

            if let Some(surface) = &app.surface {
                surface.commit();
            }
        }

        let new_keys = pressed_keys.lock().unwrap().clone();
        let new_stats = stats.lock().unwrap().clone();
        let keys_changed = new_keys != app.pressed_keys;
        let stats_changed = new_stats.cpm() != app.stats.cpm() || new_stats.wpm() != app.stats.wpm()
            || new_stats.chars != app.stats.chars
            || new_stats.words != app.stats.words;

        if keys_changed {
            app.pressed_keys = new_keys;
            app.needs_render = true;
        }
        if stats_changed {
            app.stats = new_stats;
            app.needs_render = true;
        }

        if app.needs_render && app.configured {
            app.needs_render = false;

            if let (Some(surface), Some(buffer), Some(file)) = (
                &app.surface,
                &app.buffer,
                &app.pool_file,
            ) {
                let mut pixmap = Pixmap::new(app.width, app.height).unwrap();
                render_to_buffer(
                    &mut pixmap,
                    &app.layout,
                    &app.pressed_keys,
                    &app.stats,
                    &app.font,
                    &app.metrics,
                );
                write_pixmap_to_file(&pixmap, file);

                surface.attach(Some(buffer), 0, 0);
                surface.damage_buffer(0, 0, app.width as i32, app.height as i32);
                surface.commit();
            }
        }

        if app.configured {
            if let Some(surface) = &app.surface {
                let callback = surface.frame(&qh, ());
                drop(callback);
                surface.commit();
            }
        }
    }
}
