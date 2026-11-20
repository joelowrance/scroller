//! Plays rendered buffers through the sound device (rodio's mixer).
//!
//! The device is opened lazily by [`Output::open`], which the plugin calls only
//! after the player's first input: browsers refuse to start audio before a
//! user gesture and log an autoplay warning if the game tries.

use std::num::NonZero;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

use bevy::log::{info, warn};
use rodio::{ChannelCount, MixerDeviceSink, SampleRate, Source};

use super::synth::SAMPLE_RATE;

/// Controls shared with every playing voice, read once per sample.
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
}

/// Stops a playing voice (used for music).
#[derive(Clone, Default)]
pub struct StopHandle(Arc<AtomicBool>);

impl StopHandle {
    pub fn stop(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

/// One buffer playing once or on a loop.
struct Voice {
    samples: Arc<[f32]>,
    pos: usize,
    looping: bool,
    /// Whether gameplay pause silences this voice (menu sounds keep playing).
    pausable: bool,
    master: Arc<Master>,
    stop: StopHandle,
}

impl Iterator for Voice {
    type Item = rodio::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stop.0.load(Ordering::Relaxed) {
            return None;
        }
        if self.pausable && self.master.paused.load(Ordering::Relaxed) {
            return Some(0.0);
        }
        if self.pos >= self.samples.len() {
            if !self.looping || self.samples.is_empty() {
                return None;
            }
            self.pos = 0;
        }
        let sample = self.samples[self.pos] * self.master.gain();
        self.pos += 1;
        Some(sample as rodio::Sample)
    }
}

impl Source for Voice {
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
    Open(MixerDeviceSink),
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
                info!("Sound device open");
                Device::Open(sink)
            }
            Err(err) => {
                warn!("No sound device, playing silently: {err}");
                Device::Missing
            }
        };
    }

    /// Starts a buffer. Returns a handle to stop it early, or `None` if
    /// there is no open device.
    pub fn play(&self, samples: &Arc<[f32]>, looping: bool, pausable: bool) -> Option<StopHandle> {
        let Device::Open(sink) = &self.device else {
            return None;
        };
        let stop = StopHandle::default();
        sink.mixer().add(Voice {
            samples: samples.clone(),
            pos: 0,
            looping,
            pausable,
            master: self.master.clone(),
            stop: stop.clone(),
        });
        Some(stop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn voice(samples: &[f32], looping: bool, master: &Arc<Master>) -> Voice {
        Voice {
            samples: samples.into(),
            pos: 0,
            looping,
            pausable: true,
            master: master.clone(),
            stop: StopHandle::default(),
        }
    }

    #[test]
    fn one_shot_ends_and_loop_repeats() {
        let master = Arc::new(Master::default());
        master.set_gain(0.5);
        let once: Vec<_> = voice(&[1.0, -1.0], false, &master).collect();
        assert_eq!(once, [0.5, -0.5]);
        let looped: Vec<_> = voice(&[1.0, -1.0], true, &master).take(5).collect();
        assert_eq!(looped, [0.5, -0.5, 0.5, -0.5, 0.5]);
    }

    #[test]
    fn pause_holds_position_and_stop_ends() {
        let master = Arc::new(Master::default());
        master.set_gain(1.0);
        let mut v = voice(&[1.0, 2.0], false, &master);
        assert_eq!(v.next(), Some(1.0));
        master.set_paused(true);
        assert_eq!(v.next(), Some(0.0));
        master.set_paused(false);
        assert_eq!(v.next(), Some(2.0));
        let stop = v.stop.clone();
        v.pos = 0;
        stop.stop();
        assert_eq!(v.next(), None);
    }
}
