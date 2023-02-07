// TODO: SWITCH BEAT RATE TO CHANGE ON BEATS INSTEAD OF DURATIONS, 
// TODO: BECAUSE THAT WAS A STUPID IDEA

use chrono::Duration;
use std::collections::BTreeMap;
use ordered_float::OrderedFloat as Float;

pub type BeatPosition = Float<f32>;

/// Like BPM, but not necessarily represented in terms of minutes
/// Only BPM jumps for now; no smooth accel/decel
pub struct BeatRate {
    initial: StaticBeatRate,
    changes: BTreeMap<BeatPosition, StaticBeatRate>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
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

impl BeatRate {
    pub fn at_beat(&self, pos: BeatPosition) -> StaticBeatRate {
        match self.changes.first_key_value() {
            Some((first_change, _)) => {
                if &pos < first_change {
                    self.initial
                } else {
                    *self.changes.iter().rev().find(|&el| el.0 <= &pos).unwrap().1
                }
            },
            None => self.initial,
        }
    }
    
    pub fn add_change(&mut self, new_pos: BeatPosition, new_rate: StaticBeatRate) {
        self.changes.insert(new_pos, new_rate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rate_when_no_changes() {
        let rate: BeatRate = StaticBeatRate::from_bpm(100.0).into();
        assert_eq!(rate.at_beat(5.0.into()), StaticBeatRate::from_bpm(100.0));
    }

    #[test]
    fn rate_before_changes() {
        let mut rate: BeatRate = StaticBeatRate::from_bpm(100.0).into();
        rate.add_change(5.0.into(), StaticBeatRate::from_bpm(120.0));
        assert_eq!(rate.at_beat(3.0.into()), StaticBeatRate::from_bpm(100.0));
    }

    #[test]
    fn rate_after_change() {
        let mut rate: BeatRate = StaticBeatRate::from_bpm(100.0).into();
        rate.add_change(5.0.into(), StaticBeatRate::from_bpm(120.0));
        rate.add_change(10.0.into(), StaticBeatRate::from_bpm(140.0));
        assert_eq!(rate.at_beat(6.0.into()), StaticBeatRate::from_bpm(120.0));
    }

    #[test]
    fn rate_at_change() {
        let mut rate: BeatRate = StaticBeatRate::from_bpm(100.0).into();
        rate.add_change(5.0.into(), StaticBeatRate::from_bpm(120.0));
        rate.add_change(10.0.into(), StaticBeatRate::from_bpm(140.0));
        assert_eq!(rate.at_beat(5.0.into()), StaticBeatRate::from_bpm(120.0));
    }
}
