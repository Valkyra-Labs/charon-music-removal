//! Audio playback with an instant A/B/C switch.
//!
//! The output callback owns the play position (an atomic the UI reads and
//! may overwrite to seek) and never blocks: it takes the tracks with
//! `try_lock` and plays silence on the rare miss. The processed sound is
//! not stored; it is `input - cut(t) * removed`, computed per sample, so a
//! change of removal depth is heard immediately without re-running the
//! model.

use anyhow::{anyhow, Result};
use charon_audio::AudioBuffer;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

/// What is heard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Listen {
    Original = 0,
    Result = 1,
    Removed = 2,
}

/// A span with the fraction of the music estimate that is subtracted
/// (0: none, 1: all).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cut {
    pub start: usize,
    pub end: usize,
    pub amount: f32,
}

/// The sound the player plays from.
pub struct Tracks {
    pub input: Arc<AudioBuffer>,
    /// The full music estimate inside the regions (with edge fades),
    /// silent elsewhere.
    pub removed: Option<Arc<AudioBuffer>>,
    pub cuts: Vec<Cut>,
}

impl Tracks {
    /// The processed sample at `t`, channel `ch`.
    pub fn result_at(&self, ch: usize, t: usize) -> f32 {
        let x = self.input.data[[ch, t]];
        match &self.removed {
            Some(r) => x - cut_at(&self.cuts, t) * r.data[[ch, t]],
            None => x,
        }
    }

    /// The processed audio as a buffer, for export.
    pub fn render_result(&self) -> AudioBuffer {
        let mut out = (*self.input).clone();
        if let Some(r) = &self.removed {
            for c in &self.cuts {
                if c.amount == 0.0 {
                    continue;
                }
                for ch in 0..out.channels() {
                    for t in c.start..c.end.min(out.samples()) {
                        out.data[[ch, t]] -= c.amount * r.data[[ch, t]];
                    }
                }
            }
        }
        out
    }
}

fn cut_at(cuts: &[Cut], t: usize) -> f32 {
    cuts.iter()
        .find(|c| t >= c.start && t < c.end)
        .map(|c| c.amount)
        .unwrap_or(0.0)
}

struct Shared {
    tracks: Mutex<Option<Arc<Tracks>>>,
    /// Position in source samples, as f64 bits.
    pos: AtomicU64,
    playing: AtomicBool,
    listen: AtomicU8,
}

/// The player. Dropping it stops the output stream.
pub struct Player {
    shared: Arc<Shared>,
    _stream: cpal::Stream,
}

impl Player {
    pub fn new() -> Result<Self> {
        let shared = Arc::new(Shared {
            tracks: Mutex::new(None),
            pos: AtomicU64::new(0f64.to_bits()),
            playing: AtomicBool::new(false),
            listen: AtomicU8::new(Listen::Result as u8),
        });
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow!("no audio output device"))?;
        let config = device.default_output_config()?.config();
        let out_channels = config.channels as usize;
        let out_rate = config.sample_rate as f64;
        let s = shared.clone();
        // For automated runs: compute everything, output silence.
        let mute = std::env::var_os("CHARON_MUTE").is_some();
        let stream = device.build_output_stream::<f32, _, _>(
            config,
            move |data: &mut [f32], _| {
                data.fill(0.0);
                if !s.playing.load(Ordering::Relaxed) {
                    return;
                }
                let Ok(guard) = s.tracks.try_lock() else {
                    return;
                };
                let Some(tracks) = guard.as_ref() else {
                    return;
                };
                let listen = s.listen.load(Ordering::Relaxed);
                let len = tracks.input.samples();
                let src_channels = tracks.input.channels();
                let step = tracks.input.sample_rate as f64 / out_rate;
                let mut pos = f64::from_bits(s.pos.load(Ordering::Relaxed));
                for frame in data.chunks_mut(out_channels) {
                    let t = pos as usize;
                    if t >= len {
                        s.playing.store(false, Ordering::Relaxed);
                        break;
                    }
                    for (ch, out) in frame.iter_mut().enumerate() {
                        let c = ch.min(src_channels - 1);
                        *out = match listen {
                            0 => tracks.input.data[[c, t]],
                            1 => tracks.result_at(c, t),
                            _ => match &tracks.removed {
                                Some(r) => cut_at(&tracks.cuts, t) * r.data[[c, t]],
                                None => 0.0,
                            },
                        };
                    }
                    pos += step;
                }
                s.pos.store(pos.to_bits(), Ordering::Relaxed);
                if mute {
                    data.fill(0.0);
                }
            },
            |e| log::warn!("audio output: {e}"),
            None,
        )?;
        stream.play()?;
        Ok(Self {
            shared,
            _stream: stream,
        })
    }

    pub fn set_tracks(&self, tracks: Option<Arc<Tracks>>) {
        // The UI thread may wait here briefly; the callback never does.
        if let Ok(mut t) = self.shared.tracks.lock() {
            *t = tracks;
        }
    }

    pub fn position(&self) -> usize {
        f64::from_bits(self.shared.pos.load(Ordering::Relaxed)) as usize
    }

    pub fn seek(&self, sample: usize) {
        self.shared
            .pos
            .store((sample as f64).to_bits(), Ordering::Relaxed);
    }

    pub fn playing(&self) -> bool {
        self.shared.playing.load(Ordering::Relaxed)
    }

    pub fn set_playing(&self, on: bool) {
        log::info!("playing: {on}");
        self.shared.playing.store(on, Ordering::Relaxed);
    }

    pub fn listen(&self) -> Listen {
        match self.shared.listen.load(Ordering::Relaxed) {
            0 => Listen::Original,
            1 => Listen::Result,
            _ => Listen::Removed,
        }
    }

    pub fn set_listen(&self, l: Listen) {
        self.shared.listen.store(l as u8, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::Array2;

    #[test]
    fn result_is_input_minus_the_cut_part_of_the_estimate() {
        let input = Arc::new(AudioBuffer::new(Array2::from_elem((2, 10), 1.0), 44100));
        let removed = Arc::new(AudioBuffer::new(Array2::from_elem((2, 10), 0.5), 44100));
        let tracks = Tracks {
            input,
            removed: Some(removed),
            cuts: vec![
                Cut {
                    start: 2,
                    end: 5,
                    amount: 1.0,
                },
                Cut {
                    start: 6,
                    end: 8,
                    amount: 0.5,
                },
            ],
        };
        let rendered = tracks.render_result();
        let row: Vec<f32> = rendered.data.row(0).to_vec();
        assert_eq!(
            row,
            vec![1.0, 1.0, 0.5, 0.5, 0.5, 1.0, 0.75, 0.75, 1.0, 1.0]
        );
        for t in 0..10 {
            assert_eq!(tracks.result_at(1, t), rendered.data[[1, t]]);
        }
    }
}
