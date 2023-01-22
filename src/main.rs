use iced::{
    widget::{button, column, text},
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


fn gd_path() -> PathBuf {
    let mut path_buf = home::home_dir().unwrap();
    #[cfg(unix)]
    path_buf.extend([".local", "share", "Steam", "steamapps", 
                     "compatdata", "322170", "pfx", "drive_c",
                     "users", "steamuser", "AppData", "Local",
                     "GeometryDash"].iter());
    #[cfg(windows)]
    path_buf.extend(["AppData", "Local", "GeometryDash"].iter());
    path_buf
}


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
        println!("{:?}", gd_path.as_os_str());
        let music_file = BufReader::new(File::open(gd_path.join("613929.mp3")).unwrap());
        let source = Decoder::new(music_file).unwrap();
        //sink.append(source);
        //sink.pause();
        Self {sink}
    }

    fn view(&self) -> Element<Message> {
        column![button("pause").on_press(Message::Pause), button("play").on_press(Message::Play)].into()
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
    Guider::run(Settings::default())
}
