use crate::model::Settings;
use macroquad::audio::{load_sound_from_bytes, play_sound_once, set_sound_volume, Sound};

pub struct AudioState {
    throw: Option<Sound>,
    impact: Option<Sound>,
    win: Option<Sound>,
    ui: Option<Sound>,
    volume: f32,
}

impl AudioState {
    pub async fn new(settings: &Settings) -> Self {
        let throw = load_sound_from_bytes(&make_wav_bytes(660.0, 0.08, 0.35)).await.ok();
        let impact = load_sound_from_bytes(&make_wav_bytes(120.0, 0.12, 0.45)).await.ok();
        let win = load_sound_from_bytes(&make_wav_bytes(880.0, 0.2, 0.30)).await.ok();
        let ui = load_sound_from_bytes(&make_wav_bytes(420.0, 0.05, 0.25)).await.ok();
        let mut state = Self {
            throw,
            impact,
            win,
            ui,
            volume: 0.0,
        };
        state.sync(settings);
        state
    }

    pub fn sync(&mut self, settings: &Settings) {
        self.volume = (settings.master_volume * settings.sfx_volume).clamp(0.0, 1.0);
        for sound in [&self.throw, &self.impact, &self.win, &self.ui]
            .into_iter()
            .flatten()
        {
            set_sound_volume(sound, self.volume);
        }
    }

    pub fn throw(&self) {
        self.play(self.throw.as_ref());
    }

    pub fn impact(&self) {
        self.play(self.impact.as_ref());
    }

    pub fn win(&self) {
        self.play(self.win.as_ref());
    }

    pub fn ui(&self) {
        self.play(self.ui.as_ref());
    }

    fn play(&self, sound: Option<&Sound>) {
        if let Some(sound) = sound {
            play_sound_once(sound);
        }
    }
}

fn make_wav_bytes(freq: f32, duration: f32, amplitude: f32) -> Vec<u8> {
    let sample_rate = 44_100u32;
    let sample_count = (duration * sample_rate as f32) as usize;
    let mut pcm = Vec::with_capacity(sample_count * 2);
    for i in 0..sample_count {
        let t = i as f32 / sample_rate as f32;
        let envelope = 1.0 - (i as f32 / sample_count.max(1) as f32);
        let sample = ((t * freq * std::f32::consts::TAU).sin() * amplitude * envelope * i16::MAX as f32) as i16;
        pcm.extend_from_slice(&sample.to_le_bytes());
    }

    let data_len = pcm.len() as u32;
    let mut bytes = Vec::with_capacity(44 + pcm.len());
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    bytes.extend_from_slice(&pcm);
    bytes
}
