//! Generated chip audio: sound effects and music with no audio files.
//!
//! Sound effects are tone lists in `assets/audio/sfx.ron`; songs are note
//! data in `assets/audio/music.ron`. Both are compiled in and rendered to
//! buffers once at start-up.
//!
//! - Play a sound effect: write a [`PlaySfx`] message.
//! - Music follows the screen (see [`song_for_screen`]); the Stage Card after a
//!   cleared Stage plays the [`Song::StageClear`] sting. While a Stage is
//!   playing, the song is [`StageMusic`], which the Cabinet sets (for example
//!   in `OnEnter(Screen::Playing)`); it is cleared when the Stage ends.
//! - Lost lives and extra lives make their sounds here, for every Cabinet.
//! - **M** mutes. The master volume and mute are stored with the high scores.
//! - Music and gameplay sounds stop while a Stage is paused.
//! - On the web, nothing plays until the first input (browser autoplay rules).

use std::collections::HashMap;
use std::sync::Arc;

use bevy::platform::time::Instant;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::cabinets::LifeLost;
use crate::flow::{CardReason, Paused, Screen};
use crate::input::Controls;
use crate::run::Session;
use crate::storage::{self, AUDIO_KEY};

mod music;
mod output;
mod synth;

use output::{Output, Play, StopHandle};

const SFX_RON: &str = include_str!("../../assets/audio/sfx.ron");
const MUSIC_RON: &str = include_str!("../../assets/audio/music.ron");

/// Music sits under the effects: its mix is scaled by this.
const MUSIC_GAIN: f32 = 0.6;

/// Declares the [`Sfx`] enum and its `ALL` list from one list of names.
macro_rules! sfx_list {
    ($($(#[$doc:meta])* $name:ident,)*) => {
        /// Every sound effect. Each needs an entry in `assets/audio/sfx.ron`.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
        pub enum Sfx {
            $($(#[$doc])* $name,)*
        }

        impl Sfx {
            pub const ALL: &[Sfx] = &[$(Sfx::$name,)*];
        }
    };
}

sfx_list! {
    // Everywhere.
    MenuMove,
    MenuSelect,
    /// The Hero lost a life (played here on [`LifeLost`]).
    LifeLost,
    /// The Run earned an extra life (played here when lives go up).
    ExtraLife,
    // Cabinet 1: the Wren.
    /// A volley; plays up to 10 times a second, so it is a quiet tick.
    Shot,
    /// A volley at gun level 4.
    Lance,
    /// The Undertow bomb.
    Undertow,
    /// Power-up chirps: the same rise with a different top note each.
    PowerUpGun,
    PowerUpDrone,
    PowerUpTail,
    PowerUpSpeed,
    PowerUpWard,
    PowerUpBomb,
    /// A Power-up picked up at its maximum.
    Medal,
    /// The Ward absorbed a hit.
    WardBreak,
    // Cabinet 1: enemies.
    /// A shot hit an enemy without killing it.
    EnemyHit,
    ExplosionSmall,
    ExplosionLarge,
    /// A shot glanced off a closed Clam.
    ShellPing,
    /// An Arc Gate's arc switched on.
    ArcHum,
    /// The chevron warning before a Hook.
    HookWarning,
}

impl Sfx {
    /// Menu sounds keep playing over the pause screen.
    fn pausable(self) -> bool {
        !matches!(self, Sfx::MenuMove | Sfx::MenuSelect)
    }
}

/// Every song. Each needs an entry in `assets/audio/music.ron`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum Song {
    Title,
    /// Cabinet 1, Stage 1.
    LowTide,
    /// Cabinet 1, Stage 2.
    Foundry,
    /// Cabinet 1, Stage 2's run-out up the shaft: rising sequences.
    FoundryClimb,
    /// A one-shot sting on the Stage Card after a Stage is cleared.
    StageClear,
}

impl Song {
    pub const ALL: [Song; 5] = [
        Song::Title,
        Song::LowTide,
        Song::Foundry,
        Song::FoundryClimb,
        Song::StageClear,
    ];
}

/// Plays a sound effect.
#[derive(Message, Clone, Copy, Debug)]
pub struct PlaySfx(pub Sfx);

/// The song for the current Stage, set by its Cabinet. It is used while the
/// screen is [`Screen::Playing`] and cleared when the Stage ends.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct StageMusic(pub Option<Song>);

/// The song that should be playing now, worked out every frame by
/// [`choose_music`] so no schedule ordering can overwrite a Cabinet's choice.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
struct WantedMusic(Option<Song>);

/// The song each screen outside a Stage plays. The Stage Card's depends on
/// why it is showing.
pub fn song_for_screen(screen: Screen, card: Option<CardReason>) -> Option<Song> {
    match screen {
        Screen::Title | Screen::PracticeSelect => Some(Song::Title),
        Screen::StageCard if card == Some(CardReason::StageCleared) => Some(Song::StageClear),
        _ => None,
    }
}

/// Master volume (0–1) and mute, saved across sessions.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioSettings {
    pub volume: f32,
    pub muted: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            volume: 0.8,
            muted: false,
        }
    }
}

impl AudioSettings {
    pub fn gain(&self) -> f32 {
        if self.muted {
            0.0
        } else {
            self.volume.clamp(0.0, 1.0)
        }
    }
}

/// Every rendered buffer, made once at start-up.
#[derive(Resource)]
pub struct AudioBank {
    sfx: HashMap<Sfx, Arc<[f32]>>,
    /// Each song with whether it loops.
    songs: HashMap<Song, (Arc<[f32]>, bool)>,
}

impl AudioBank {
    pub fn render() -> Result<Self, String> {
        let defs: HashMap<Sfx, Vec<synth::Tone>> =
            ron::from_str(SFX_RON).map_err(|e| format!("assets/audio/sfx.ron: {e}"))?;
        let mut sfx = HashMap::new();
        for &id in Sfx::ALL {
            let tones = defs
                .get(&id)
                .ok_or_else(|| format!("assets/audio/sfx.ron: no {id:?}"))?;
            sfx.insert(id, synth::render_tones(tones).into());
        }

        let defs: HashMap<Song, music::SongDef> =
            ron::from_str(MUSIC_RON).map_err(|e| format!("assets/audio/music.ron: {e}"))?;
        let mut songs = HashMap::new();
        for id in Song::ALL {
            let def = defs
                .get(&id)
                .ok_or_else(|| format!("assets/audio/music.ron: no {id:?}"))?;
            let samples = def
                .render()
                .map_err(|e| format!("assets/audio/music.ron: {id:?}: {e}"))?;
            songs.insert(id, (samples.into(), def.looping));
        }
        Ok(Self { sfx, songs })
    }
}

/// The song playing now and how to stop it.
#[derive(Resource, Default)]
struct NowPlaying(Option<(Song, StopHandle)>);

/// Opens the sound device on the first input. On the desktop there is no
/// autoplay rule, so it opens straight away.
fn open_on_first_input(
    mut output: ResMut<Output>,
    controls: Res<Controls>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    let input = controls.any_just_pressed()
        || keys.get_just_pressed().next().is_some()
        || mouse.get_just_pressed().next().is_some();
    if input || cfg!(not(target_arch = "wasm32")) {
        output.open();
    }
}

fn toggle_mute(keys: Res<ButtonInput<KeyCode>>, mut settings: ResMut<AudioSettings>) {
    if keys.just_pressed(KeyCode::KeyM) {
        settings.muted = !settings.muted;
        storage::store(AUDIO_KEY, &*settings);
        info!("Sound {}", if settings.muted { "muted" } else { "on" });
    }
}

fn apply_settings(settings: Res<AudioSettings>, output: Res<Output>) {
    output.master.set_gain(settings.gain());
}

fn apply_pause(paused: Res<Paused>, screen: Res<State<Screen>>, output: Res<Output>) {
    output
        .master
        .set_paused(paused.0 && *screen.get() == Screen::Playing);
}

fn choose_music(
    screen: Res<State<Screen>>,
    stage: Res<StageMusic>,
    card: Option<Res<CardReason>>,
    mut wanted: ResMut<WantedMusic>,
) {
    let song = match screen.get() {
        Screen::Playing => stage.0,
        other => song_for_screen(*other, card.map(|c| *c)),
    };
    wanted.set_if_neq(WantedMusic(song));
}

fn clear_stage_music(mut stage: ResMut<StageMusic>) {
    stage.0 = None;
}

fn play_music(
    music: Res<WantedMusic>,
    bank: Res<AudioBank>,
    output: Res<Output>,
    mut now: ResMut<NowPlaying>,
) {
    if now.0.as_ref().map(|(song, _)| *song) == music.0 {
        return;
    }
    if let Some((_, stop)) = now.0.take() {
        stop.stop();
    }
    if let Some(song) = music.0 {
        let (samples, looping) = &bank.songs[&song];
        let play = Play {
            looping: *looping,
            pausable: true,
            gain: MUSIC_GAIN,
        };
        // Without a device yet, `now` stays empty and this retries next frame.
        now.0 = output.play(samples, play).map(|stop| (song, stop));
    }
}

/// Plays each requested sound once per frame, however many times it was
/// asked for (a Hook pair, several enemies dying together).
fn play_sfx(mut requests: MessageReader<PlaySfx>, bank: Res<AudioBank>, output: Res<Output>) {
    let mut seen = Vec::new();
    for PlaySfx(sfx) in requests.read() {
        if seen.contains(sfx) {
            continue;
        }
        seen.push(*sfx);
        let play = Play {
            looping: false,
            pausable: sfx.pausable(),
            gain: 1.0,
        };
        output.play(&bank.sfx[sfx], play);
    }
}

/// Lost and extra lives sound the same in every Cabinet, so they are played
/// here rather than by each Cabinet. Lives are only watched during a Stage,
/// so a new Run's fresh lives don't sound like an extra life.
fn life_sounds(
    mut lost: MessageReader<LifeLost>,
    screen: Res<State<Screen>>,
    session: Option<Res<Session>>,
    mut last_lives: Local<Option<u32>>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    if lost.read().count() > 0 {
        sfx.write(PlaySfx(Sfx::LifeLost));
    }
    let playing = *screen.get() == Screen::Playing;
    let lives = session.filter(|_| playing).map(|s| s.lives);
    if let (Some(before), Some(now)) = (*last_lives, lives)
        && now > before
    {
        sfx.write(PlaySfx(Sfx::ExtraLife));
    }
    *last_lives = lives;
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        let started = Instant::now();
        let bank = AudioBank::render().expect("embedded audio data must be valid");
        info!("Generated audio in {} ms", started.elapsed().as_millis());

        let settings = storage::load::<AudioSettings>(AUDIO_KEY).unwrap_or_default();
        app.insert_resource(bank)
            .insert_resource(settings)
            .init_resource::<Output>()
            .init_resource::<StageMusic>()
            .init_resource::<WantedMusic>()
            .init_resource::<NowPlaying>()
            .add_message::<PlaySfx>()
            .add_systems(OnExit(Screen::Playing), clear_stage_music)
            .add_systems(Update, life_sounds.before(play_sfx))
            .add_systems(
                Update,
                (
                    open_on_first_input.run_if(|o: Res<Output>| o.is_closed()),
                    toggle_mute,
                    apply_settings.run_if(resource_changed::<AudioSettings>),
                    apply_pause,
                    choose_music,
                    play_music,
                    play_sfx,
                )
                    .chain(),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_audio_renders() {
        let bank = AudioBank::render().unwrap();
        for id in Sfx::ALL {
            let len = bank.sfx[id].len();
            assert!(len > 0 && len < synth::SAMPLE_RATE as usize * 2, "{id:?}");
        }
        for id in Song::ALL {
            assert!(!bank.songs[&id].0.is_empty(), "{id:?}");
        }
    }

    #[test]
    fn rendering_the_bank_is_deterministic() {
        let a = AudioBank::render().unwrap();
        let b = AudioBank::render().unwrap();
        assert_eq!(a.sfx, b.sfx);
        assert_eq!(a.songs, b.songs);
    }

    #[test]
    fn songs_loop_on_whole_rows_and_the_sting_does_not_loop() {
        let defs: HashMap<Song, music::SongDef> = ron::from_str(MUSIC_RON).unwrap();
        let bank = AudioBank::render().unwrap();
        for id in Song::ALL {
            let def = &defs[&id];
            let (samples, looping) = &bank.songs[&id];
            assert_eq!(*looping, id != Song::StageClear, "{id:?}");
            if *looping {
                let rows = def.rows().unwrap() * def.samples_per_row();
                assert_eq!(samples.len(), rows, "{id:?}");
            }
        }
        let sting = bank.songs[&Song::StageClear].0.len() as f32 / synth::SAMPLE_RATE as f32;
        assert!((1.0..4.0).contains(&sting), "sting lasts {sting} s");
    }

    #[test]
    fn stage_tracks_are_near_their_design_tempo() {
        let defs: HashMap<Song, music::SongDef> = ron::from_str(MUSIC_RON).unwrap();
        // docs/cabinets/1-side-shooter.md: about 150 and 160 BPM.
        assert_eq!(defs[&Song::LowTide].bpm, 150.0);
        assert_eq!(defs[&Song::Foundry].bpm, 160.0);
        assert_eq!(defs[&Song::FoundryClimb].bpm, 160.0);
    }

    /// Writes every sound to `target/audio-preview/*.wav` for listening:
    /// `cargo test export_wavs -- --ignored`.
    #[test]
    #[ignore]
    fn export_wavs() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/audio-preview");
        std::fs::create_dir_all(&dir).unwrap();
        let bank = AudioBank::render().unwrap();
        let named = bank.sfx.iter().map(|(id, s)| (format!("{id:?}"), s)).chain(
            bank.songs
                .iter()
                .map(|(id, (s, _))| (format!("song-{id:?}"), s)),
        );
        for (name, samples) in named {
            let clipped = samples.iter().filter(|s| s.abs() >= 1.0).count();
            let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
            println!(
                "{name}: {} samples, peak {peak:.2}, clipped {clipped}",
                samples.len()
            );
            std::fs::write(dir.join(format!("{name}.wav")), wav(samples)).unwrap();
        }
    }

    /// 16-bit mono PCM WAV.
    fn wav(samples: &[f32]) -> Vec<u8> {
        let data_len = samples.len() as u32 * 2;
        let mut out = Vec::with_capacity(44 + data_len as usize);
        out.extend(b"RIFF");
        out.extend((36 + data_len).to_le_bytes());
        out.extend(b"WAVEfmt ");
        out.extend(16u32.to_le_bytes());
        out.extend(1u16.to_le_bytes()); // PCM
        out.extend(1u16.to_le_bytes()); // mono
        out.extend(synth::SAMPLE_RATE.to_le_bytes());
        out.extend((synth::SAMPLE_RATE * 2).to_le_bytes());
        out.extend(2u16.to_le_bytes());
        out.extend(16u16.to_le_bytes());
        out.extend(b"data");
        out.extend(data_len.to_le_bytes());
        for s in samples {
            out.extend(((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16).to_le_bytes());
        }
        out
    }

    /// An app with just the screens and the music choice.
    fn music_app() -> App {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<Screen>()
            .init_resource::<StageMusic>()
            .init_resource::<WantedMusic>()
            .add_systems(OnExit(Screen::Playing), clear_stage_music)
            .add_systems(Update, choose_music);
        app
    }

    fn wanted(app: &App) -> Option<Song> {
        app.world().resource::<WantedMusic>().0
    }

    fn go(app: &mut App, screen: Screen) {
        app.world_mut()
            .resource_mut::<NextState<Screen>>()
            .set(screen);
        app.update();
    }

    #[test]
    fn music_follows_the_screen() {
        let mut app = music_app();
        app.update();
        assert_eq!(wanted(&app), Some(Song::Title));
        go(&mut app, Screen::StageCard);
        assert_eq!(wanted(&app), None);
        go(&mut app, Screen::Playing);
        assert_eq!(wanted(&app), None);
    }

    #[test]
    fn a_cabinet_sets_stage_music_on_enter() {
        let mut app = music_app();
        app.add_systems(OnEnter(Screen::Playing), |mut stage: ResMut<StageMusic>| {
            stage.0 = Some(Song::Title)
        });
        app.update();
        go(&mut app, Screen::StageCard);
        go(&mut app, Screen::Playing);
        assert_eq!(wanted(&app), Some(Song::Title));
        // It survives later frames and is cleared when the Stage ends.
        app.update();
        assert_eq!(wanted(&app), Some(Song::Title));
        go(&mut app, Screen::StageCard);
        assert_eq!(wanted(&app), None);
        assert_eq!(app.world().resource::<StageMusic>().0, None);
    }

    #[test]
    fn the_stage_card_stings_only_after_a_clear() {
        let mut app = music_app();
        app.update();
        app.insert_resource(CardReason::NewRun);
        go(&mut app, Screen::StageCard);
        assert_eq!(wanted(&app), None);
        go(&mut app, Screen::Playing);
        app.insert_resource(CardReason::StageCleared);
        go(&mut app, Screen::StageCard);
        assert_eq!(wanted(&app), Some(Song::StageClear));
    }

    fn life_app() -> App {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<Screen>()
            .add_message::<LifeLost>()
            .add_message::<PlaySfx>()
            .add_systems(Update, life_sounds);
        app
    }

    fn played(app: &mut App) -> Vec<Sfx> {
        let mut messages = app.world_mut().resource_mut::<Messages<PlaySfx>>();
        messages.drain().map(|PlaySfx(s)| s).collect()
    }

    #[test]
    fn lives_lost_and_gained_make_their_sounds() {
        let mut app = life_app();
        let tuning = crate::tuning::Tuning::default();
        app.insert_resource(Session::new_run(&tuning));
        go(&mut app, Screen::Playing);
        assert_eq!(played(&mut app), []);

        app.world_mut().write_message(LifeLost);
        app.update();
        assert_eq!(played(&mut app), [Sfx::LifeLost]);

        app.world_mut().resource_mut::<Session>().lives += 1;
        app.update();
        assert_eq!(played(&mut app), [Sfx::ExtraLife]);
        app.update();
        assert_eq!(played(&mut app), []);

        // Game Over, then a new Run with fresh lives: not an extra life.
        app.world_mut().resource_mut::<Session>().lives = 0;
        go(&mut app, Screen::GameOver);
        app.insert_resource(Session::new_run(&tuning));
        go(&mut app, Screen::StageCard);
        go(&mut app, Screen::Playing);
        assert_eq!(played(&mut app), []);
    }

    #[test]
    fn only_menu_sounds_play_over_the_pause_menu() {
        for sfx in Sfx::ALL {
            let menu = matches!(sfx, Sfx::MenuMove | Sfx::MenuSelect);
            assert_eq!(sfx.pausable(), !menu, "{sfx:?}");
        }
    }

    #[test]
    fn mute_silences_and_volume_is_clamped() {
        let muted = AudioSettings {
            muted: true,
            ..default()
        };
        assert_eq!(muted.gain(), 0.0);
        let loud = AudioSettings {
            volume: 3.0,
            muted: false,
        };
        assert_eq!(loud.gain(), 1.0);
    }
}
