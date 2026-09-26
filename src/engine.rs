//! Work that takes longer than a frame, on worker threads.
//!
//! The UI sends a job and polls events every frame; it never waits.
//! Inference runs in this process on a worker thread (decision 0011: the
//! model is small enough).

use crate::{media, pipeline};
use charon_audio::{AudioBuffer, CancelToken, Region};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Peaks per second of the waveform envelope.
pub const PEAKS_PER_SECOND: usize = 100;

pub enum Event {
    Progress {
        stage: &'static str,
        fraction: f32,
    },
    Loaded {
        path: PathBuf,
        info: media::MediaInfo,
        audio: Arc<AudioBuffer>,
        peaks: Vec<f32>,
        seconds: f64,
    },
    /// The regions are known; separation is still running.
    Detected {
        regions: Vec<Region>,
    },
    Analyzed {
        regions: Vec<Region>,
        removed: Arc<AudioBuffer>,
        removed_peaks: Vec<f32>,
        seconds: f64,
    },
    Exported {
        path: PathBuf,
        seconds: f64,
    },
    Cancelled,
    Failed(String),
}

enum Job {
    Load(PathBuf),
    Analyze(Arc<AudioBuffer>),
    Export {
        source: PathBuf,
        audio: AudioBuffer,
        destination: PathBuf,
    },
}

/// Handle to the worker.
pub struct Engine {
    jobs: Sender<(Job, CancelToken)>,
    pub events: Receiver<Event>,
    cancel: Mutex<Option<CancelToken>>,
}

/// Mono envelope: the loudest sample per 1/100 s.
pub fn peaks(audio: &AudioBuffer) -> Vec<f32> {
    let step = (audio.sample_rate as usize / PEAKS_PER_SECOND).max(1);
    let n = audio.samples().div_ceil(step);
    (0..n)
        .map(|i| {
            let a = i * step;
            let b = (a + step).min(audio.samples());
            let mut m = 0.0f32;
            for ch in 0..audio.channels() {
                for t in a..b {
                    m = m.max(audio.data[[ch, t]].abs());
                }
            }
            m
        })
        .collect()
}

impl Engine {
    pub fn start(model: PathBuf, repaint: impl Fn() + Send + Sync + 'static) -> Self {
        let (jobs_tx, jobs_rx) = mpsc::channel::<(Job, CancelToken)>();
        let (events_tx, events_rx) = mpsc::channel::<Event>();
        std::thread::Builder::new()
            .name("engine".into())
            .spawn(move || worker(model, jobs_rx, events_tx, repaint))
            .expect("spawn the engine thread");
        Self {
            jobs: jobs_tx,
            events: events_rx,
            cancel: Mutex::new(None),
        }
    }

    fn send(&self, job: Job) {
        let token = CancelToken::new();
        if let Ok(mut c) = self.cancel.lock() {
            *c = Some(token.clone());
        }
        let _ = self.jobs.send((job, token));
    }

    pub fn load(&self, path: PathBuf) {
        self.send(Job::Load(path));
    }

    pub fn analyze(&self, audio: Arc<AudioBuffer>) {
        self.send(Job::Analyze(audio));
    }

    pub fn export(&self, source: PathBuf, audio: AudioBuffer, destination: PathBuf) {
        self.send(Job::Export {
            source,
            audio,
            destination,
        });
    }

    pub fn cancel(&self) {
        if let Ok(c) = self.cancel.lock() {
            if let Some(t) = c.as_ref() {
                t.cancel();
            }
        }
    }
}

fn worker(
    model: PathBuf,
    jobs: Receiver<(Job, CancelToken)>,
    events: Sender<Event>,
    repaint: impl Fn() + Send + Sync + 'static,
) {
    let repaint = Arc::new(repaint);
    let send = |e: Event| {
        let _ = events.send(e);
        repaint();
    };
    let mut models: Option<pipeline::Models> = None;
    while let Ok((job, cancel)) = jobs.recv() {
        let t = Instant::now();
        let progress = |stage: &'static str| {
            let events = events.clone();
            let repaint = repaint.clone();
            move |fraction: f32| {
                let _ = events.send(Event::Progress { stage, fraction });
                repaint();
            }
        };
        let outcome: anyhow::Result<Event> = match job {
            Job::Load(path) => (|| {
                let info = media::probe(&path)?;
                if info.audio_tracks == 0 {
                    anyhow::bail!("this file has no audio track");
                }
                let audio = media::decode_audio(&path, &progress("Reading audio"))?;
                let peaks = peaks(&audio);
                Ok(Event::Loaded {
                    path,
                    info,
                    audio: Arc::new(audio),
                    peaks,
                    seconds: t.elapsed().as_secs_f64(),
                })
            })(),
            Job::Analyze(audio) => (|| {
                if models.is_none() {
                    progress("Loading the model")(0.0);
                    models = Some(pipeline::Models::load(&model)?);
                }
                let models = models.as_ref().expect("loaded");
                let job = |stage| pipeline::Job {
                    cancel: cancel.clone(),
                    progress: Arc::new(progress(stage)),
                };
                let detection = pipeline::detect(
                    models,
                    &audio,
                    pipeline::DetectParams::default(),
                    &job("Finding music"),
                )?;
                send(Event::Detected {
                    regions: detection.regions.clone(),
                });
                let removal =
                    pipeline::remove(models, &audio, &detection.regions, &job("Separating music"))?;
                let removed_peaks = peaks(&removal.removed);
                Ok(Event::Analyzed {
                    regions: detection.regions,
                    removed: Arc::new(removal.removed),
                    removed_peaks,
                    seconds: t.elapsed().as_secs_f64(),
                })
            })(),
            Job::Export {
                source,
                audio,
                destination,
            } => media::export(
                &source,
                &audio,
                &destination,
                &progress("Writing the video"),
            )
            .map(|()| Event::Exported {
                path: destination,
                seconds: t.elapsed().as_secs_f64(),
            }),
        };
        match outcome {
            Ok(e) => send(e),
            Err(_) if cancel.is_cancelled() => send(Event::Cancelled),
            Err(e) => send(Event::Failed(format!("{e:#}"))),
        }
    }
}

/// Preview frames on their own thread, following the audio clock.
///
/// Playing: frames are decoded one after another from a reader opened at
/// the clock and each is shown when the clock reaches its time; the reader
/// is reopened only when the clock jumps (a seek). Paused: the latest
/// requested time gets one seek-and-decode through the image generator,
/// and requests that pile up while one is in flight collapse to the last.
/// The shape comes from an earlier player of mine; the costs on a 1080p60 H.264
/// recording are in docs/measurements.md (seek 27 ms, sequential frame
/// 0.9 ms).
pub struct Preview {
    shared: Arc<PreviewShared>,
    /// The latest frame and a counter of frames published.
    pub slot: Arc<Mutex<Option<(u64, media::Frame)>>>,
    _stop: Sender<()>,
}

struct PreviewShared {
    /// The clock in seconds, as f64 bits.
    clock: AtomicU64,
    playing: AtomicBool,
    /// Bumped for each paused request.
    request: AtomicU64,
}

/// A clock jump larger than this reopens the reader.
const JUMP_SECONDS: f64 = 0.5;

impl Preview {
    pub fn start(
        path: &Path,
        size: (u32, u32),
        repaint: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        let shared = Arc::new(PreviewShared {
            clock: AtomicU64::new(0f64.to_bits()),
            playing: AtomicBool::new(false),
            request: AtomicU64::new(1),
        });
        let slot = Arc::new(Mutex::new(None));
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let (sh, s, p) = (shared.clone(), slot.clone(), path.to_path_buf());
        std::thread::Builder::new()
            .name("preview".into())
            .spawn(move || preview_loop(&p, size, &sh, &s, &stop_rx, &repaint))
            .expect("spawn the preview thread");
        Self {
            shared,
            slot,
            _stop: stop_tx,
        }
    }

    /// Follow the clock; cheap, call every frame.
    pub fn follow(&self, seconds: f64, playing: bool) {
        let prev = f64::from_bits(self.shared.clock.load(Ordering::Acquire));
        self.shared
            .clock
            .store(seconds.to_bits(), Ordering::Release);
        self.shared.playing.store(playing, Ordering::Release);
        if !playing && (prev - seconds).abs() >= 1.0 / 120.0 {
            self.shared.request.fetch_add(1, Ordering::AcqRel);
        }
    }
}

fn preview_loop(
    path: &Path,
    size: (u32, u32),
    shared: &PreviewShared,
    slot: &Mutex<Option<(u64, media::Frame)>>,
    stop: &Receiver<()>,
    repaint: &dyn Fn(),
) {
    let (w, h) = media::preview_size(size);
    let generator = media::FrameSource::new(path, w as f64, h as f64);
    let mut published = 0u64;
    let mut publish = |frame: media::Frame| {
        published += 1;
        if let Ok(mut s) = slot.lock() {
            *s = Some((published, frame));
        }
        repaint();
    };
    let mut reader: Option<media::FrameReader> = None;
    let mut pending: Option<media::TimedFrame> = None;
    // Time of the frame on screen, and whether the reader ran out.
    let mut shown = f64::NEG_INFINITY;
    let mut exhausted = false;
    let mut served = 0u64;
    let idle = std::time::Duration::from_millis(4);
    loop {
        if !matches!(stop.try_recv(), Err(mpsc::TryRecvError::Empty)) {
            break;
        }
        let clock = f64::from_bits(shared.clock.load(Ordering::Acquire));
        if !shared.playing.load(Ordering::Acquire) {
            let want = shared.request.load(Ordering::Acquire);
            if want == served {
                std::thread::sleep(idle);
                continue;
            }
            served = want;
            match generator.frame(clock) {
                Ok(frame) => {
                    publish(frame);
                    shown = clock;
                }
                Err(e) => log::debug!("preview: {e}"),
            }
            // The next play starts from the clock, not from a stale reader.
            reader = None;
            pending = None;
            exhausted = false;
            continue;
        }
        let ahead = pending.as_ref().map_or(shown, |p| p.seconds);
        let jumped = clock < shown - JUMP_SECONDS || clock > ahead + JUMP_SECONDS;
        if jumped || (reader.is_none() && !exhausted) {
            pending = None;
            exhausted = false;
            reader = match media::FrameReader::new(path, clock, w, h) {
                Ok(r) => Some(r),
                Err(e) => {
                    log::warn!("preview reader: {e:#}");
                    exhausted = true;
                    None
                }
            };
            // A jump backwards must not be held back by the old frame.
            shown = clock - JUMP_SECONDS / 2.0;
        }
        if pending.is_none() {
            if let Some(r) = reader.as_mut() {
                match r.next() {
                    Some(Ok(f)) => pending = Some(f),
                    Some(Err(e)) => log::debug!("preview: {e}"),
                    None => {
                        reader = None;
                        exhausted = true;
                    }
                }
            }
        }
        match pending.take() {
            Some(f) if f.seconds <= clock => {
                shown = f.seconds;
                publish(f.frame);
            }
            Some(f) => {
                let wait = (f.seconds - clock).clamp(0.001, 0.004);
                pending = Some(f);
                std::thread::sleep(std::time::Duration::from_secs_f64(wait));
            }
            None => std::thread::sleep(idle),
        }
    }
}
