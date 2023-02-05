mod gd;

use eframe;
use eframe::egui;
use std::boxed::Box;
use std::collections::VecDeque;


struct PipeDash {
    msg_queue: VecDeque<Message>,
    selected_level: Option<usize>,
    selected_color: Option<Color>,
    level_list: Vec<gd::OuterLevel>,
}

#[derive(Debug, PartialEq, Eq)]
enum Color {
    Orange,
    Yellow,
    Green,
}

#[derive(Debug)]
enum Message {
    ColorSelected(Color),
    LevelSelected(usize),
}


impl PipeDash {
    fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            selected_level: None,
            selected_color: None,
            msg_queue: VecDeque::new(),
            level_list: gd::OuterLevel::load_all(),
        }
    }

}

impl eframe::App for PipeDash {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        use Message::*;
        use Color::*;
        ctx.set_pixels_per_point(2f32);

        egui::SidePanel::left("level_picker").default_width(100f32).show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                for (idx, level) in self.level_list.iter().enumerate() {
                    if ui.selectable_label(self.selected_level == Some(idx), level.display_name()).clicked() {
                        self.msg_queue.push_back(LevelSelected(idx));
                    }
                }
            })
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Song name");
                        ui.label("Song id");
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        ui.vertical(|ui| {
                            if ui.selectable_label(self.selected_color == Some(Orange), "orange").clicked() {
                                self.msg_queue.push_back(ColorSelected(Orange));
                            }
                            if ui.selectable_label(self.selected_color == Some(Yellow), "yellow").clicked() {
                                self.msg_queue.push_back(ColorSelected(Yellow));
                            }
                            if ui.selectable_label(self.selected_color == Some(Green), "green").clicked() {
                                self.msg_queue.push_back(ColorSelected(Green));
                            }
                        });
                    })
                });
                ui.label("custom editor panel goes here")
            });
        });

        for message in self.msg_queue.drain(..) {
            println!("{:?}", message);
            match message {
                Message::ColorSelected(color) => {
                    self.selected_color = Some(color);
                },
                Message::LevelSelected(idx) => {
                    self.selected_level = Some(idx);
                },
            }
        }
    }
}

fn main() {
    let app: PipeDash;
    let opts = eframe::NativeOptions::default();
    eframe::run_native("PipeDash", opts, Box::new(|cc| Box::new(PipeDash::new(cc))));
}

