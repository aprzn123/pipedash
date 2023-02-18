use base64::engine::{general_purpose::URL_SAFE, Engine};
use eframe::egui::TextFormat;
use eframe::epaint::text::LayoutJob;
use flate2::read::GzDecoder;
use gd_plist::Value;
use reqwest::blocking as req;
use std::fs::File;
use std::io::{Cursor, Read};
use std::num::ParseIntError;
use std::path::PathBuf;
use thiserror::Error;

struct User {
    name: String,
    id: Option<u64>,
}

pub enum Song {
    Official { id: i32 /*k8*/ },
    Newgrounds { id: i32 /*k45*/ },
    Unknown,
}

#[derive(Clone)]
pub struct SongResponse([Option<String>; 9]);

struct Level {
    outer: OuterLevel,
    song: Song,
}

#[derive(Debug)]
pub struct OuterLevel {
    name: String,          // k2
    revision: Option<i64>, // k46
}

#[derive(Debug, Error, Clone)]
pub enum SongRequestError {
    #[error("Request failed")]
    ConnectionFailure,
    #[error("Index is not an int?????")]
    ParseFailure(#[from] ParseIntError),
    #[error("Not a Newgrounds song")]
    NotNewgrounds,
}

impl From<reqwest::Error> for SongRequestError {
    fn from(_: reqwest::Error) -> Self {
        Self::ConnectionFailure
    }
}

impl Song {
    pub fn get_response(&self) -> Result<SongResponse, SongRequestError> {
        match self {
            Self::Newgrounds { id } => {
                let mut out = SongResponse(Default::default());
                req::Client::new()
                    .post("http://boomlings.com/database/getGJSongInfo.php")
                    .body(format!(
                        r#" {{ "secret": "Wmfd2893gb7", "songID": {id} }} "#,
                    ))
                    .send()?
                    .text()?
                    .split("~|~")
                    .array_chunks()
                    .try_for_each(|[id, value]| -> Result<(), SongRequestError> {
                        out.0[id.parse::<usize>()? - 1] = Some(value.into());
                        Ok(())
                    })
                    .map(|_| out)
            }
            _ => Err(SongRequestError::NotNewgrounds),
        }
    }
}

impl SongResponse {
    pub fn id(&self) -> Option<i32> {
        self.0[0].as_deref().and_then(|s| s.parse().ok())
    }
    pub fn name(&self) -> Option<&str> {
        self.0[1].as_deref()
    }
    pub fn artist_id(&self) -> Option<i32> {
        self.0[2].as_deref().and_then(|s| s.parse().ok())
    }
    pub fn artist_name(&self) -> Option<&str> {
        self.0[3].as_deref()
    }
    pub fn size(&self) -> Option<i32> {
        self.0[4].as_deref().and_then(|s| s.parse().ok())
    }
    pub fn video_id(&self) -> Option<&str> {
        self.0[5].as_deref()
    }
    pub fn youtube_url(&self) -> Option<&str> {
        self.0[6].as_deref()
    }
    pub fn song_priority(&self) -> Option<i32> {
        self.0[8].as_deref().and_then(|s| s.parse().ok())
    }
    pub fn download_link(&self) -> Option<String> {
        self.0[9].as_deref().and_then(|url| urlencoding::decode(url).ok().map(std::borrow::Cow::into_owned))
    }
}

impl OuterLevel {
    pub fn load_all() -> Vec<Self> {
        let plist = get_local_level_plist();
        let levels: Vec<Self> = plist
            .as_dictionary()
            .and_then(|dict| dict.get("LLM_01"))
            .unwrap()
            .as_dictionary()
            .unwrap()
            .into_iter()
            .filter(|(key, _)| key.as_str() != "_isArr")
            .map(|(_, val)| {
                let mut builder = LevelBuilder::new();
                let props = val.as_dictionary().unwrap();
                if let Some(title) = props.get("k2") {
                    builder.with_name(title.as_string().unwrap().into());
                }
                if let Some(rev) = props.get("k46") {
                    builder.with_revision(rev.as_signed_integer().unwrap());
                }
                builder.build_outer_level().unwrap()
            })
            .collect();
        levels
    }

    pub fn display_name(&self) -> LayoutJob {
        if let Some(rev) = self.revision {
            let mut job = LayoutJob::default();
            job.append(&format!("{} ", self.name), 0f32, TextFormat::default());
            job.append(
                &format!("(rev {rev})"),
                0f32,
                TextFormat {
                    italics: true,
                    ..Default::default()
                },
            );
            job
        }
        else {
            let mut job = LayoutJob::default();
            job.append(&self.name, 0f32, TextFormat::default());
            job
        }
    }
}

pub fn save_path() -> PathBuf {
    let mut path_buf = home::home_dir().unwrap();
    #[cfg(unix)]
    path_buf.extend(
        [
            ".local",
            "share",
            "Steam",
            "steamapps",
            "compatdata",
            "322170",
            "pfx",
            "drive_c",
            "users",
            "steamuser",
            "AppData",
            "Local",
            "GeometryDash",
        ]
        .iter(),
    );
    #[cfg(windows)]
    path_buf.extend(["AppData", "Local", "GeometryDash"].iter());
    path_buf
}

#[derive(Default)]
struct LevelBuilder {
    name: Option<String>,
    song: Option<Song>,
    revision: Option<i64>,
}

impl LevelBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn with_name(&mut self, name: String) {
        self.name = Some(name);
    }

    fn with_song(&mut self, song: Song) {
        self.song = Some(song);
    }

    fn with_revision(&mut self, revision: i64) {
        self.revision = Some(revision);
    }

    fn build_level(self) -> Option<Level> {
        match self {
            Self {
                name: Some(name),
                song: Some(song),
                revision,
            } => Some(Level {
                song,
                outer: OuterLevel { name, revision },
            }),
            _ => None,
        }
    }

    fn build_outer_level(self) -> Option<OuterLevel> {
        match self {
            Self {
                name: Some(name),
                revision,
                ..
            } => Some(OuterLevel { name, revision }),
            _ => None,
        }
    }
}

fn get_local_level_plist() -> Value {
    let raw_save_data = {
        let mut save_file =
            File::open(save_path().join("CCLocalLevels.dat")).expect("No save file found!");
        let mut sd = Vec::new();
        save_file.read_to_end(&mut sd).unwrap();
        sd
    };
    let data_post_xor: Vec<u8> = raw_save_data
        .iter()
        .map(|b| b ^ 11)
        .filter(|&b| b != 0u8)
        .collect();
    let data_post_b64 = URL_SAFE.decode(data_post_xor).unwrap();
    let mut decoder = GzDecoder::<&[u8]>::new(data_post_b64.as_ref());
    let mut plist = String::new();
    if decoder.read_to_string(&mut plist).is_err() {
        println!("Warning: Game save likely corrupted (gzip decode failed)");
    }
    Value::from_reader(Cursor::new(plist)).unwrap()
}
