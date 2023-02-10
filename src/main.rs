mod gd;
mod music;

use eframe;
use eframe::egui;
use std::boxed::Box;
use std::collections::VecDeque;
use std::marker::PhantomData;
use symphonia::core::io::{MediaSource, MediaSourceStream};

struct PipeDash {
    msg_queue: VecDeque<Message>,
    selected_level: Option<usize>,
    level_list: Vec<gd::OuterLevel>,
    loaded_song: Option<Song>,
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
    subdivisions: f32,
    data: GdlData,
}

struct GdlData {
    green_lines: music::Lines,
    orange_lines: music::Lines,
    yellow_lines: music::Lines,
    beat_rate: music::BeatRate,
    time_signatures: music::TimeSignature,
}

struct Song {
    name: String,
    id: u32,
    stream: MediaSourceStream,
}

struct Orange;
struct Yellow;
struct Green;

struct BeatRateWidget<'a>(&'a mut Editor);
struct TimeSignatureWidget<'a>(&'a mut Editor);
struct LinesWidget<'a, C: WithColor>(&'a mut Editor, PhantomData<C>);
struct WaveformWidget<'a>(&'a mut Editor);

trait WithColor {
    const COLOR: Color;
}

impl From<Color> for eframe::epaint::Color32 {
    fn from(rhs: Color) -> Self {
        match rhs {
            Color::Green => Self::from_rgb(0, 255, 0),
            Color::Orange => Self::from_rgb(255, 127, 0),
            Color::Yellow => Self::from_rgb(255, 255, 0),
        }
    }
}

impl WithColor for Orange {
    const COLOR: Color = Color::Orange;
}
impl WithColor for Yellow {
    const COLOR: Color = Color::Yellow;
}
impl WithColor for Green {
    const COLOR: Color = Color::Green;
}

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

    pub fn waveform_widget(&mut self) -> WaveformWidget {
        WaveformWidget(self)
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

impl<'a, C: WithColor> egui::Widget for LinesWidget<'a, C> {
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

impl<'a> egui::Widget for WaveformWidget<'a> {
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
            loaded_song: None,
            editor: Editor {
                scroll_pos: 0f32,
                pts_per_second: 5f32,
                subdivisions: 4.0,
                data: GdlData {
                    beat_rate: music::StaticBeatRate::from_bpm(120f32).into(),
                    time_signatures: music::StaticTimeSignature::new(4, 4).into(),
                    green_lines: music::Lines::new(),
                    orange_lines: music::Lines::new(),
                    yellow_lines: music::Lines::new(),
                },
            },
        }
    }

    fn side_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::left("level_picker")
            .default_width(100f32)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.button("Load Level");
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
            });
    }

    fn center_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.label(
                    egui::RichText::new(match &self.loaded_song {
                        Some(song) => &song.name,
                        None => "No song loaded...",
                    })
                    .size(32.0),
                );
                ui.label(
                    egui::RichText::new(match &self.loaded_song {
                        Some(song) => song.id.to_string(),
                        None => "No song loaded...".into(),
                    })
                    .size(20.0),
                );

                ui.add(self.editor.time_signature_widget());
                ui.add(self.editor.beat_rate_widget());
                ui.add(self.editor.lines_widget::<Green>());
                ui.add(self.editor.lines_widget::<Yellow>());
                ui.add(self.editor.lines_widget::<Orange>());
                ui.add(self.editor.waveform_widget());
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
