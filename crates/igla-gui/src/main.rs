//! Окно «ИГЛА».
//!
//! Скелет: панель параметров уже ходит через тот же [`ProjectFile`], что и
//! CLI, поле графика — на месте, контура пока нет. Смысл в том, чтобы связка
//! «параметры → ядро → отрисовка» существовала до появления физики и потом
//! не переписывалась.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use igla_core::project::{ChamberFile, ContourFile, DesignFile, GeometryFile, OperatingFile};
use igla_core::{ProjectFile, Propellant, project};

fn app_icon() -> egui::IconData {
    if cfg!(target_os = "macos") {
        return egui::IconData::default();
    }
    eframe::icon_data::from_png_bytes(include_bytes!("../../../packaging/icon-512.png"))
        .expect("иконка приложения не разобралась")
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_icon(app_icon()),
        ..Default::default()
    };
    eframe::run_native(
        "ИГЛА — сопло внешнего расширения",
        options,
        Box::new(|_cc| Ok(Box::<App>::default())),
    )
}

struct App {
    file: ProjectFile,
}

impl Default for App {
    fn default() -> Self {
        Self {
            file: ProjectFile {
                schema: project::SCHEMA,
                name: Some("Кольцевой LH₂".to_owned()),
                geometry: GeometryFile::Annular {
                    throat_radius_mm: 80.0,
                },
                propellant: project::PropellantFile::LoxLh2,
                chamber: ChamberFile { pressure_bar: 80.0 },
                design: DesignFile::AreaRatio { epsilon: 40.0 },
                contour: ContourFile {
                    truncation_pct: 22.0,
                    points: 160,
                },
                operating: OperatingFile { altitude_km: 0.0 },
            },
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("params")
            .default_size(300.0)
            .show(ui, |ui| self.params_panel(ui));

        egui::Panel::bottom("status").show(ui, |ui| {
            // Проверка идёт через то же ядро, что и `igla check`: расхождению
            // между окном и командной строкой взяться неоткуда.
            match self.file.clone().into_params() {
                Ok(_) => ui.label("Параметры допустимы"),
                Err(e) => ui.colored_label(egui::Color32::from_rgb(220, 80, 80), e.to_string()),
            };
        });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Меридиональное сечение");
            egui_plot::Plot::new("contour")
                .data_aspect(1.0)
                .show(ui, |_plot_ui| {
                    // Контур появится здесь после порта igla_core::contour.
                });
        });
    }
}

impl App {
    fn params_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Параметры");
        ui.separator();

        ui.label("Топливная пара");
        let current = Propellant::from(self.file.propellant);
        egui::ComboBox::from_id_salt("propellant")
            .selected_text(current.label())
            .show_ui(ui, |ui| {
                for candidate in Propellant::TABLE {
                    if ui
                        .selectable_label(current == candidate, candidate.label())
                        .clicked()
                    {
                        self.file.propellant = candidate.into();
                    }
                }
            });

        ui.add_space(8.0);
        ui.label("Давление в камере, бар");
        ui.add(egui::Slider::new(
            &mut self.file.chamber.pressure_bar,
            1.0..=300.0,
        ));

        ui.add_space(8.0);
        ui.label("Степень расширения ε");
        if let DesignFile::AreaRatio { epsilon } = &mut self.file.design {
            ui.add(egui::Slider::new(epsilon, 1.5..=200.0).logarithmic(true));
        }

        ui.add_space(8.0);
        ui.label("Усечение иглы, %");
        ui.add(egui::Slider::new(
            &mut self.file.contour.truncation_pct,
            5.0..=100.0,
        ));

        ui.add_space(8.0);
        ui.label("Радиус критики, мм");
        if let GeometryFile::Annular { throat_radius_mm } = &mut self.file.geometry {
            ui.add(egui::Slider::new(throat_radius_mm, 5.0..=500.0));
        }
    }
}
