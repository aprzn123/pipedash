use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use base64::engine::{general_purpose::URL_SAFE, Engine};
use flate2::read::GzDecoder;
use quick_xml::Reader;

struct User {
    name: String,
    id: Option<u64>,
}

struct InnerLevel; // TODO: write this

type Difficulty = u8;

enum Song {
    Official{id: i32 /*k8*/},
    Newgrounds{id: i32 /*k45*/},
}

struct Level {
    name: String, // k2
    song: Song,
}

pub fn gd_path() -> PathBuf {
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


struct LevelBuilder {
    name: Option<String>,
    song: Option<Song>,
}

impl Default for LevelBuilder {
    fn default() -> Self {
        Self {name: None, song: None}
    }
}

impl LevelBuilder {
    fn with_name(&mut self, name: String) {
        self.name = Some(name);
    }
    fn with_song(&mut self, song: Song) {
        self.song = Some(song);
    }
    fn make_level(self) -> Option<Level> {
        if let Some(name) = self.name {
            if let Some(song) = self.song {
                return Some(Level {name, song});
            }
        }
        None
    }
}

fn get_levels() -> Vec<Level> {
    let raw_save_data = {
        let mut save_file = File::open(gd_path().join("CCLocalLevels.dat")).expect("No save file found!");
        let mut sd = Vec::new();
        save_file.read_to_end(&mut sd);
        sd
    };
    let data_post_xor: Vec<u8> = raw_save_data.iter().map(|b| b ^ 11).collect();
    let data_post_b64 = URL_SAFE.decode(data_post_xor).unwrap();
    let plist = {
        let mut decoder = GzDecoder::<&[u8]>::new(data_post_b64.as_ref());
        let mut plist = String::new();
        if let Err(_) = decoder.read_to_string(&mut plist) {
            println!("Warning: Game save likely corrupted (gzip decode failed)");
        }
        plist
    };
    let reader = Reader::from_str(plist.as_ref());
    let mut out = vec![];
    loop {
        break out;
    }
}
