#![feature(iter_array_chunks)]
//#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(dead_code)]

mod gd;
mod music;

use eframe::egui;
use reqwest::blocking as req;
use std::boxed::Box;
use std::collections::VecDeque;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::mem;
use thiserror::Error;
use std::time;
use rodio::source::Source;

fn project_dirs() -> directories::ProjectDirs {
    directories::ProjectDirs::from("xyz", "interestingzinc", "pipedash").expect("Home dir missing?")
}

struct PipeDash {
    msg_queue: VecDeque<Message>,
    selected_level: Option<usize>,
    level_list: Vec<gd::Level>,
    loaded_level_checksum: Option<(gd::Level, md5::Digest)>,
    editor_mode: EditorMode,
    errors: VecDeque<Box<dyn Error>>,
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
    CloseError,
    LoadLevel,
}

enum EditorMode {
    NoSong,
    RhythmWizard {editor: WizardEditor, song: Song},
    Full {editor: Editor, song: Song}
}

#[derive(Default)]
struct Editor {
    state: EditorState,
    data: GdlData,
}

struct WizardEditor {
    state: EditorState,
    data: WizardData,
}

struct EditorState {
    scroll_pos: f64,
    pts_per_second: f64, // zoom level
    subdivisions: u32,
}

struct GdlData {
    green_lines: music::Lines,
    orange_lines: music::Lines,
    yellow_lines: music::Lines,
    beat_rate: music::BeatRate,
    time_signatures: music::TimeSignature,
}

struct WizardData {
    green_lines: music::Lines<time::Duration>,
    orange_lines: music::Lines<time::Duration>,
    yellow_lines: music::Lines<time::Duration>,
    beat_rate: Option<music::BeatRate>,
    time_signatures: Option<music::TimeSignature>,
}

struct Song {
    name: String,
    id: i64,
    buffer: rodio::buffer::SamplesBuffer<i16>,
    length: time::Duration,
    stream: rodio::OutputStream,
    sink: rodio::Sink,
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
    #[error("Unable to create audio stream")]
    StreamError(#[from] rodio::StreamError),
    #[error("Unable to create audio sink")]
    SinkError(#[from] rodio::PlayError),
}

struct BeatRateWidget<'a> {
    state: &'a mut EditorState,
    beat_rate: Option<&'a mut music::BeatRate>,
    song: &'a Song, // for waveform
}

struct TimeSignatureWidget<'a> {
    state: &'a mut EditorState,
    time_signatures: Option<&'a mut music::TimeSignature>,
    song: &'a Song, // for waveform
}

struct LinesWidget<'a, T = music::BeatPosition> 
where T: Ord
{
    state: &'a mut EditorState,
    lines: &'a mut music::Lines<T>,
    color: Color,
    song: &'a Song, // for waveform
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

impl EditorMode {
    pub fn display(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx();
        match self {
            EditorMode::RhythmWizard { editor, song } => {
                ui.label("Rhythm Wizard");
            },
            EditorMode::Full { editor, song } => {
                editor.handle_keyboard_input(ctx, song);
                ui.label("Editor");
                ui.add(editor.time_signature_widget(song));
                ui.add(editor.beat_rate_widget(song));
                ui.add(editor.lines_widget(Color::Green, song));
                ui.add(editor.lines_widget(Color::Orange, song));
                ui.add(editor.lines_widget(Color::Yellow, song));
            },
            EditorMode::NoSong => {
                ui.label("No song to edit");
            },
        }
    }
}

impl Default for EditorState {
    fn default() -> Self {
        EditorState { scroll_pos: 0.0, pts_per_second: 10.0, subdivisions: 4 }
    }
}

impl Default for GdlData {
    fn default() -> Self {
        GdlData {
            green_lines: Default::default(),
            orange_lines: Default::default(),
            yellow_lines: Default::default(),
            beat_rate: music::StaticBeatRate::from_bpm(120.0).into(),
            time_signatures: music::StaticTimeSignature::new(4, 4).into(),
        }
    }
}

impl Song {
    pub fn try_new(gd_song: &gd::Song) -> Result<Self, SongError> {
        if let &gd::Song::Newgrounds { id } = gd_song {
            let song_result = gd_song.get_response();
            let song_path = gd::save_path().join(format!("{id}.mp3"));

            let (file, name) = match (File::open(&song_path), song_result) {
                (Ok(file), response) => {
                    let name = response
                        .ok()
                        .and_then(|response| response.name().map(Into::into))
                        .unwrap_or("Missing Name".into());

                    (file, name)
                }
                (Err(_), Ok(response)) => {
                    let song_blob = response
                        .download_link()
                        .ok_or(SongError::MissingLink)
                        .and_then(|link| Ok(req::get(link)?.bytes()?))?;

                    let mut file = File::open(&song_path)?;
                    file.write_all(&song_blob)?;

                    (file, response.name().unwrap_or_default().into())
                }
                (Err(err), Err(_)) => return Err(err.into()),
            };
            
            let decoder = rodio::Decoder::new_mp3(file)?;

            let channels = decoder.channels();
            let sample_rate = decoder.sample_rate();
            let samples: Vec<i16> = decoder.collect();

            let length = time::Duration::from_secs(samples.len() as u64 / channels as u64 / sample_rate as u64);
            let buffer = rodio::buffer::SamplesBuffer::new(channels, sample_rate, samples);

            let (stream, stream_handle) = rodio::OutputStream::try_default()?;
            let sink = rodio::Sink::try_new(&stream_handle)?;

            Ok(Self { name, id, buffer, length, stream, sink })
        } else {
            Err(SongError::NotNewgrounds)
        }
    }
}

impl Editor {
    pub fn beat_rate_widget<'a>(&'a mut self, song: & 'a mut Song) -> BeatRateWidget {
        BeatRateWidget {
            state: &mut self.state,
            beat_rate: Some(&mut self.data.beat_rate),
            song,
        }
    }

    pub fn time_signature_widget<'a>(&'a mut self, song: &'a mut Song) -> TimeSignatureWidget {
        TimeSignatureWidget {
            state: &mut self.state,
            time_signatures: Some(&mut self.data.time_signatures),
            song,
        }
    }

    pub fn lines_widget<'a>(&'a mut self, col: Color, song: &'a mut Song) -> LinesWidget {
        LinesWidget {
            state: &mut self.state,
            lines: match col {
                Color::Green => &mut self.data.green_lines,
                Color::Yellow => &mut self.data.yellow_lines,
                Color::Orange => &mut self.data.orange_lines,
            },
            color: col,
            song,
        }
    }

    /// points in width of entire song
    fn song_width(&self, song: &Song) -> f64 {
        song.length.as_secs_f64() * self.state.pts_per_second
    }

    fn play_pause(&self, song: &Song) {
        todo!("toggle song playback")
    }

    fn handle_keyboard_input(&mut self, ctx: &egui::Context, song: &Song) {
        use egui::Key;
        use egui::Event;
        ctx.input().events
            .iter()
            .for_each(|ev| match ev {
                Event::Key { key: Key::ArrowLeft, pressed: true, modifiers } => self.scroll(-5.0, song),
                Event::Key { key: Key::ArrowRight, pressed: true, modifiers } => self.scroll(5.0, song),
                Event::Key { key: Key::Space, pressed: true, modifiers } if modifiers.is_none() => self.play_pause(song),
                _ => (),
            });
    }

    fn scroll(&mut self, pts: f64, song: &Song) {
        self.state.scroll_pos += pts;
        self.state.scroll_pos.clamp(0f64, self.song_width(song));
    }

}

impl WizardEditor {
    pub fn beat_rate_widget<'a>(&'a mut self, song: &'a mut Song) -> BeatRateWidget {
        BeatRateWidget {
            state: &mut self.state,
            beat_rate: self.data.beat_rate.as_mut(),
            song,
        }
    }

    pub fn time_signature_widget<'a>(&'a mut self, song: &'a mut Song) -> TimeSignatureWidget {
        TimeSignatureWidget {
            state: &mut self.state,
            time_signatures: self.data.time_signatures.as_mut(),
            song,
        }
    }

    pub fn lines_widget<'a>(&'a mut self, col: Color, song: &'a mut Song) -> LinesWidget<time::Duration> {
        LinesWidget {
            state: &mut self.state,
            lines: match col {
                Color::Green => &mut self.data.green_lines,
                Color::Yellow => &mut self.data.yellow_lines,
                Color::Orange => &mut self.data.orange_lines,
            },
            color: col,
            song,
        }
    }
}

impl From<gd::RawLinesTriplet> for WizardData {
    fn from(lines: gd::RawLinesTriplet) -> Self {
        Self {
            green_lines: lines.green,
            orange_lines: lines.orange,
            yellow_lines: lines.yellow,
            beat_rate: None,
            time_signatures: None,
        }
    }
}

impl From<gd::RawLinesTriplet> for WizardEditor {
    fn from(lines: gd::RawLinesTriplet) -> Self {
        Self { state: Default::default(), data: lines.into() }
    }
}

impl<'a> egui::Widget for BeatRateWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, res) = allocate_editor_space(ui);
        // handle interactions
        // draw widget
        if ui.is_rect_visible(rect) {
            ui.painter()
                .rect_filled(rect, 0.0, eframe::epaint::Color32::from_gray(0));
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
                .rect_filled(rect, 0.0, eframe::epaint::Color32::from_gray(0));
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
                .rect_filled(rect, 0.0, eframe::epaint::Color32::from_gray(0));
        }
        res
    }
}

impl PipeDash {
    fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            selected_level: None,
            msg_queue: VecDeque::new(),
            level_list: gd::Level::load_all(),
            loaded_level_checksum: None,
            errors: VecDeque::new(),
            editor_mode: EditorMode::NoSong,
        }
    }

    fn side_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::left("level_picker")
            .default_width(100.0)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    if ui
                        .add_enabled(
                            self.selected_level.is_some(),
                            egui::Button::new("Load Level"),
                        )
                        .clicked()
                    {
                        self.msg_queue.push_back(Message::LoadLevel);
                    }
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
                use EditorMode::*;
                ui.label(
                    egui::RichText::new(match &self.editor_mode {
                        RhythmWizard { song, .. } | Full { song, .. } => &song.name,
                        NoSong => "No song loaded...",
                    })
                    .size(32.0),
                );
                ui.label(
                    egui::RichText::new(match &self.editor_mode {
                        RhythmWizard { song, .. } | Full { song, .. } => song.id.to_string(),
                        NoSong => "No song loaded...".into(),
                    })
                    .size(20.0),
                );

                self.editor_mode.display(ui);
            });
        });
    }

    fn handle_message(&mut self, message: Message) {
        match message {
            Message::LevelSelected(idx) => self.selected_level = Some(idx),
            Message::CloseError => { self.errors.pop_front(); },
            Message::LoadLevel => {
                // Load song, GdlData, checksum; if there are no lines go straight into editor, otherwise
                // start the rhythm wizard
                let level = self
                    .selected_level
                    .and_then(|idx| self.level_list.get(idx))
                    .unwrap() // will not panic. selected_level range is same as level_list...
                    .clone(); // ...length - 1; message will not be sent if selected_level is none

                let song = match Song::try_new(&level.song()) {
                    Ok(song) => song,
                    Err(e) => {
                        self.errors.push_front(Box::new(e)); 
                        return;
                    },
                };

                let inner_level = level.load_inner();
                let lines = inner_level.get_lines();
                if lines.orange.empty() && lines.green.empty() && lines.yellow.empty() {
                    self.editor_mode = EditorMode::Full {
                        editor: Default::default(),
                        song,
                    }
                } else {
                    self.editor_mode = EditorMode::RhythmWizard {
                        editor: lines.into(),
                        song,
                    }
                }

                self.loaded_level_checksum = Some((level, inner_level.hash()));
            }
        }
    }

    fn handle_messages(&mut self) {
        for message in mem::take(&mut self.msg_queue) {
            log::info!("{message:?}");
            self.handle_message(message);
        }
    }
}

impl eframe::App for PipeDash {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(2.0);

        if let Some(boxed_err) = &self.errors.front() {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label(boxed_err.to_string());
                if ui.button("Close").clicked() {
                    self.msg_queue.push_back(Message::CloseError);
                }
            });
        } else {
            self.side_panel(ctx, frame);
            self.center_panel(ctx, frame);
        }

        self.handle_messages();
    }
}

fn main() {
    std::fs::create_dir_all(project_dirs().data_local_dir());
    simplelog::WriteLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        File::create(project_dirs().data_local_dir().join("run.log"))
            .expect("file creation failed?"),
    )
    .map_err(|e| println!("Logging uninitialized"));
    let app: PipeDash;
    let opts = eframe::NativeOptions::default();
    eframe::run_native("PipeDash", opts, Box::new(|cc| Box::new(PipeDash::new(cc))));
}
