mod gd;

use iced::{
    widget::{
        button, 
        column, 
        row, 
        text, 
        scrollable, 
        radio::Radio,
        Column
    },
    Sandbox, 
    Settings,
    Element,
    Alignment,
};
use rodio::{
    source::Source,
    Decoder,
    OutputStream,
    Sink,
};
use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
};
use gd::gd_path;


struct Guider {
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Msg,
}

impl Sandbox for Guider {
    type Message = Message;
    fn new() -> Self {
        Self {}
    }

    fn view(&self) -> Element<Message> {
        // column![button("pause").on_press(Message::Pause), button("play").on_press(Message::Play)].into()
        row![
            scrollable(["lvl 1", "lvl 2", "lvl 3"].iter().fold(
                Column::new(), |col, item| col.push(button(*item).on_press(Message::Msg))
            )),
            column![
                row![
                    column![text("Title"), text("Song Title & ID")],
                    column![
                        text("G"),
                        text("Y"),
                        text("O"),
                    ]
                ], 
                row![text("zoom in"), text("zoom out"), text("delete"), text("add"), text("edit")], 
                text("view canvas"),
            ]
        ].into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Msg => {println!("Message received!")}
        }
    }

    fn title(&self) -> String {
        String::from("Guider")
    }
}


fn main() -> iced::Result {
    println!("{:?}", gd::get_outer_levels());
    Guider::run(Settings::default())
}
