#![feature(iter_array_chunks)]
//#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

mod gd;
mod music;

use eframe::egui;
use reqwest::blocking as req;
use std::boxed::Box;
use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use thiserror::Error;

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
    state: EditorState,
    data: GdlData,
}

struct EditorState {
    scroll_pos: f32,
    pts_per_second: f32, // zoom level
    subdivisions: f32,
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
    id: i32,
    decoder: rodio::Decoder<File>,
}

#[derive(Error, Debug)]
enum SongError {
    #[error("Not a Newgrounds song")]
    NotNewgrounds,
    #[error("Song mp3 couldn't be downloaded")]
    MissingFile(#[from] std::io::Error),
    #[error("Couldn't decode mp3 file")]
    BrokenSong(#[from] rodio::decoder::DecoderError),
    #[error("Couldn't access song data on servers")]
    GdServerError(#[from] gd::SongRequestError),
    #[error("Couldn't fetch song from Newgrounds")]
    NgServerError(#[from] reqwest::Error),
    #[error("Missing download link")]
    MissingLink,
}

struct BeatRateWidget<'a> {
    state: &'a mut EditorState,
    beat_rate: &'a mut music::BeatRate,
}
struct TimeSignatureWidget<'a> {
    state: &'a mut EditorState,
    time_signatures: &'a mut music::TimeSignature,
}
struct LinesWidget<'a> {
    state: &'a mut EditorState,
    lines: &'a mut music::Lines,
    color: Color,
}
struct WaveformWidget<'a> {
    state: &'a mut EditorState,
    song: &'a Song,
}

fn allocate_editor_space(ui: &mut egui::Ui) -> (egui::Rect, egui::Response) {
    let max_rect = ui.max_rect();
    let preferred_size = egui::Vec2::new(max_rect.size().x, 60.0);
    ui.allocate_exact_size(preferred_size, egui::Sense::click_and_drag())
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

impl Song {
    pub fn try_new(gd_song: gd::Song) -> Result<Self, SongError> {
        if let gd::Song::Newgrounds { id } = gd_song {
            let song_response = gd_song.get_response();
            let song_path = gd::save_path().join(format!("{id}.mp3"));

            let (file, name) = match (File::open(&song_path), song_response) {
                (Ok(file), response) => {
                    let name = response
                        .ok()
                        .and_then(|response| response.name().map(Into::into))
                        .unwrap_or_default();

                    (file, name)
                }
                (Err(_), Ok(response)) => {
                    let song_blob = response
                        .download_link()
                        .ok_or(SongError::MissingLink)
                        .and_then(|link| Ok(req::get(link)?.bytes()?))?;

                    let mut file = File::open(&song_path)?;
                    file.write_all(&song_blob)?;

                    (file, response.name().unwrap_or("").into())
                }
                (Err(err), Err(_)) => return Err(err.into()),
            };

            let decoder = rodio::Decoder::new_mp3(file)?;
            
            Ok(Self { name, id, decoder })
        } else {
            Err(SongError::NotNewgrounds)
        }
    }
}


impl Editor {
    pub fn beat_rate_widget(&mut self) -> BeatRateWidget {
        BeatRateWidget {
            state: &mut self.state,
            beat_rate: &mut self.data.beat_rate,
        }
    }

    pub fn time_signature_widget(&mut self) -> TimeSignatureWidget {
        TimeSignatureWidget {
            state: &mut self.state,
            time_signatures: &mut self.data.time_signatures,
        }
    }

    pub fn lines_widget(&mut self, col: Color) -> LinesWidget {
        LinesWidget {
            state: &mut self.state,
            lines: match col {
                Color::Green => &mut self.data.green_lines,
                Color::Yellow => &mut self.data.yellow_lines,
                Color::Orange => &mut self.data.orange_lines,
            },
            color: col,
        }
    }

    pub fn waveform_widget<'a>(&'a mut self, song: &'a Song) -> WaveformWidget {
        WaveformWidget {
            state: &mut self.state,
            song,
        }
    }
}

impl<'a> egui::Widget for BeatRateWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, res) = allocate_editor_space(ui);
        // handle interactions
        // draw widget
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

impl<'a> egui::Widget for LinesWidget<'a> {
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
                state: EditorState {
                    scroll_pos: 0f32,
                    pts_per_second: 5f32,
                    subdivisions: 4.0,
                },
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

                if let Some(song) = &self.loaded_song {
                    ui.add(self.editor.time_signature_widget());
                    ui.add(self.editor.beat_rate_widget());
                    ui.add(self.editor.lines_widget(Color::Green));
                    ui.add(self.editor.lines_widget(Color::Orange));
                    ui.add(self.editor.lines_widget(Color::Yellow));
                    ui.add(self.editor.waveform_widget(song));
                }
            });
        });
    }

    fn handle_messages(&mut self) {
        for message in self.msg_queue.drain(..) {
            println!("{message:?}");
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
