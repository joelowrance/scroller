//! The chip synth: renders one [`Tone`] to mono samples.
//!
//! Pure and deterministic, so the same tone always gives the same samples
//! (noise comes from a fixed-seed shift register, not a random generator).

use serde::Deserialize;

/// Output rate of every rendered buffer. Low, like the arcade chips it imitates,
/// which also keeps start-up rendering cheap.
pub const SAMPLE_RATE: u32 = 22_050;

/// Levels of the triangle wave, which is stepped like the classic 4-bit one.
const TRIANGLE_STEPS: f32 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub enum Wave {
    /// Pulse wave; `duty` is the high fraction of each cycle (0.125, 0.25, 0.5…).
    Square {
        duty: f32,
    },
    Triangle,
    /// Shift-register noise, clocked at the tone's frequency.
    Noise,
}

/// Attack, decay, sustain and release, in seconds (sustain is a level 0–1).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(default)]
pub struct Envelope {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            attack: 0.0,
            decay: 0.0,
            sustain: 1.0,
            release: 0.02,
        }
    }
}

impl Envelope {
    /// Level while the note is held, `t` seconds after it starts.
    fn held(&self, t: f32) -> f32 {
        if t < self.attack {
            t / self.attack
        } else if t < self.attack + self.decay {
            let k = (t - self.attack) / self.decay;
            1.0 + (self.sustain - 1.0) * k
        } else {
            self.sustain
        }
    }

    /// Level `t` seconds into a note whose key is released at `gate`.
    pub fn level(&self, t: f32, gate: f32) -> f32 {
        if t < gate {
            self.held(t)
        } else if self.release <= 0.0 {
            0.0
        } else {
            let k = ((t - gate) / self.release).min(1.0);
            self.held(gate) * (1.0 - k)
        }
    }
}

/// Pitch wobble: `depth` in semitones, `rate` in Hz.
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize)]
pub struct Vibrato {
    pub depth: f32,
    pub rate: f32,
}

/// One synthesized note: everything needed to render it.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(default)]
pub struct Tone {
    /// Start time within a sound effect, in seconds (ignored by the sequencer).
    pub at: f32,
    pub wave: Wave,
    /// Starting pitch in Hz.
    pub freq: f32,
    /// How long the key is held, in seconds; the release follows.
    pub gate: f32,
    pub env: Envelope,
    /// Pitch change across the gate, in semitones (negative falls).
    pub slide: f32,
    pub vibrato: Vibrato,
    pub volume: f32,
}

impl Default for Tone {
    fn default() -> Self {
        Self {
            at: 0.0,
            wave: Wave::Square { duty: 0.5 },
            freq: 440.0,
            gate: 0.1,
            env: Envelope::default(),
            slide: 0.0,
            vibrato: Vibrato::default(),
            volume: 0.5,
        }
    }
}

/// Number of samples covering `seconds`, rounded up.
pub fn samples_for(seconds: f32) -> usize {
    (seconds.max(0.0) * SAMPLE_RATE as f32).ceil() as usize
}

impl Tone {
    /// Rendered length in samples: the gate plus the release tail.
    pub fn len(&self) -> usize {
        samples_for(self.gate + self.env.release.max(0.0))
    }

    /// Renders the tone and adds it into `out` starting at sample `start`.
    /// With `wrap`, samples past the end continue from the start (for loops);
    /// otherwise they are dropped.
    pub fn mix_into(&self, out: &mut [f32], start: usize, wrap: bool) {
        if out.is_empty() {
            return;
        }
        let rate = SAMPLE_RATE as f32;
        let mut phase = 0.0f32;
        let mut lfsr: u16 = 1;
        let mut noise = 1.0f32;
        for i in 0..self.len() {
            let mut idx = start + i;
            if idx >= out.len() {
                if !wrap {
                    break;
                }
                idx %= out.len();
            }
            let t = i as f32 / rate;
            let slide = self.slide * (t / self.gate.max(1e-6)).min(1.0);
            let wobble = self.vibrato.depth * (std::f32::consts::TAU * self.vibrato.rate * t).sin();
            let freq = self.freq * 2f32.powf((slide + wobble) / 12.0);

            let sample = match self.wave {
                Wave::Square { duty } => {
                    if phase < duty {
                        1.0
                    } else {
                        -1.0
                    }
                }
                Wave::Triangle => {
                    let tri = 1.0 - 4.0 * (phase - 0.5).abs();
                    (tri * TRIANGLE_STEPS / 2.0).round() / (TRIANGLE_STEPS / 2.0)
                }
                Wave::Noise => noise,
            };
            out[idx] += sample * self.env.level(t, self.gate) * self.volume;

            phase += freq / rate;
            if phase >= 1.0 {
                phase -= phase.floor();
                // Clock the 15-bit noise register once per cycle.
                let bit = (lfsr ^ (lfsr >> 1)) & 1;
                lfsr = (lfsr >> 1) | (bit << 14);
                noise = if lfsr & 1 == 1 { 1.0 } else { -1.0 };
            }
        }
    }
}

/// Renders a sound effect made of one or more tones, each starting at its `at`.
pub fn render_tones(tones: &[Tone]) -> Vec<f32> {
    let len = tones
        .iter()
        .map(|t| samples_for(t.at) + t.len())
        .max()
        .unwrap_or(0);
    let mut out = vec![0.0; len];
    for tone in tones {
        tone.mix_into(&mut out, samples_for(tone.at), false);
    }
    normalize(&mut out);
    out
}

/// The loudest a rendered buffer may be. Leaves headroom below 1.0 so no
/// buffer is ever flat-topped; the output bus limits the final mix.
pub const MAX_PEAK: f32 = 0.95;

/// Scales a buffer down, if needed, so its peak is at most [`MAX_PEAK`].
/// Quieter buffers are left as they are.
pub fn normalize(samples: &mut [f32]) {
    let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if peak > MAX_PEAK {
        let k = MAX_PEAK / peak;
        for s in samples {
            *s *= k;
        }
    }
}

/// Frequency in Hz of a note name such as `C4`, `F#3` or `Bb5` (A4 = 440 Hz).
pub fn note_freq(name: &str) -> Option<f32> {
    let mut chars = name.chars();
    let letter = chars.next()?;
    let base = match letter.to_ascii_uppercase() {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    let rest: &str = chars.as_str();
    let (accidental, octave) = match rest.chars().next()? {
        '#' => (1, &rest[1..]),
        'b' => (-1, &rest[1..]),
        _ => (0, rest),
    };
    let octave: i32 = octave.parse().ok()?;
    let midi = 12 * (octave + 1) + base + accidental;
    Some(440.0 * 2f32.powf((midi - 69) as f32 / 12.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blip() -> Tone {
        Tone {
            wave: Wave::Square { duty: 0.25 },
            freq: 880.0,
            gate: 0.1,
            env: Envelope {
                attack: 0.01,
                decay: 0.02,
                sustain: 0.5,
                release: 0.05,
            },
            slide: -5.0,
            vibrato: Vibrato {
                depth: 0.3,
                rate: 8.0,
            },
            ..Tone::default()
        }
    }

    #[test]
    fn rendering_is_deterministic() {
        for wave in [Wave::Square { duty: 0.125 }, Wave::Triangle, Wave::Noise] {
            let tone = Tone { wave, ..blip() };
            assert_eq!(render_tones(&[tone]), render_tones(&[tone]));
        }
    }

    #[test]
    fn length_is_gate_plus_release() {
        let tone = blip();
        // 0.15 s at 22 050 Hz.
        assert_eq!(render_tones(&[tone]).len(), 3308);
        let later = Tone { at: 0.5, ..tone };
        assert_eq!(render_tones(&[tone, later]).len(), 11025 + 3308);
    }

    #[test]
    fn envelope_starts_and_ends_silent() {
        let samples = render_tones(&[blip()]);
        assert_eq!(samples[0], 0.0);
        assert!(samples.last().unwrap().abs() < 0.01);
        assert!(samples.iter().any(|s| s.abs() > 0.2));
        assert!(samples.iter().all(|s| s.abs() <= 1.0));
    }

    #[test]
    fn noise_is_not_silent_or_constant() {
        let tone = Tone {
            wave: Wave::Noise,
            freq: 4000.0,
            ..blip()
        };
        let samples = render_tones(&[tone]);
        assert!(samples.iter().any(|s| *s > 0.1));
        assert!(samples.iter().any(|s| *s < -0.1));
    }

    #[test]
    fn loud_buffers_are_scaled_not_clipped() {
        let mut loud = vec![0.5, -1.9, 1.2];
        normalize(&mut loud);
        assert_eq!(loud[1], -MAX_PEAK);
        assert!((loud[0] / loud[2] - 0.5 / 1.2).abs() < 1e-6, "shape kept");
        let mut quiet = vec![0.2, -0.4];
        normalize(&mut quiet);
        assert_eq!(quiet, [0.2, -0.4]);
    }

    #[test]
    fn note_names() {
        assert_eq!(note_freq("A4"), Some(440.0));
        assert!((note_freq("C4").unwrap() - 261.63).abs() < 0.01);
        assert_eq!(note_freq("C#4"), note_freq("Db4"));
        assert_eq!(note_freq("A3"), Some(220.0));
        assert_eq!(note_freq("H4"), None);
        assert_eq!(note_freq("C"), None);
    }

    #[test]
    fn wrapping_tail_lands_at_the_start() {
        let mut out = vec![0.0; 1000];
        let tone = Tone {
            gate: 0.05,
            ..blip()
        };
        tone.mix_into(&mut out, 900, true);
        assert!(out[..100].iter().any(|s| *s != 0.0));
    }
}
