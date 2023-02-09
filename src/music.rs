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

pub struct TimeSignature {
    initial: StaticTimeSignature,
    changes: BTreeMap<BeatPosition, StaticTimeSignature>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StaticTimeSignature {
    numerator: u32,
    denominator: u32,
}


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

/// Changes: when the time signature changes, the bar immediately resets
impl StaticTimeSignature {
    pub fn new(numerator: u32, denominator: u32) -> Self {Self {numerator, denominator}}

    fn beats_per_bar(&self) -> BeatPosition {(self.numerator as f32).into()}
}

impl From<StaticTimeSignature> for TimeSignature {
    fn from(rhs: StaticTimeSignature) -> Self {
        Self {
            initial: rhs,
            changes: BTreeMap::new(),
        }
    }
}

impl TimeSignature {
    pub fn add_change(&mut self, position: BeatPosition, signature: StaticTimeSignature) {
        self.changes.insert(position, signature);
    }

    pub fn at_beat(&self, pos: BeatPosition) -> StaticTimeSignature {
        match self.changes.first_key_value() {
            Some((first_change, _)) => {
                if &pos < first_change {
                    self.initial
                } else {
                    *self.changes.iter().rev().find(|&el| el.0 <= &pos).unwrap().1
                }
            },
            none => self.initial,
        }
    }

    pub fn position_in_bar(&self, pos: BeatPosition) -> BeatPosition {
        match self.changes.first_key_value() {
            Some((first_change, first_change_sig)) => {
                if &pos < first_change {
                    pos % self.initial.beats_per_bar()
                } else {
                    let (signature_start_point, signature) = self.changes.iter().rev().find(|&el| el.0 <= &pos).unwrap();
                    (pos - signature_start_point) % signature.beats_per_bar()
                }
            },
            None => pos % self.initial.beats_per_bar(),
        }
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
