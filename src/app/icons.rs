use cosmic::widget;

const DISCONNECT_SVG: &[u8] =
    include_bytes!("../../resources/icons/bluetooth-disconnected-symbolic.svg");
const RENAME_SVG: &[u8] = include_bytes!("../../resources/icons/edit-symbolic.svg");
const FULLY_RECONNECT_SVG: &[u8] =
    include_bytes!("../../resources/icons/view-refresh-symbolic.svg");
const REMOVE_SVG: &[u8] = include_bytes!("../../resources/icons/user-trash-symbolic.svg");

fn svg_icon(bytes: &'static [u8]) -> widget::icon::Handle {
    let mut svg = String::from_utf8_lossy(bytes).into_owned();
    for color in ["#2e3436", "#2e3434", "#232323", "#2e3436", "#2e3434"] {
        svg = svg.replace(color, "#dcdcdc");
    }
    svg = svg.replace("fill-opacity=\"0.34902\"", "fill-opacity=\"1\"");
    svg = svg.replace("fill-opacity=\"0.95\"", "fill-opacity=\"1\"");
    widget::icon::from_svg_bytes(svg.into_bytes())
}

pub fn disconnect_icon() -> widget::icon::Handle {
    svg_icon(DISCONNECT_SVG)
}

pub fn rename_icon() -> widget::icon::Handle {
    svg_icon(RENAME_SVG)
}

pub fn reconnect_icon() -> widget::icon::Handle {
    svg_icon(FULLY_RECONNECT_SVG)
}

pub fn remove_icon() -> widget::icon::Handle {
    svg_icon(REMOVE_SVG)
}
