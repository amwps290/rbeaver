use egui::{Color32, Context, Margin, Rounding, Shadow, Stroke, Visuals};

/// Setup light theme for the application (DBeaver-like styling)
pub fn setup_light_theme(ctx: &Context) {
    let mut visuals = Visuals::light();

    // Background colors
    visuals.window_fill = Color32::from_rgb(248, 248, 248);
    visuals.panel_fill = Color32::from_rgb(245, 245, 245);
    visuals.faint_bg_color = Color32::from_rgb(240, 240, 240);

    // Text colors (these are read-only in egui 0.28, set via override_text_color if needed)
    // visuals.text_color = Color32::from_rgb(33, 37, 41);
    // visuals.strong_text_color = Color32::from_rgb(0, 0, 0);
    // visuals.weak_text_color = Color32::from_rgb(108, 117, 125);

    // Widget colors
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(255, 255, 255);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(206, 212, 218));
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(33, 37, 41));

    visuals.widgets.inactive.bg_fill = Color32::from_rgb(248, 249, 250);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(206, 212, 218));
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(73, 80, 87));

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(233, 236, 239);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(173, 181, 189));
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::from_rgb(33, 37, 41));

    visuals.widgets.active.bg_fill = Color32::from_rgb(0, 123, 255);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0, 86, 179));
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::from_rgb(255, 255, 255));

    visuals.widgets.open.bg_fill = Color32::from_rgb(220, 248, 198);
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, Color32::from_rgb(40, 167, 69));
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, Color32::from_rgb(33, 37, 41));

    // Selection colors
    visuals.selection.bg_fill = Color32::from_rgb(0, 123, 255).linear_multiply(0.2);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(0, 123, 255));

    // Hyperlink colors
    visuals.hyperlink_color = Color32::from_rgb(0, 123, 255);

    // Error colors
    visuals.error_fg_color = Color32::from_rgb(220, 53, 69);
    visuals.warn_fg_color = Color32::from_rgb(255, 193, 7);

    // Window styling
    visuals.window_rounding = Rounding::same(6.0);
    visuals.window_shadow = Shadow {
        offset: egui::Vec2::new(2.0, 4.0),
        blur: 8.0,
        spread: 0.0,
        color: Color32::from_black_alpha(25),
    };
    visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(206, 212, 218));

    // Menu styling
    visuals.menu_rounding = Rounding::same(4.0);

    // Popup styling
    visuals.popup_shadow = Shadow {
        offset: egui::Vec2::new(1.0, 2.0),
        blur: 6.0,
        spread: 0.0,
        color: Color32::from_black_alpha(20),
    };

    // Resize handle
    visuals.resize_corner_size = 12.0;

    // Indent and spacing
    visuals.indent_has_left_vline = true;

    ctx.set_visuals(visuals);

    // Set custom style
    let mut style = (*ctx.style()).clone();

    // Spacing
    style.spacing.item_spacing = egui::Vec2::new(8.0, 6.0);
    style.spacing.button_padding = egui::Vec2::new(12.0, 6.0);
    style.spacing.menu_margin = Margin::same(8.0);
    style.spacing.indent = 20.0;
    style.spacing.window_margin = Margin::same(8.0);
    // Note: scroll_bar_width is not available in egui 0.28

    // Text styles
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(18.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(13.0, egui::FontFamily::Monospace),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(12.0, egui::FontFamily::Proportional),
    );

    ctx.set_style(style);
}

/// Get colors for SQL syntax highlighting
pub fn get_sql_syntax_colors() -> SqlSyntaxColors {
    SqlSyntaxColors {
        keyword: Color32::from_rgb(0, 0, 255),     // Blue for keywords
        string: Color32::from_rgb(163, 21, 21),    // Dark red for strings
        comment: Color32::from_rgb(0, 128, 0),     // Green for comments
        number: Color32::from_rgb(255, 140, 0),    // Orange for numbers
        operator: Color32::from_rgb(128, 0, 128),  // Purple for operators
        identifier: Color32::from_rgb(33, 37, 41), // Default text color
        function: Color32::from_rgb(128, 0, 255),  // Purple for functions
    }
}

/// Colors for SQL syntax highlighting
pub struct SqlSyntaxColors {
    pub keyword: Color32,
    pub string: Color32,
    pub comment: Color32,
    pub number: Color32,
    pub operator: Color32,
    pub identifier: Color32,
    pub function: Color32,
}

/// Get table colors for result display
pub struct TableColors {
    pub header_bg: Color32,
    pub header_text: Color32,
    pub row_bg_even: Color32,
    pub row_bg_odd: Color32,
    pub row_text: Color32,
    pub selected_bg: Color32,
    pub selected_text: Color32,
    pub border: Color32,
}

pub fn get_table_colors() -> TableColors {
    TableColors {
        header_bg: Color32::from_rgb(233, 236, 239),
        header_text: Color32::from_rgb(33, 37, 41),
        row_bg_even: Color32::from_rgb(255, 255, 255),
        row_bg_odd: Color32::from_rgb(248, 249, 250),
        row_text: Color32::from_rgb(33, 37, 41),
        selected_bg: Color32::from_rgb(0, 123, 255).linear_multiply(0.2),
        selected_text: Color32::from_rgb(33, 37, 41),
        border: Color32::from_rgb(206, 212, 218),
    }
}
