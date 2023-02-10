mod gd;
mod music;

use eframe;
use eframe::egui;
use std::boxed::Box;
use std::collections::VecDeque;
use std::marker::PhantomData;

struct PipeDash {
    msg_queue: VecDeque<Message>,
    selected_level: Option<usize>,
    level_list: Vec<gd::OuterLevel>,
    editor: Editor,
}

#[derive(Debug, PartialEq, Eq)]
enum Color {
    Orange,
    Yellow,
    Green,
}

#[derive(Debug)]
enum Message {
    LevelSelected(usize),
}

struct Editor {
    scroll_pos: f32,
    pts_per_second: f32,
    beats_per_bar: f32,
    subdivisions: f32,
    beat_rate: music::BeatRate,
    time_signatures: music::TimeSignature,
    green_lines: music::Lines,
    orange_lines: music::Lines,
    yellow_lines: music::Lines,
}

struct Orange;
struct Yellow;
struct Green;

struct BeatRateWidget<'a>(&'a mut Editor);
struct TimeSignatureWidget<'a>(&'a mut Editor);
struct LinesWidget<'a, C: WithColor>(&'a mut Editor, PhantomData<C>);

trait WithColor {}

impl WithColor for Orange {}
impl WithColor for Yellow {}
impl WithColor for Green {}

impl Editor {
    pub fn beat_rate_widget(&mut self) -> BeatRateWidget {
        BeatRateWidget(self)
    }

    pub fn time_signature_widget(&mut self) -> TimeSignatureWidget {
        TimeSignatureWidget(self)
    }

    pub fn lines_widget<C: WithColor>(&mut self) -> LinesWidget<C> {
        LinesWidget(self, Default::default())
    }
}

impl<'a> egui::Widget for BeatRateWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // 1. choose size
        let max_rect = ui.max_rect();
        let preferred_size = egui::Vec2::new(max_rect.size().x, 60.0);
        // 2. allocate space
        let (rect, res) = ui.allocate_exact_size(preferred_size, egui::Sense::click_and_drag());
        // 3. handle interactions
        // 4. draw widget
        if ui.is_rect_visible(rect) {
            ui.painter()
                .rect_filled(rect, 0f32, eframe::epaint::Color32::from_gray(0));
        }
        res
    }
}

impl<'a> egui::Widget for TimeSignatureWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // 1. choose size
        let max_rect = ui.max_rect();
        let preferred_size = egui::Vec2::new(max_rect.size().x, 60.0);
        // 2. allocate space
        let (rect, res) = ui.allocate_exact_size(preferred_size, egui::Sense::click_and_drag());
        // 3. handle interactions
        // 4. draw widget
        if ui.is_rect_visible(rect) {
            ui.painter()
                .rect_filled(rect, 0f32, eframe::epaint::Color32::from_gray(0));
        }
        res
    }
}

impl PipeDash {
    fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            selected_level: None,
            msg_queue: VecDeque::new(),
            level_list: gd::OuterLevel::load_all(),
            editor: Editor {
                scroll_pos: 0f32,
                pts_per_second: 5f32,
                beats_per_bar: 4.0,
                subdivisions: 4.0,
                beat_rate: music::StaticBeatRate::from_bpm(120f32).into(),
                time_signatures: music::StaticTimeSignature::new(4, 4).into(),
                green_lines: music::Lines::new(),
                orange_lines: music::Lines::new(),
                yellow_lines: music::Lines::new(),
            },
        }
    }

    fn side_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::left("level_picker")
            .default_width(100f32)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                        for (idx, level) in self.level_list.iter().enumerate() {
                            if ui
                                .selectable_label(
                                    self.selected_level == Some(idx),
                                    level.display_name(),
                                )
                                .clicked()
                            {
                                self.msg_queue.push_back(Message::LevelSelected(idx));
                            }
                        }
                    })
                });
            });
    }

    fn center_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.vertical(|ui| {
                    ui.label("Song name");
                    ui.label("Song id");
                });
                ui.add(self.editor.beat_rate_widget());
            });
        });
    }

    fn handle_messages(&mut self) {
        for message in self.msg_queue.drain(..) {
            println!("{:?}", message);
            match message {
                Message::LevelSelected(idx) => {
                    self.selected_level = Some(idx);
                }
            }
        }
    }
}

impl eframe::App for PipeDash {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(2f32);

        self.side_panel(ctx, frame);
        self.center_panel(ctx, frame);

        self.handle_messages();
    }
}

fn main() {
    let app: PipeDash;
    let opts = eframe::NativeOptions::default();
    eframe::run_native("PipeDash", opts, Box::new(|cc| Box::new(PipeDash::new(cc))));
}
