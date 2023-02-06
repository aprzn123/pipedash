// TODO: SWITCH BEAT RATE TO CHANGE ON BEATS INSTEAD OF DURATIONS, 
// TODO: BECAUSE THAT WAS A STUPID IDEA

use chrono::Duration;
use std::collections::BTreeMap;

/// Like BPM, but not necessarily represented in terms of minutes
/// Only BPM jumps for now; no smooth accel/decel
pub struct BeatRate {
    initial: StaticBeatRate,
    changes: BTreeMap<Duration, StaticBeatRate>,
}

pub struct StaticBeatRate(Duration);

impl StaticBeatRate {
    pub fn from_bpm(bpm: f32) -> Self {
        Self(Duration::microseconds(60_000_000 / bpm as i64))
    }
}

impl From<StaticBeatRate> for BeatRate {
    fn from(rhs: StaticBeatRate) -> Self {
        Self {
            initial: rhs,
            changes: BTreeMap::new(),
        }
    }
}
