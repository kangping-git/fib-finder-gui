#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod calc;
use eframe::egui;
use std::{cmp::max, sync::atomic::Ordering};

#[derive(PartialEq, Eq)]
enum Status {
    Stop,
    Started,
    Stopping,
}
struct FibApp {
    threads: i32,
    target: String,
    is_started: Status,
    calc_status: Option<calc::CalcStatus>,
    last_ans: Option<i64>,
}

impl Default for FibApp {
    fn default() -> Self {
        Self {
            threads: max(1, (num_cpus::get() * 4 / 5) as i32),
            target: "31415".to_string(),
            is_started: Status::Stop,
            calc_status: None,
            last_ans: None,
        }
    }
}

impl eframe::App for FibApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(status_vec) = &self.calc_status {
            let mut finished_count = 0;
            for status in &status_vec.status {
                if status.is_finished.load(Ordering::Relaxed) {
                    finished_count += 1
                };
            }
            self.last_ans = Some(status_vec.ans.load(Ordering::Relaxed));
            self.is_started = if finished_count == 0 {
                Status::Started
            } else if finished_count == self.threads {
                self.calc_status = None;
                Status::Stop
            } else {
                Status::Stopping
            };
        }
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.heading("fibonacci finder");
        });

        egui::SidePanel::left("side").show(ctx, |ui| {
            ui.label("settings");
            ui.add_enabled(
                self.is_started == Status::Stop,
                egui::Slider::new(&mut self.threads, 1..=(num_cpus::get() as i32)).text("threads"),
            );
            ui.horizontal(|ui| {
                ui.label("target: ");
                ui.add_enabled(
                    self.is_started == Status::Stop,
                    egui::TextEdit::singleline(&mut self.target),
                );
            });
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                let start =
                    ui.add_enabled(self.is_started == Status::Stop, egui::Button::new("start"));
                if start.clicked() {
                    self.is_started = Status::Started;
                    self.calc_status = Some(calc::calc(
                        self.target.clone(),
                        self.threads as usize,
                        10000,
                    ));
                    self.last_ans = None;
                }
                let start = ui.add_enabled(
                    self.is_started == Status::Started,
                    egui::Button::new("stop"),
                );
                if start.clicked() {
                    self.is_started = Status::Stopping;
                    if let Some(status_vec) = &self.calc_status {
                        status_vec.is_stop.store(true, Ordering::Relaxed);
                    }
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(egui::Label::new(format!(
                "ans: {}",
                self.last_ans.unwrap_or(-1)
            )));
            if let Some(status) = &self.calc_status {
                for i in 0..self.threads {
                    ui.horizontal(|ui| {
                        let percent = status.status[i as usize].percent.load(Ordering::Relaxed);
                        ui.add(
                            egui::ProgressBar::new(percent as f32 / 100.0)
                                .desired_width(ui.available_width() * 0.8),
                        );
                        ui.label(format!(
                            "({}%, {})",
                            percent,
                            status.status[i as usize].place.load(Ordering::Relaxed)
                        ))
                    });
                }
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "fibonacci finder",
        native_options,
        Box::new(|_cc| Ok(Box::new(FibApp::default()))),
    )
}
