use clap::Parser;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1;

#[derive(Parser)]
#[command(name = "wkboverlay", about = "Keyboard overlay for wlroots Wayland compositors")]
pub struct Args {
    /// Gap between keys in pixels
    #[arg(long, default_value = "4")]
    pub paddings: u32,

    /// Screen anchor position (e.g. bottom-right, top-left)
    #[arg(long, default_value = "bottom-right")]
    pub place: String,

    /// Scale factor for overlay size
    #[arg(long, default_value = "1.0")]
    pub scale: f32,

    /// Global alpha/opacity (0.0 - 1.0)
    #[arg(long, default_value = "0.5")]
    pub alpha: f32,

    /// Show the function key row
    #[arg(long, default_value = "false")]
    pub fx_keys: bool,

    /// Split keyboard layout (lily58, corne, cheapino)
    #[arg(long)]
    pub split: Option<String>,
}

pub fn parse_anchor(place: &str) -> zwlr_layer_surface_v1::Anchor {
    use zwlr_layer_surface_v1::Anchor;
    let mut anchor = Anchor::empty();
    let place = place.to_lowercase();
    if place.contains("bottom") {
        anchor |= Anchor::Bottom;
    }
    if place.contains("top") {
        anchor |= Anchor::Top;
    }
    if place.contains("left") {
        anchor |= Anchor::Left;
    }
    if place.contains("right") {
        anchor |= Anchor::Right;
    }
    if anchor.is_empty() {
        anchor = Anchor::Bottom | Anchor::Right;
    }
    anchor
}
