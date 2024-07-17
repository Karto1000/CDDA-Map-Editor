use bevy::prelude::Commands;
use bevy_egui::egui;
use bevy_egui::egui::{Align, Color32, Margin, Ui, WidgetText};

pub fn add_settings_frame(
    name: impl Into<WidgetText>,
    fill: Color32,
    ui: &mut Ui,
    add: impl FnOnce(&mut Ui),
) {
    egui::Frame::none()
        .fill(fill)
        .inner_margin(Margin::same(8.))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.spacing_mut().item_spacing.y = 12.;
            ui.vertical(|ui| {
                ui.label(name);
                add(ui);
            });
        });
}

pub fn input_group(
    ui: &mut Ui,
    string: &mut String,
    label: String
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.;

        let text_edit = egui::widgets::TextEdit::singleline(string)
            .vertical_align(Align::Center)
            .desired_width(0.);

        ui.add_sized([410., 32.], text_edit);

        ui.label(label);
    });
}