#[derive(Clone, Copy, Debug, Default)]
struct EguiThemeColors {
    bg_0: egui::Color32,
    bg_1: egui::Color32,
    bg_2: egui::Color32,
    panel_0: egui::Color32,
    panel_2: egui::Color32,
    border_0: egui::Color32,
    text_0: egui::Color32,
    text_2: egui::Color32,
    accent_cyan: egui::Color32,
}

fn resolve_egui_theme_colors(
    themes: Option<&YamlValue>,
    theme_key: &str,
) -> Option<EguiThemeColors> {
    let themes = themes?;
    let theme = themes.get(theme_key)?;

    let colors_root = theme
        .get("tokens")
        .and_then(|tokens| tokens.get("colors"))
        .or_else(|| theme.get("colors"))?;

    let get_color = |key: &str| {
        colors_root
            .get(key)
            .and_then(|value| value.as_str())
            .and_then(parse_hex_color)
    };

    Some(EguiThemeColors {
        bg_0: get_color("bg_0")?,
        bg_1: get_color("bg_1")?,
        bg_2: get_color("bg_2")?,
        panel_0: get_color("panel_0")?,
        panel_2: get_color("panel_2")?,
        border_0: get_color("border_0")?,
        text_0: get_color("text_0")?,
        text_2: get_color("text_2")?,
        accent_cyan: get_color("accent_cyan")?,
    })
}

fn parse_hex_color(value: &str) -> Option<egui::Color32> {
    let value = value.trim();
    let hex = value.strip_prefix('#').unwrap_or(value);

    fn byte_from_hex(pair: &str) -> Option<u8> {
        u8::from_str_radix(pair, 16).ok()
    }

    match hex.len() {
        6 => {
            let red = byte_from_hex(&hex[0..2])?;
            let green = byte_from_hex(&hex[2..4])?;
            let blue = byte_from_hex(&hex[4..6])?;
            Some(egui::Color32::from_rgb(red, green, blue))
        }
        8 => {
            let red = byte_from_hex(&hex[0..2])?;
            let green = byte_from_hex(&hex[2..4])?;
            let blue = byte_from_hex(&hex[4..6])?;
            let alpha = byte_from_hex(&hex[6..8])?;
            Some(egui::Color32::from_rgba_unmultiplied(
                red, green, blue, alpha,
            ))
        }
        _ => None,
    }
}