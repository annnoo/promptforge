use eframe::egui::{self, Color32, CornerRadius, Stroke, Visuals};

/// Configures dark theme styling for the PromptForge GUI application.
pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    // Dark sleek developer-tool palette
    visuals.window_fill = Color32::from_rgb(20, 22, 28);
    visuals.panel_fill = Color32::from_rgb(24, 27, 34);
    visuals.extreme_bg_color = Color32::from_rgb(15, 17, 21);

    // Widgets inactive
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(33, 37, 46);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(50, 56, 68));
    visuals.widgets.inactive.corner_radius = CornerRadius::same(4);

    // Widgets hovered
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(45, 51, 64);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(90, 105, 130));
    visuals.widgets.hovered.corner_radius = CornerRadius::same(4);

    // Widgets active
    visuals.widgets.active.bg_fill = Color32::from_rgb(55, 65, 85);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(100, 180, 240));
    visuals.widgets.active.corner_radius = CornerRadius::same(4);

    // Text colors
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(220, 225, 235));
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(200, 205, 215));

    ctx.set_visuals(visuals);
}
