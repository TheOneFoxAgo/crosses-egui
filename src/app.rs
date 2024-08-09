use crosses_core::{
    board_manager::{Cell, CellKind},
    player_manager::PlayerManager,
};
use egui::Color32;

use crate::sample_core::{sample_cell::SampleCell, CrossesCore};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    game: CrossesCore,
    current_error: String,
    export_field: String,
    import_field: String,
    max_x: usize,
    max_y: usize,
    focused: Option<(usize, usize)>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            game: CrossesCore {
                board_manager: Default::default(),
                player_manager: PlayerManager::new(4, 2, [None; 2]),
                log: Default::default(),
            },
            current_error: Default::default(),
            export_field: Default::default(),
            import_field: Default::default(),
            max_x: 10,
            max_y: 10,
            focused: Default::default(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Крестики (Версия для ценителей)");

            ui.horizontal(|ui| {
                ui.label("Загрузить игру: ");
                ui.text_edit_singleline(&mut self.import_field);
            });
            ui.label("Выгрузить игру: WIP");
            ui.horizontal(|ui| {
                self.game_board(ui);
                self.info(ui);
            });

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

impl TemplateApp {
    fn game_board(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            for y in 0..self.game.board_manager.max_y {
                ui.horizontal(|ui| {
                    for x in 0..self.game.board_manager.max_x {
                        let cell = self.game.board_manager.board[x][y];
                        let mut button = egui::Button::new(if cell.kind() == CellKind::Cross {
                            egui::RichText::new("x").monospace().color(get_color(cell))
                        } else {
                            egui::RichText::new(" ").monospace()
                        });
                        if cell.kind() == CellKind::Filled {
                            button = button.fill(get_color(cell));
                        } else {
                            if cell.is_checked() {
                                button = button.fill(Color32::GOLD)
                            } else if cell.is_active(self.game.player_manager.current_player() == 1)
                            {
                                button = button.fill(Color32::GRAY)
                            }
                        }
                        let response = ui.add(button);
                        if response.clicked() {
                            self.game.board_manager.clear_checked();
                            if let Err(e) = self.game.make_move(x, y) {
                                self.current_error = e.to_string();
                            }
                        }
                        if response.secondary_clicked() {
                            self.focused = Some((x, y));
                        }
                    }
                });
            }
        });
    }
    fn info(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            egui::Grid::new("Info").show(ui, |ui| {
                ui.label("Ошибка:");
                ui.label(self.current_error.to_string());
                ui.end_row();
                ui.label("Ходы и крестики синих:");
                ui.monospace(format!(
                    "({}, {})",
                    self.game.board_manager.moves_counter[0],
                    self.game.board_manager.crosses_counter[0]
                ));
                ui.end_row();
                ui.label("Ходы и крестики красных:");
                ui.monospace(format!(
                    "({}, {})",
                    self.game.board_manager.moves_counter[1],
                    self.game.board_manager.crosses_counter[1]
                ));
                ui.end_row();
                let focused_cell = self
                    .focused
                    .map(|(x, y)| self.game.board_manager.board[x][y]);
                ui.label("Координаты:");
                if let Some((x, y)) = self.focused {
                    ui.monospace(format!("x: {x} y: {y}"));
                }
                ui.end_row();
                ui.label("Тип:");
                if let Some(cell) = focused_cell {
                    ui.monospace(match cell.kind() {
                        CellKind::Empty => "Пустая",
                        CellKind::Cross => "Крестик",
                        CellKind::Filled => "Закрашена",
                        CellKind::Border => "Граница",
                    });
                }
                ui.end_row();
                ui.label("Игрок:");
                if let Some(cell) = focused_cell {
                    ui.monospace(match cell.kind() {
                        CellKind::Cross | CellKind::Filled => {
                            if cell.player() {
                                "Красный"
                            } else {
                                "Синий"
                            }
                        }
                        _ => "Никакого",
                    });
                }
                ui.end_row();
                ui.label("Красные активации:");
                if let Some(cell) = focused_cell {
                    ui.monospace(format!("{}", cell.activity(true)));
                }
                ui.end_row();
                ui.label("Синие активации:");
                if let Some(cell) = focused_cell {
                    ui.monospace(format!("{}", cell.activity(false)));
                }
                ui.end_row();
                ui.label("Перегрев:");
                if let Some(cell) = focused_cell {
                    ui.monospace(if cell.is_overheated() {
                        "Перегрета"
                    } else {
                        "Нету"
                    });
                }
                ui.end_row();
                ui.label("Важность:");
                if let Some(cell) = focused_cell {
                    ui.monospace(match cell.kind() {
                        CellKind::Cross | CellKind::Filled => {
                            if cell.is_important() {
                                "Важная"
                            } else {
                                "Не важная"
                            }
                        }
                        _ => "Никакая",
                    });
                }
                ui.end_row();
                ui.label("Живость:");
                if let Some(cell) = focused_cell {
                    ui.monospace(if cell.kind() == CellKind::Filled {
                        if cell.is_alive() {
                            "Живая"
                        } else {
                            "Мёртвая"
                        }
                    } else {
                        "Никакая"
                    });
                }
                ui.end_row();
            });
        });
    }
}
fn get_color(cell: SampleCell) -> Color32 {
    if cell.player() {
        egui::Color32::RED
    } else {
        egui::Color32::BLUE
    }
}
fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
