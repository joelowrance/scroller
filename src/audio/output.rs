//! Plays rendered buffers through the sound device.
//!
//! Every playing buffer is a voice on one bus, which mixes them, applies the
//! master gain and a soft limiter (so effects over music never hard-clip),
//! and hands one mono stream to rodio.
//!
//! The device is opened lazily by [`Output::open`], which the plugin calls only
//! after the player's first input: browsers refuse to start audio before a
//! user gesture and log an autoplay warning if the game tries.

use std::num::NonZero;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use bevy::log::{info, warn};
use rodio::{ChannelCount, MixerDeviceSink, SampleRate, Source};

use super::synth::SAMPLE_RATE;

/// Mixed level above which the limiter starts to round peaks off.
const LIMIT_KNEE: f32 = 0.8;

/// Controls shared with the bus, read once per sample.
#[derive(Default)]
pub struct Master {
    gain: AtomicU32,
    paused: AtomicBool,
}

impl Master {
    pub fn set_gain(&self, gain: f32) {
        self.gain.store(gain.to_bits(), Ordering::Relaxed);
    }

    fn gain(&self) -> f32 {
        f32::from_bits(self.gain.load(Ordering::Relaxed))
    }

    pub fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::Relaxed);
    }

    fn paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }
}

/// Stops a playing voice (used for music).
#[derive(Clone, Default)]
pub struct StopHandle(Arc<AtomicBool>);

impl StopHandle {
    pub fn stop(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    fn stopped(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// How a buffer is played.
#[derive(Clone, Copy, Debug)]
pub struct Play {
    pub looping: bool,
    /// Whether gameplay pause silences it (menu sounds keep playing).
    pub pausable: bool,
    pub gain: f32,
}

/// One buffer playing once or on a loop.
struct Voice {
    samples: Arc<[f32]>,
    pos: usize,
    play: Play,
    stop: StopHandle,
}

impl Voice {
    /// The next sample, or `None` once the voice has finished.
    fn next(&mut self, paused: bool) -> Option<f32> {
        if self.stop.stopped() {
            return None;
        }
        if self.play.pausable && paused {
            return Some(0.0);
        }
        if self.pos >= self.samples.len() {
            if !self.play.looping || self.samples.is_empty() {
                return None;
            }
            self.pos = 0;
        }
        let sample = self.samples[self.pos] * self.play.gain;
        self.pos += 1;
        Some(sample)
    }
}

/// Rounds off peaks above [`LIMIT_KNEE`] so the output never passes ±1.
fn limit(x: f32) -> f32 {
    let over = x.abs() - LIMIT_KNEE;
    if over <= 0.0 {
        return x;
    }
    let room = 1.0 - LIMIT_KNEE;
    x.signum() * (LIMIT_KNEE + room * (over / room).tanh())
}

/// Mixes every voice into one endless mono stream.
struct Bus {
    incoming: Receiver<Voice>,
    voices: Vec<Voice>,
    master: Arc<Master>,
}

impl Iterator for Bus {
    type Item = rodio::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        self.voices.extend(self.incoming.try_iter());
        let paused = self.master.paused();
        let mut sum = 0.0;
        self.voices.retain_mut(|v| match v.next(paused) {
            Some(s) => {
                sum += s;
                true
            }
            None => false,
        });
        Some(limit(sum * self.master.gain()) as rodio::Sample)
    }
}

impl Source for Bus {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        NonZero::<u16>::MIN
    }

    fn sample_rate(&self) -> SampleRate {
        NonZero::new(SAMPLE_RATE).expect("sample rate is not zero")
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

#[derive(Default)]
enum Device {
    /// Waiting for the first input.
    #[default]
    Closed,
    Open {
        _sink: MixerDeviceSink,
        bus: Sender<Voice>,
    },
    /// No device (e.g. a headless machine): play nothing, quietly. This is
    /// not retried, so a device plugged in later needs a restart.
    Missing,
}

#[derive(bevy::prelude::Resource, Default)]
pub struct Output {
    device: Device,
    pub master: Arc<Master>,
}

impl Output {
    pub fn is_closed(&self) -> bool {
        matches!(self.device, Device::Closed)
    }

    /// Opens the sound device. Only the first call does anything.
    pub fn open(&mut self) {
        if !self.is_closed() {
            return;
        }
        self.device = match rodio::DeviceSinkBuilder::open_default_sink() {
            Ok(mut sink) => {
                sink.log_on_drop(false);
                let (bus, incoming) = channel();
                sink.mixer().add(Bus {
                    incoming,
                    voices: Vec::new(),
                    master: self.master.clone(),
                });
                info!("Sound device open");
                Device::Open { _sink: sink, bus }
            }
            Err(err) => {
                warn!("No sound device, playing silently: {err}");
                Device::Missing
            }
        };
    }

    /// Starts a buffer. Returns a handle to stop it early, or `None` if
    /// there is no open device.
    pub fn play(&self, samples: &Arc<[f32]>, play: Play) -> Option<StopHandle> {
        let Device::Open { bus, .. } = &self.device else {
            return None;
        };
        let stop = StopHandle::default();
        let voice = Voice {
            samples: samples.clone(),
            pos: 0,
            play,
            stop: stop.clone(),
        };
        bus.send(voice).ok().map(|_| stop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONCE: Play = Play {
        looping: false,
        pausable: true,
        gain: 1.0,
    };

    fn voice(samples: &[f32], play: Play) -> (Voice, StopHandle) {
        let stop = StopHandle::default();
        let v = Voice {
            samples: samples.into(),
            pos: 0,
            play,
            stop: stop.clone(),
        };
        (v, stop)
    }

    #[test]
    fn one_shot_ends_and_loop_repeats() {
        let (mut once, _) = voice(&[0.5, -0.5], ONCE);
        let got: Vec<_> = std::iter::from_fn(|| once.next(false)).collect();
        assert_eq!(got, [0.5, -0.5]);
        let looping = Play {
            looping: true,
            gain: 0.5,
            ..ONCE
        };
        let (mut looped, _) = voice(&[1.0, -1.0], looping);
        let got: Vec<_> = (0..5).map(|_| looped.next(false).unwrap()).collect();
        assert_eq!(got, [0.5, -0.5, 0.5, -0.5, 0.5]);
    }

    #[test]
    fn pause_holds_position_and_stop_ends() {
        let (mut v, stop) = voice(&[0.25, 0.5], ONCE);
        assert_eq!(v.next(false), Some(0.25));
        assert_eq!(v.next(true), Some(0.0));
        assert_eq!(v.next(false), Some(0.5));
        v.pos = 0;
        stop.stop();
        assert_eq!(v.next(false), None);
    }

    #[test]
    fn unpausable_voices_play_through_pause() {
        let menu = Play {
            pausable: false,
            ..ONCE
        };
        let (mut v, _) = voice(&[0.25], menu);
        assert_eq!(v.next(true), Some(0.25));
    }

    #[test]
    fn bus_mixes_applies_gain_and_drops_finished_voices() {
        let master = Arc::new(Master::default());
        master.set_gain(0.5);
        let (tx, incoming) = channel();
        let mut bus = Bus {
            incoming,
            voices: Vec::new(),
            master,
        };
        tx.send(voice(&[0.2, 0.2], ONCE).0).unwrap();
        tx.send(voice(&[0.4], ONCE).0).unwrap();
        assert!((bus.next().unwrap() - 0.3).abs() < 1e-6);
        assert!((bus.next().unwrap() - 0.1).abs() < 1e-6);
        assert_eq!(bus.voices.len(), 1);
        assert_eq!(bus.next(), Some(0.0));
        assert!(bus.voices.is_empty());
    }

    #[test]
    fn limiter_is_transparent_below_the_knee_and_never_clips() {
        assert_eq!(limit(0.5), 0.5);
        assert_eq!(limit(-0.8), -0.8);
        for x in [0.9f32, 1.0, 1.5, 3.0, 50.0] {
            let y = limit(x);
            assert!(y > LIMIT_KNEE && y <= 1.0, "{x} -> {y}");
            assert_eq!(limit(-x), -y);
        }
        assert!(limit(1.5) > limit(1.0));
    }
}
