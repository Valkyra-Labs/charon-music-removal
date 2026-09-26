//! Music detection and removal on decoded audio, on top of charon-audio.

use anyhow::{anyhow, Result};
use charon_audio::models::TIGER_SEGMENT_SAMPLES;
use charon_audio::{
    AudioBuffer, BufferSink, BufferSource, CancelToken, Control, Region, RegionPlan, Separator,
    SeparatorConfig,
};
use ndarray::{s, Array2, Axis};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Frame length for loudness decisions, in seconds.
pub const FRAME_SECONDS: f64 = 0.5;

/// Tuning of the music detector.
///
/// Levels are medians over a sliding window, not single frames: music is
/// steady and speech is not, so a quiet score under dialogue keeps a
/// steady median share (-22 dB on the Hogwarts Legacy recording) while
/// what the model lets through from speech alone stays far lower (at
/// most -40.8 dB smoothed on the RE8 recording with music off, -52 dB on
/// the synthetic set). Per-frame thresholds of -45 dBFS and -18 dB missed
/// that score and half of a song at -15 dB under speech
/// (docs/measurements.md).
#[derive(Debug, Clone, Copy)]
pub struct DetectParams {
    /// Music counts as present when the music stem's median level is
    /// above this (dBFS)...
    pub min_level_db: f32,
    /// ...and its median share of the mix is above this (dB).
    pub min_share_db: f32,
    /// Half-width of the median window (seconds).
    pub smoothing: f64,
    /// Shorter music runs are ignored (seconds).
    pub min_region: f64,
    /// Gaps shorter than this between music runs are bridged (seconds).
    pub bridge: f64,
}

impl Default for DetectParams {
    fn default() -> Self {
        Self {
            min_level_db: -65.0,
            min_share_db: -35.0,
            smoothing: 2.5,
            min_region: 2.0,
            bridge: 2.0,
        }
    }
}

/// What detection found.
#[derive(Debug, Clone)]
pub struct Detection {
    /// Music stem level per frame (dBFS), for the timeline.
    pub music_db: Vec<f32>,
    /// Mix level per frame (dBFS).
    pub mix_db: Vec<f32>,
    /// Detected regions in samples.
    pub regions: Vec<Region>,
}

fn level_db(x: &[f32]) -> f32 {
    let e = x.iter().map(|v| v * v).sum::<f32>() / x.len().max(1) as f32;
    10.0 * (e + 1e-12).log10()
}

/// How many model sessions run at once and with how many threads each.
///
/// One session does not get faster past about 6-7 intra-op threads on
/// this graph (12-core M4 Pro: 144 s of audio in 53.0 s on 7 threads,
/// 47.7 s on 12), and two sessions of 6 run 1.58x faster than one of 12;
/// three are no faster than two and four run out of memory bandwidth
/// (charon-audio docs/MEASUREMENTS.md).
/// A session holds about 3.4 GB at its peak, so two need a machine with
/// 16 GB. `CHARON_SESSIONS` overrides the count for measurements.
pub fn session_plan() -> (usize, usize) {
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let memory = objc2_foundation::NSProcessInfo::processInfo().physicalMemory();
    let auto = if memory >= 16 << 30 && cores >= 8 {
        2
    } else {
        1
    };
    let sessions = std::env::var("CHARON_SESSIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&n: &usize| (1..=4).contains(&n))
        .unwrap_or(auto);
    (sessions, (cores / sessions).max(1))
}

/// Overlap of the removal windows: 0.25 measured equal in quality to 0.5
/// on the synthetic set and 18% faster. `CHARON_REMOVE_OVERLAP` overrides it
/// for measurements.
fn remove_overlap() -> f32 {
    std::env::var("CHARON_REMOVE_OVERLAP")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|v: &f32| (0.0..1.0).contains(v))
        .unwrap_or(0.25)
}

/// The loaded model sessions: each detects without overlap (one pass,
/// cheapest) and removes with overlapping windows on the same session.
pub struct Models {
    detect: Vec<Separator>,
    remove: Vec<Separator>,
}

impl Models {
    pub fn load(model: &Path) -> Result<Self> {
        let (sessions, threads) = session_plan();
        log::info!("{sessions} model session(s), {threads} threads each");
        let mut detect = Vec::new();
        let mut remove = Vec::new();
        for _ in 0..sessions {
            let mut config = SeparatorConfig::tiger_music(model)
                .with_overlap(0.0)
                .with_progress(false);
            config.model.onnx.intra_threads = Some(threads);
            let separator =
                Separator::new(config).map_err(|e| anyhow!("loading the model: {e}"))?;
            let mut process = separator.process_config().clone();
            process.overlap = remove_overlap();
            remove.push(separator.with_process_config(process));
            detect.push(separator);
        }
        Ok(Self { detect, remove })
    }
}

/// Cancellation and progress (0 to 1) for one job that runs in parts on
/// several threads.
#[derive(Clone)]
pub struct Job {
    pub cancel: CancelToken,
    pub progress: Arc<dyn Fn(f32) + Send + Sync>,
}

impl Job {
    pub fn quiet() -> Self {
        Self {
            cancel: CancelToken::new(),
            progress: Arc::new(|_| {}),
        }
    }

    /// One control per part; the job's progress is the sum over parts.
    fn parts(&self, n: usize) -> Vec<Control> {
        let counts: Arc<Vec<(AtomicUsize, AtomicUsize)>> = Arc::new(
            (0..n)
                .map(|_| (AtomicUsize::new(0), AtomicUsize::new(0)))
                .collect(),
        );
        (0..n)
            .map(|i| {
                let counts = counts.clone();
                let progress = self.progress.clone();
                Control::new()
                    .with_cancel(self.cancel.clone())
                    .with_progress(move |p| {
                        counts[i].0.store(p.done, Ordering::Relaxed);
                        counts[i].1.store(p.total, Ordering::Relaxed);
                        let done: usize = counts.iter().map(|c| c.0.load(Ordering::Relaxed)).sum();
                        let total: usize = counts.iter().map(|c| c.1.load(Ordering::Relaxed)).sum();
                        progress(done as f32 / total.max(1) as f32);
                    })
            })
            .collect()
    }
}

/// Run `work` on each session with its own part of the job, at once.
fn on_sessions<T: Send>(
    separators: &[Separator],
    job: &Job,
    work: impl Fn(usize, &Separator, &Control) -> Result<T> + Sync,
) -> Result<Vec<T>> {
    let controls = job.parts(separators.len());
    std::thread::scope(|scope| {
        let handles: Vec<_> = separators
            .iter()
            .zip(&controls)
            .enumerate()
            .map(|(i, (sep, control))| {
                let work = &work;
                scope.spawn(move || work(i, sep, control))
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().map_err(|_| anyhow!("a model thread panicked"))?)
            .collect()
    })
}

/// Find the music regions: separate the mono downmix once and threshold
/// the music stem's frame level. With several sessions the downmix is
/// split at window boundaries; without overlap the windows, and so the
/// estimate, are the same as in one pass.
pub fn detect(
    models: &Models,
    audio: &AudioBuffer,
    params: DetectParams,
    job: &Job,
) -> Result<Detection> {
    let mono = audio.to_mono();
    let mono = AudioBuffer::new(mono, audio.sample_rate);
    let len = mono.samples();
    let windows = len.div_ceil(TIGER_SEGMENT_SAMPLES).max(1);
    let n = models.detect.len().min(windows);
    let per = windows.div_ceil(n) * TIGER_SEGMENT_SAMPLES;
    let parts = on_sessions(&models.detect[..n], job, |i, sep, control| {
        let (a, b) = ((i * per).min(len), ((i + 1) * per).min(len));
        if a == b {
            return Ok(Array2::zeros((1, 0)));
        }
        let chunk = AudioBuffer::new(mono.data.slice(s![.., a..b]).to_owned(), mono.sample_rate);
        let mut sink = BufferSink::new();
        sep.separate_stream(&mut BufferSource::new(&chunk), &mut sink, control)
            .map_err(|e| anyhow!("detection: {e}"))?;
        Ok(sink
            .into_stems()
            .into_iter()
            .find(|(name, _)| name == "music")
            .ok_or_else(|| anyhow!("the model has no music stem"))?
            .1
            .data)
    })?;
    let views: Vec<_> = parts.iter().map(|p| p.view()).collect();
    let music = AudioBuffer::new(ndarray::concatenate(Axis(1), &views)?, mono.sample_rate);
    let frame = (FRAME_SECONDS * audio.sample_rate as f64) as usize;
    let mix_row = mono.data.row(0);
    let music_row = music.data.row(0);
    let n = mix_row.len().div_ceil(frame);
    let mut music_db = Vec::with_capacity(n);
    let mut mix_db = Vec::with_capacity(n);
    for i in 0..n {
        let a = i * frame;
        let b = ((i + 1) * frame).min(mix_row.len());
        let m = level_db(&music_row.as_slice().expect("contiguous")[a..b]);
        let x = level_db(&mix_row.as_slice().expect("contiguous")[a..b]);
        music_db.push(m);
        mix_db.push(x);
    }
    let share: Vec<f32> = music_db.iter().zip(&mix_db).map(|(m, x)| m - x).collect();
    let half = (params.smoothing / FRAME_SECONDS).round() as usize;
    let level = medians(&music_db, half);
    let share = medians(&share, half);
    let active: Vec<bool> = level
        .iter()
        .zip(&share)
        .map(|(&l, &s)| l > params.min_level_db && s > params.min_share_db)
        .collect();
    let regions = runs_to_regions(&active, frame, mono.data.ncols(), params);
    Ok(Detection {
        music_db,
        mix_db,
        regions,
    })
}

/// Sliding median over `x[i - half ..= i + half]`, clipped at the ends.
fn medians(x: &[f32], half: usize) -> Vec<f32> {
    (0..x.len())
        .map(|i| {
            let mut w = x[i.saturating_sub(half)..(i + half + 1).min(x.len())].to_vec();
            w.sort_by(f32::total_cmp);
            w[w.len() / 2]
        })
        .collect()
}

fn runs_to_regions(active: &[bool], frame: usize, len: usize, params: DetectParams) -> Vec<Region> {
    let mut runs: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < active.len() {
        if active[i] {
            let start = i;
            while i < active.len() && active[i] {
                i += 1;
            }
            runs.push((start, i));
        } else {
            i += 1;
        }
    }
    let bridge = (params.bridge / FRAME_SECONDS).round() as usize;
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for run in runs {
        match merged.last_mut() {
            Some(last) if run.0 - last.1 <= bridge => last.1 = run.1,
            _ => merged.push(run),
        }
    }
    let min_frames = (params.min_region / FRAME_SECONDS).ceil() as usize;
    merged
        .into_iter()
        .filter(|(a, b)| b - a >= min_frames)
        .map(|(a, b)| Region {
            start: (a * frame).min(len),
            end: (b * frame).min(len),
            keep: 0.0,
        })
        .filter(|r| r.end > r.start)
        .collect()
}

/// The processed audio and the part that was removed.
pub struct Removal {
    pub result: AudioBuffer,
    pub removed: AudioBuffer,
}

/// Remove music inside `regions` from every channel. With several
/// sessions the channels are shared out between them; the model runs on
/// each channel alone, so the result is the same as in one pass.
pub fn remove(
    models: &Models,
    audio: &AudioBuffer,
    regions: &[Region],
    job: &Job,
) -> Result<Removal> {
    let rate = audio.sample_rate;
    if regions.is_empty() {
        return Ok(Removal {
            result: audio.clone(),
            removed: AudioBuffer::new(Array2::zeros(audio.data.dim()), rate),
        });
    }
    let plan = RegionPlan {
        target: "music".to_string(),
        regions: regions.to_vec(),
        context: (3.0 * rate as f64) as usize,
        crossfade: (0.25 * rate as f64) as usize,
    };
    let channels = audio.channels();
    let n = models.remove.len().min(channels);
    // Channel c goes to session c % n.
    let parts = on_sessions(&models.remove[..n], job, |i, sep, control| {
        let mine: Vec<usize> = (i..channels).step_by(n).collect();
        let part = AudioBuffer::new(audio.data.select(Axis(0), &mine), rate);
        let mut sink = BufferSink::new();
        sep.remove_in_regions(&mut BufferSource::new(&part), &plan, &mut sink, control)
            .map_err(|e| anyhow!("removal: {e}"))?;
        let mut outputs = sink.into_stems().into_iter();
        let result = outputs.next().ok_or_else(|| anyhow!("no result"))?.1;
        let removed = outputs.next().ok_or_else(|| anyhow!("no removed part"))?.1;
        Ok((mine, result, removed))
    })?;
    let mut result = Array2::zeros(audio.data.dim());
    let mut removed = Array2::zeros(audio.data.dim());
    for (mine, r, m) in parts {
        for (k, &ch) in mine.iter().enumerate() {
            result.row_mut(ch).assign(&r.data.row(k));
            removed.row_mut(ch).assign(&m.data.row(k));
        }
    }
    Ok(Removal {
        result: AudioBuffer::new(result, rate),
        removed: AudioBuffer::new(removed, rate),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_ignores_a_lone_frame_and_keeps_a_steady_level() {
        let mut x = vec![-90.0f32; 20];
        x[3] = 0.0;
        for v in &mut x[10..20] {
            *v = -20.0;
        }
        let m = medians(&x, 2);
        assert_eq!(m[3], -90.0);
        assert_eq!(m[15], -20.0);
        assert_eq!(m.len(), 20);
    }

    #[test]
    fn runs_merge_bridge_and_drop_short_ones() {
        let p = DetectParams {
            min_region: 1.5,
            bridge: 1.0,
            ..DetectParams::default()
        };
        // frames of 0.5 s; bridge 2 frames; minimum 3 frames
        let active = [
            true, true, false, false, true, true, // merged: 0..6
            false, false, false, // gap of 3: not bridged
            true, false, false, false, false, // single frame: dropped
            true, true, true, // kept
        ];
        let regions = runs_to_regions(&active, 100, 10_000, p);
        assert_eq!(
            regions.iter().map(|r| (r.start, r.end)).collect::<Vec<_>>(),
            vec![(0, 600), (1400, 1700)]
        );
    }
}
