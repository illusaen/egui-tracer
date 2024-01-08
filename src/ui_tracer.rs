use super::state::{CollectedEvent, LogsState, TracerLevel};
use super::EventCollector;
use eframe::egui;
pub use eframe::egui::Widget;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use time;

static PADDING_LEFT: f32 = 4.0;
pub(super) static SEPARATOR_SPACING: f32 = 6.0;
static CELL_SPACING: f32 = PADDING_LEFT * 2.0 + 10.0;

pub struct LogUi {
    collector: EventCollector,
}

impl LogUi {
    pub fn new(collector: EventCollector) -> Self {
        Self { collector }
    }
}

impl Widget for LogUi {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let events = self.collector.events();
        let state = ui.memory_mut(|mem| {
            mem.data
                .get_persisted_mut_or_insert_with(ui.id(), || {
                    Arc::new(Mutex::new(LogsState::default()))
                })
                .clone()
        });
        let filtered_events = events
            .iter()
            .filter(|&event| {
                matches!(
                    state.lock().unwrap().level_filter.get(&event.level),
                    Some(true)
                )
            })
            .collect::<Vec<_>>();

        let row_height = SEPARATOR_SPACING
            + ui.style()
                .text_styles
                .get(&egui::TextStyle::Small)
                .unwrap()
                .size;

        table(
            ui,
            row_height,
            filtered_events.iter(),
            || {
                self.collector.clear();
            },
            |ui| {
                table_header(ui, Some(100.0), |ui| {
                    ui.label("Time");
                });
                table_header(ui, Some(80.0), |ui| {
                    level_menu_button(ui, &mut state.lock().unwrap().level_filter);
                });
                table_header(ui, Some(120.0), |ui| {
                    ui.label("Target");
                });
                table_header(ui, None, |ui| {
                    ui.label("Message");
                });
            },
            table_row,
        )
    }
}

fn table_row(ui: &mut egui::Ui, event: &&CollectedEvent) {
    let time_format = time::macros::format_description!("[hour repr:12]:[minute]:[second][period]");
    let time_format_long = time::macros::format_description!(
        "[day]-[month]-[year repr:last_two] [hour repr:12]:[minute]:[second][period]"
    );
    table_cell(ui, 100.0, |ui| {
        ui.colored_label(
            ui.visuals().weak_text_color(),
            event
                .time
                .format(time_format)
                .expect("Time formatted correctly with short format from tracer"),
        )
        .on_hover_text(
            event
                .time
                .format(time_format_long)
                .expect("Time formatted correctly with long format from tracer"),
        );
    });
    table_cell(ui, 80.0, |ui| {
        ui.colored_label(event.level.to_color32(ui), event.level.to_string());
    });
    table_cell(ui, 120.0, |ui| {
        ui.colored_label(ui.visuals().text_color(), event.target.clone())
            .on_hover_text(&event.target);
    });
    table_cell(ui, 120.0, |ui| {
        let message = event.fields.get("message").unwrap();
        ui.add(egui::Label::new(message.clone().truncate(108)).wrap(false))
            .on_hover_text(message);
    });
}

fn level_menu_button(ui: &mut egui::Ui, state: &mut BTreeMap<TracerLevel, bool>) {
    ui.menu_button("Level", |ui| {
        state.iter_mut().for_each(|(level, enabled)| {
            if ui.selectable_label(*enabled, level.to_string()).clicked() {
                *enabled = !*enabled;
            }
        });
    });
}

fn table_header(
    ui: &mut egui::Ui,
    min_width: Option<f32>,
    content: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    ui.horizontal(|ui| {
        if let Some(min_width) = min_width {
            ui.set_width(min_width);
        }
        let available_space = ui.available_size_before_wrap();
        let size = egui::vec2(PADDING_LEFT, available_space.y);
        let (rect, response) = ui.allocate_at_least(size, egui::Sense::hover());
        if ui.is_rect_visible(response.rect) {
            let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
            let painter = ui.painter();
            painter.vline(rect.left(), rect.top()..=rect.bottom(), stroke);
        }

        content(ui);
    })
    .response
}

fn table_cell(
    ui: &mut egui::Ui,
    min_width: f32,
    content: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    ui.horizontal(|ui| {
        ui.set_width(min_width);
        ui.add_space(CELL_SPACING);
        content(ui);
    })
    .response
}

fn table<T>(
    ui: &mut egui::Ui,
    row_height: f32,
    values: std::slice::Iter<T>,
    on_clear: impl FnOnce(),
    header: impl FnOnce(&mut egui::Ui),
    row: impl Fn(&mut egui::Ui, &T),
) -> egui::Response {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.horizontal(|ui| {
                header(ui);
            });
            ui.add_space(ui.available_width() - 60.0);
            ui.separator();
            if ui.button("Clear").on_hover_text("Clear Events").clicked() {
                on_clear();
            }
        });

        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show_rows(ui, row_height + SEPARATOR_SPACING, values.len(), |ui, _| {
                values.for_each(|value| {
                    ui.horizontal(|ui| {
                        row(ui, value);
                    });
                    ui.separator();
                });
            })
    })
    .response
}

trait Ellipse {
    fn truncate(&self, len: usize) -> String;
}

impl Ellipse for String {
    fn truncate(&self, len: usize) -> String {
        if self.len() <= len {
            return self.clone();
        }

        self.chars().take(len).collect::<String>() + "..."
    }
}
