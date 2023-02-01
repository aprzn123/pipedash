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
    sink: Sink,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Play,
    Pause,
}

impl Sandbox for Guider {
    type Message = Message;
    fn new() -> Self {
        let gd_path = gd_path();
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let music_file = BufReader::new(File::open(gd_path.join("613929.mp3")).unwrap());
        let source = Decoder::new(music_file).unwrap();
        //sink.append(source);
        //sink.pause();
        Self {sink}
    }

    fn view(&self) -> Element<Message> {
        // column![button("pause").on_press(Message::Pause), button("play").on_press(Message::Play)].into()
        row![
            scrollable(["lvl 1", "lvl 2", "lvl 3"].iter().fold(
                Column::new(), |col, item| col.push(button(*item).on_press(Message::Pause))
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
            Message::Pause => self.sink.pause(),
            Message::Play => self.sink.play(),
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
