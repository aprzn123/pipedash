use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use base64::engine::{general_purpose::URL_SAFE, Engine};
use flate2::read::GzDecoder;

struct User {
    name: String,
    id: Option<u64>,
}

enum Song {
    Official{id: i32 /*k8*/},
    Newgrounds{id: i32 /*k45*/},
    Unknown
}

struct Level {
    outer: OuterLevel,
    song: Song,
}

struct OuterLevel {
    name: String, // k2
    revision: i32, // k46
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
    revision: Option<i32>,
}

impl Default for LevelBuilder {
    fn default() -> Self {
        Self {name: None, song: None, revision: None}
    }
}

impl LevelBuilder {
    fn new() -> Self {Self::default()}

    fn with_name(&mut self, name: String) {
        self.name = Some(name);
    }

    fn with_song(&mut self, song: Song) {
        self.song = Some(song);
    }

    fn with_revision(&mut self, revision: i32) {
        self.revision = Some(revision);
    }

    fn make_level(self) -> Option<Level> {
        match self {
            Self {
                name: Some(name), 
                song: Some(song), 
                revision: Some(revision)
            } => Some(Level{song, outer: OuterLevel {name, revision}}),
            _ => None,
        }
    }

    fn make_outer_level(self) -> Option<OuterLevel> {
        match self {
            Self {
                name: Some(name), 
                song, 
                revision: Some(revision)
            } => Some(OuterLevel {name, revision}),
            _ => None,
        }
    }
}

fn verbosify_plist(plist: &str) -> String {
    plist
    .replace("</d>","</dict>")
    .replace("</k>","</key>")
    .replace("<d>","<dict>")
    .replace("<d />","<dict />")
    .replace("<d/>","<dict />")
    .replace("<k>","<key>")
    .replace("<s>","<string>")
    .replace("</s>","</string>")
    .replace("<i>","<integer>")
    .replace("</i>","</integer>")
    .replace("<t />","<true />")
    .replace("<t/>","<true />")
    .replace("<r>","<real>")
    .replace("</r>","</real>")
}

fn concisify_plist(plist: &str) -> String {
    plist
    .replace("</dict>","</d>")
    .replace("</key>","</k>")
    .replace("<dict>","<d>")
    .replace("<dict />","<d/>")
    .replace("<key>","<k>")
    .replace("<string>","<s>")
    .replace("</string>","</s>")
    .replace("<integer>","<i>")
    .replace("</integer>","</i>")
    .replace("<true />","<t/>")
    .replace("<real>","<r>")
    .replace("</real>","</r>")
}

fn get_local_level_plist() -> String {
    let raw_save_data = {
        let mut save_file = File::open(gd_path().join("CCLocalLevels.dat")).expect("No save file found!");
        let mut sd = Vec::new();
        save_file.read_to_end(&mut sd).unwrap();
        sd
    };
    let data_post_xor: Vec<u8> = raw_save_data.iter().map(|b| b ^ 11).collect();
    let data_post_b64 = URL_SAFE.decode(data_post_xor).unwrap();
    let mut decoder = GzDecoder::<&[u8]>::new(data_post_b64.as_ref());
    let mut plist = String::new();
    if let Err(_) = decoder.read_to_string(&mut plist) {
        println!("Warning: Game save likely corrupted (gzip decode failed)");
    }
    verbosify_plist(&plist)
}

fn get_outer_levels() -> Vec<Level> {
    let plist = get_local_level_plist();
}
