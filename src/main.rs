mod gd;

use eframe;
use eframe::egui;
use std::boxed::Box;

/*use iced::{
    widget::{
        button, 
        column, 
        row, 
        text, 
        scrollable, 
        Column,
        Row,
    },
    Application, 
    Settings,
    Element,
    Alignment,
    Theme,
    executor,
    Command,
};
use rodio::{
    source::Source,
    Decoder,
    OutputStream,
    Sink,
};


struct Guider {
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Msg,
}

impl Application for Guider {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;
    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self {}, Command::none())
    }

    fn view(&self) -> Element<Self::Message> {
        // column![button("pause").on_press(Message::Pause), button("play").on_press(Message::Play)].into()
        row![
            scrollable(["lvl 1", "lvl 2", "lvl 3"].iter().fold(
                Column::new(), |col, item| col.push(button(*item).on_press(Message::Msg))
            )),
            column![
                row![
                    column![text("Some Title Info?")],
                    column![
                        text("G"),
                        text("Y"),
                        text("O"),
                    ]
                ], 
                row![text("zoom in"), text("zoom out"), text("delete"), text("add"), text("edit")], 
                text("view canvas"),
            ]
        ].align_items(Alignment::Fill).into()
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::Msg => {println!("Message received!")}
        }
        Command::none()
    }

    fn title(&self) -> String {
        String::from("Guider")
    }
}

*/

struct PipeDash {
    selected: Option<i32>,
}

impl PipeDash {
    fn new(_cc: &eframe::CreationContext) -> Self {
        Self {selected: None}
    }
}

impl eframe::App for PipeDash {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::left("level_picker").show(ctx, |ui| {
            if ui.selectable_label(self.selected == Some(0), "lbl 1").clicked() {
                self.selected = Some(0);
            }
            if ui.selectable_label(self.selected == Some(1), "lbl 2").clicked() {
                self.selected = Some(1);
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("hello world!");
        });
    }
}

fn main() {
    println!("{:?}", gd::get_outer_levels());
    let app: PipeDash;
    let opts = eframe::NativeOptions::default();
    eframe::run_native("PipeDash", opts, Box::new(|cc| Box::new(PipeDash::new(cc))));
}

