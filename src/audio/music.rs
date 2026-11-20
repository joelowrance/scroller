//! The music sequencer: songs written as note data, rendered to one buffer.
//!
//! A song has a tempo, a few channels (each with an instrument), named
//! patterns and an order. A pattern gives each channel one line of
//! whitespace-separated rows:
//!
//! - a note name (`C4`, `F#3`, `Bb5`) starts a note,
//! - `-` holds the previous note for another row,
//! - `.` is a rest (it releases the previous note),
//! - any key of the channel's `kit` plays that drum.
//!
//! A looping song is rendered to exactly its length in rows, with release
//! tails wrapped to the start, so the buffer loops without a click.

use std::collections::HashMap;

use serde::Deserialize;

use super::synth::{self, Envelope, Tone, Vibrato, Wave};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(default)]
pub struct Instrument {
    pub wave: Wave,
    pub env: Envelope,
    pub slide: f32,
    pub vibrato: Vibrato,
    pub volume: f32,
}

impl Default for Instrument {
    fn default() -> Self {
        let tone = Tone::default();
        Self {
            wave: tone.wave,
            env: tone.env,
            slide: 0.0,
            vibrato: Vibrato::default(),
            volume: 0.25,
        }
    }
}

/// A drum: an instrument played at a fixed pitch.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub struct Drum {
    pub instrument: Instrument,
    pub freq: f32,
    /// How long the drum is held, in seconds (drums ignore `-`).
    pub gate: f32,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Channel {
    pub instrument: Instrument,
    pub kit: HashMap<String, Drum>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SongDef {
    pub bpm: f32,
    /// Rows per beat; 4 makes each row a sixteenth note.
    pub rows_per_beat: u32,
    #[serde(default = "yes")]
    pub looping: bool,
    pub channels: Vec<Channel>,
    /// Each pattern has one line of rows per channel.
    pub patterns: HashMap<String, Vec<String>>,
    pub order: Vec<String>,
}

fn yes() -> bool {
    true
}

/// A note or drum hit placed on the song's timeline.
#[derive(Debug, PartialEq)]
struct Event {
    row: usize,
    rows: usize,
    sound: Sound,
}

#[derive(Debug, PartialEq)]
enum Sound {
    Note(f32),
    Drum(String),
}

impl SongDef {
    pub fn samples_per_row(&self) -> usize {
        let seconds = 60.0 / (self.bpm * self.rows_per_beat as f32);
        (seconds * synth::SAMPLE_RATE as f32).round() as usize
    }

    /// The rows of one channel across the whole song, in order.
    fn channel_rows(&self, channel: usize) -> Result<Vec<&str>, String> {
        let mut rows = Vec::new();
        for name in &self.order {
            let pattern = self
                .patterns
                .get(name)
                .ok_or_else(|| format!("unknown pattern '{name}'"))?;
            if pattern.len() != self.channels.len() {
                return Err(format!(
                    "pattern '{name}' has {} lines for {} channels",
                    pattern.len(),
                    self.channels.len()
                ));
            }
            rows.extend(pattern[channel].split_whitespace());
        }
        Ok(rows)
    }

    /// Total length in rows; every channel must agree.
    pub fn rows(&self) -> Result<usize, String> {
        let mut total = None;
        for c in 0..self.channels.len() {
            let n = self.channel_rows(c)?.len();
            if total.is_some_and(|t| t != n) {
                return Err(format!("channel {c} has {n} rows, others differ"));
            }
            total = Some(n);
        }
        total.ok_or_else(|| "song has no channels".into())
    }

    fn events(&self, channel: usize) -> Result<Vec<Event>, String> {
        let kit = &self.channels[channel].kit;
        let mut events: Vec<Event> = Vec::new();
        let mut holding = false;
        for (row, token) in self.channel_rows(channel)?.into_iter().enumerate() {
            match token {
                "-" => {
                    if holding && let Some(last) = events.last_mut() {
                        last.rows += 1;
                    }
                }
                "." => holding = false,
                drum if kit.contains_key(drum) => {
                    events.push(Event {
                        row,
                        rows: 1,
                        sound: Sound::Drum(drum.to_string()),
                    });
                    holding = false;
                }
                note => {
                    let freq = synth::note_freq(note)
                        .ok_or_else(|| format!("channel {channel}: bad row '{note}'"))?;
                    events.push(Event {
                        row,
                        rows: 1,
                        sound: Sound::Note(freq),
                    });
                    holding = true;
                }
            }
        }
        Ok(events)
    }

    /// Renders the whole song to mono samples.
    pub fn render(&self) -> Result<Vec<f32>, String> {
        let spr = self.samples_per_row();
        let row_seconds = spr as f32 / synth::SAMPLE_RATE as f32;
        let mut tones = Vec::new();
        for (c, channel) in self.channels.iter().enumerate() {
            for event in self.events(c)? {
                let tone = match &event.sound {
                    Sound::Note(freq) => {
                        tone_for(&channel.instrument, *freq, event.rows as f32 * row_seconds)
                    }
                    Sound::Drum(key) => {
                        let drum = &channel.kit[key];
                        tone_for(&drum.instrument, drum.freq, drum.gate)
                    }
                };
                tones.push((event.row * spr, tone));
            }
        }
        let mut len = self.rows()? * spr;
        if !self.looping {
            // A one-shot song lets its last notes ring out.
            len = tones
                .iter()
                .map(|(at, t)| at + t.len())
                .fold(len, usize::max);
        }
        let mut out = vec![0.0; len];
        for (at, tone) in &tones {
            tone.mix_into(&mut out, *at, self.looping);
        }
        synth::clamp(&mut out);
        Ok(out)
    }
}

fn tone_for(instrument: &Instrument, freq: f32, gate: f32) -> Tone {
    Tone {
        at: 0.0,
        wave: instrument.wave,
        freq,
        gate,
        env: instrument.env,
        slide: instrument.slide,
        vibrato: instrument.vibrato,
        volume: instrument.volume,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn song(looping: bool) -> SongDef {
        ron::from_str(&format!(
            r#"(
                bpm: 120, rows_per_beat: 4, looping: {looping},
                channels: [
                    (instrument: (wave: Square(duty: 0.5), volume: 0.3,
                                  env: (release: 0.2))),
                    (kit: {{ "k": (instrument: (wave: Noise), freq: 2000, gate: 0.05) }}),
                ],
                patterns: {{
                    "a": ["C4 - - . E4 - G4 -", "k . . . k . . ."],
                }},
                order: ["a", "a"],
            )"#
        ))
        .unwrap()
    }

    #[test]
    fn rendering_is_deterministic() {
        assert_eq!(song(true).render(), song(true).render());
    }

    #[test]
    fn looping_length_is_exactly_the_rows() {
        let s = song(true);
        // 120 bpm, 4 rows per beat: 0.125 s per row.
        assert_eq!(s.samples_per_row(), 2756);
        assert_eq!(s.rows(), Ok(16));
        assert_eq!(s.render().unwrap().len(), 16 * 2756);
    }

    #[test]
    fn one_shot_song_keeps_its_release_tail() {
        let s = song(false);
        // The last note (G4, two rows) releases over 0.2 s past the end.
        let len = s.render().unwrap().len();
        assert_eq!(
            len,
            14 * 2756 + synth::samples_for(2.0 * 2756.0 / 22050.0 + 0.2)
        );
    }

    #[test]
    fn holds_and_rests_set_note_lengths() {
        let events = song(true).events(0).unwrap();
        let lengths: Vec<(usize, usize)> = events.iter().map(|e| (e.row, e.rows)).collect();
        assert_eq!(lengths[..3], [(0, 3), (4, 2), (6, 2)]);
    }

    #[test]
    fn mismatched_channels_are_rejected() {
        let mut s = song(true);
        s.patterns
            .insert("b".into(), vec!["C4".into(), "k .".into()]);
        s.order.push("b".into());
        assert!(s.rows().is_err());
        s.patterns.insert("c".into(), vec!["C4".into()]);
        s.order = vec!["c".into()];
        assert!(s.rows().is_err());
    }

    #[test]
    fn bad_notes_are_rejected() {
        let mut s = song(true);
        s.patterns.insert(
            "a".into(),
            vec!["X9 - - - - - - -".into(), "k . . . k . . .".into()],
        );
        assert!(s.render().is_err());
    }
}
