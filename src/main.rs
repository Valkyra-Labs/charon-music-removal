mod app;
mod engine;
mod media;
mod pipeline;
mod playback;

use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Where the model is looked for, in order: `CHARON_MUSIC_MODEL`, the app
/// bundle's Resources, `models/` next to the executable or the working
/// directory, `~/Music/Charon/models`.
fn default_model() -> PathBuf {
    if let Some(p) = std::env::var_os("CHARON_MUSIC_MODEL") {
        return PathBuf::from(p);
    }
    let name = "tiger_music.onnx";
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("../Resources").join(name));
            candidates.push(dir.join("models").join(name));
        }
    }
    candidates.push(PathBuf::from("models").join(name));
    if let Some(home) = std::env::var_os("HOME") {
        candidates.push(PathBuf::from(home).join("Music/Charon/models").join(name));
    }
    candidates
        .iter()
        .find(|p| p.exists())
        .cloned()
        .unwrap_or_else(|| candidates[candidates.len() - 1].clone())
}

fn run_window(open: Option<PathBuf>, analyze: bool) -> Result<()> {
    let model = default_model();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Charon Music Removal")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 560.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "Charon Music Removal",
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, model, open, analyze)))),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Headless run of the whole pipeline, for testing and measurement.
fn process(input: &Path, output: &Path, model: &Path) -> Result<()> {
    let t0 = Instant::now();
    let info = media::probe(input)?;
    println!(
        "{}: {:.1} s, video {:?}, {} audio track(s)",
        input.display(),
        info.duration,
        info.video,
        info.audio_tracks
    );
    let audio = media::decode_audio(input, &|_| {})?;
    let t_decode = t0.elapsed().as_secs_f64();

    let models = pipeline::Models::load(model)?;
    let detection = pipeline::detect(
        &models,
        &audio,
        pipeline::DetectParams::default(),
        &pipeline::Job::quiet(),
    )?;
    let t_detect = t0.elapsed().as_secs_f64();
    let rate = audio.sample_rate as f64;
    let music_s: f64 = detection
        .regions
        .iter()
        .map(|r| (r.end - r.start) as f64 / rate)
        .sum();
    for r in &detection.regions {
        println!(
            "  music {:.1}-{:.1} s",
            r.start as f64 / rate,
            r.end as f64 / rate
        );
    }

    let removal = pipeline::remove(&models, &audio, &detection.regions, &pipeline::Job::quiet())?;
    let t_remove = t0.elapsed().as_secs_f64();

    media::export(input, &removal.result, output, &|_| {})?;
    let t_export = t0.elapsed().as_secs_f64();
    println!(
        "decode {:.1} s, detect {:.1} s, remove {:.1} s ({:.1} s of music), export {:.1} s, total {:.1} s",
        t_decode,
        t_detect - t_decode,
        t_remove - t_detect,
        music_s,
        t_export - t_remove,
        t_export
    );
    println!("-> {}", output.display());
    Ok(())
}

/// Preview frame costs: a seek per frame (the image generator) against
/// sequential decoding (the frame reader), at the preview size.
fn frame_costs(path: &Path) -> Result<()> {
    let info = media::probe(path)?;
    let (w, h) = media::preview_size(info.video.unwrap_or((1920, 1080)));
    let pct = |v: &mut Vec<f64>, q: f64| {
        v.sort_by(f64::total_cmp);
        v[((v.len() - 1) as f64 * q).round() as usize]
    };
    let start = (info.duration * 0.3).min(60.0);

    let generator = media::FrameSource::new(path, w as f64, h as f64);
    let mut per = Vec::new();
    for i in 0..120 {
        let t = Instant::now();
        generator.frame(start + i as f64 / 60.0)?;
        per.push(t.elapsed().as_secs_f64() * 1e3);
    }
    println!(
        "generator, consecutive 1/60 s steps: p50 {:.1} ms, p95 {:.1} ms",
        pct(&mut per, 0.5),
        pct(&mut per, 0.95)
    );

    let mut open = Vec::new();
    for i in 0..12 {
        let at = info.duration * (i as f64 + 0.37) / 12.0;
        let t = Instant::now();
        let mut r = media::FrameReader::new(path, at, w, h)?;
        let f = r.next().expect("a frame")?;
        open.push(t.elapsed().as_secs_f64() * 1e3);
        if i == 0 {
            println!(
                "reader frame {}x{}, asked {at:.3} s, got {:.3} s",
                f.frame.width, f.frame.height, f.seconds
            );
        }
    }
    println!(
        "reader, open at a random time to first frame: p50 {:.1} ms, p95 {:.1} ms",
        pct(&mut open, 0.5),
        pct(&mut open, 0.95)
    );

    let mut r = media::FrameReader::new(path, start, w, h)?;
    let mut per = Vec::new();
    for _ in 0..600 {
        let t = Instant::now();
        if r.next().is_none() {
            break;
        }
        per.push(t.elapsed().as_secs_f64() * 1e3);
    }
    let total: f64 = per.iter().sum();
    let n = per.len();
    println!(
        "reader, sequential: {n} frames, {:.0} fps, p50 {:.2} ms, p95 {:.2} ms",
        n as f64 / total * 1e3,
        pct(&mut per, 0.5),
        pct(&mut per, 0.95)
    );
    Ok(())
}

/// The preview thread against a simulated clock at wall speed: 5 s of
/// play, a 40 s jump, 5 s more, then a pause with a scrub. Reports the
/// frames published per second, the longest gap between them, and how
/// long a jump and a paused seek take to show a new frame.
fn preview_sync(path: &Path) -> Result<()> {
    let info = media::probe(path)?;
    let size = info.video.ok_or_else(|| anyhow::anyhow!("no video"))?;
    let preview = engine::Preview::start(path, size, || {});
    let published = || {
        preview
            .slot
            .lock()
            .ok()
            .and_then(|s| s.as_ref().map(|f| f.0))
            .unwrap_or(0)
    };
    let wait_new = |since: u64| {
        let t = Instant::now();
        while published() == since && t.elapsed().as_secs_f64() < 2.0 {
            std::thread::sleep(std::time::Duration::from_micros(500));
        }
        t.elapsed().as_secs_f64() * 1e3
    };
    let origin = (info.duration * 0.2).min(30.0);
    preview.follow(origin, false);
    wait_new(0);
    for (leg, start) in [(1, origin), (2, origin + 40.0)] {
        let before = published();
        let t0 = Instant::now();
        let mut last = (before, t0);
        let mut gaps = Vec::new();
        let mut first = None;
        while t0.elapsed().as_secs_f64() < 5.0 {
            preview.follow(start + t0.elapsed().as_secs_f64(), true);
            let n = published();
            if n != last.0 {
                let now = Instant::now();
                if first.is_none() {
                    first = Some((now - t0).as_secs_f64() * 1e3);
                } else {
                    gaps.push((now - last.1).as_secs_f64() * 1e3);
                }
                last = (n, now);
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        gaps.sort_by(f64::total_cmp);
        println!(
            "play leg {leg} from {start:.1} s: {:.1} frames/s, first frame after {:.1} ms, gap p50 {:.1} ms, p99 {:.1} ms, max {:.1} ms",
            (last.0 - before) as f64 / 5.0,
            first.unwrap_or(f64::NAN),
            gaps[gaps.len() / 2],
            gaps[gaps.len() * 99 / 100],
            gaps[gaps.len() - 1]
        );
    }
    let mut seeks = Vec::new();
    for i in 0..10 {
        let before = published();
        preview.follow(origin + 3.0 * i as f64 + 0.7, false);
        seeks.push(wait_new(before));
    }
    seeks.sort_by(f64::total_cmp);
    println!(
        "paused seek to a new frame: p50 {:.1} ms, max {:.1} ms",
        seeks[seeks.len() / 2],
        seeks[seeks.len() - 1]
    );
    Ok(())
}

/// Throughput of N model sessions running at once, T intra-op threads
/// each, on the mono downmix of the first `seconds` split into N parts at
/// window boundaries (no overlap: the parts are independent). Prints the
/// wall time and the largest difference against a single session.
fn bench_parallel(path: &Path, seconds: f64, n: usize, threads: usize) -> Result<()> {
    use charon_audio::{models::TIGER_SEGMENT_SAMPLES, AudioBuffer, SeparatorConfig};
    let audio = media::decode_audio(path, &|_| {})?;
    let len = ((seconds * audio.sample_rate as f64) as usize).min(audio.samples());
    let mono = audio.to_mono().slice(ndarray::s![.., ..len]).to_owned();
    let windows = len.div_ceil(TIGER_SEGMENT_SAMPLES);
    let per = windows.div_ceil(n) * TIGER_SEGMENT_SAMPLES;
    let make = |t: usize| {
        let mut c = SeparatorConfig::tiger_music(default_model())
            .with_overlap(0.0)
            .with_progress(false);
        c.model.onnx.intra_threads = Some(t);
        charon_audio::Separator::new(c)
    };
    let seps: Vec<_> = (0..n).map(|_| make(threads)).collect::<Result<_, _>>()?;
    let t = Instant::now();
    let parts: Vec<ndarray::Array2<f32>> = std::thread::scope(|s| {
        let handles: Vec<_> = seps
            .iter()
            .enumerate()
            .map(|(i, sep)| {
                let a = (i * per).min(len);
                let b = ((i + 1) * per).min(len);
                let chunk = AudioBuffer::new(
                    mono.slice(ndarray::s![.., a..b]).to_owned(),
                    audio.sample_rate,
                );
                s.spawn(move || -> Result<ndarray::Array2<f32>> {
                    if chunk.samples() == 0 {
                        return Ok(ndarray::Array2::zeros((1, 0)));
                    }
                    let stems = sep.separate(&chunk).map_err(|e| anyhow::anyhow!("{e}"))?;
                    Ok(stems.get("music").expect("music").data.clone())
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("join"))
            .collect::<Result<_>>()
    })?;
    let wall = t.elapsed().as_secs_f64();
    let views: Vec<_> = parts.iter().map(|p| p.view()).collect();
    let joined = ndarray::concatenate(ndarray::Axis(1), &views)?;
    println!(
        "n {n} threads {threads}: {seconds:.0} s of audio in {wall:.1} s (RTF {:.3})",
        wall / seconds
    );
    let reference = std::env::temp_dir().join(format!("bench-par-{seconds}.f32"));
    if n == 1 && threads == 14 {
        let bytes: Vec<u8> = joined.iter().flat_map(|v| v.to_le_bytes()).collect();
        std::fs::write(&reference, bytes)?;
    } else if let Ok(bytes) = std::fs::read(&reference) {
        let max = bytes
            .as_chunks::<4>()
            .0
            .iter()
            .zip(joined.iter())
            .map(|(b, v)| (f32::from_le_bytes(*b) - v).abs())
            .fold(0.0f32, f32::max);
        println!("  max difference against n 1 threads 14: {max:e}");
    }
    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("probe") if args.len() == 2 => {
            println!("{:?}", media::probe(Path::new(&args[1]))?);
        }
        Some("levels") if args.len() >= 2 => {
            let model = args.get(2).map(PathBuf::from).unwrap_or_else(default_model);
            let audio = media::decode_audio(Path::new(&args[1]), &|_| {})?;
            let d = pipeline::detect(&pipeline::Models::load(&model)?, &audio, pipeline::DetectParams::default(), &pipeline::Job::quiet())?;
            for (i, (m, x)) in d.music_db.iter().zip(&d.mix_db).enumerate() {
                println!("{:7.1} s  music {m:6.1} dB  mix {x:6.1} dB  share {:6.1} dB", i as f64 * pipeline::FRAME_SECONDS, m - x);
            }
        }
        Some("run-audio") if args.len() >= 3 => {
            // Decode, detect, remove; write result.wav, removed.wav and
            // regions.txt (start and end in seconds) for evaluation.
            let model = args.get(3).map(PathBuf::from).unwrap_or_else(default_model);
            let out = PathBuf::from(&args[2]);
            std::fs::create_dir_all(&out)?;
            let t = Instant::now();
            let audio = media::decode_audio(Path::new(&args[1]), &|_| {})?;
            let t_decode = t.elapsed().as_secs_f64();
            let models = pipeline::Models::load(&model)?;
            let t_load = t.elapsed().as_secs_f64();
            let d = pipeline::detect(&models, &audio, pipeline::DetectParams::default(), &pipeline::Job::quiet())?;
            let t_detect = t.elapsed().as_secs_f64();
            let r = pipeline::remove(&models, &audio, &d.regions, &pipeline::Job::quiet())?;
            let t_all = t.elapsed().as_secs_f64();
            println!(
                "decode {t_decode:.1} s, load {:.1} s, detect {:.1} s, remove {:.1} s",
                t_load - t_decode,
                t_detect - t_load,
                t_all - t_detect
            );
            charon_audio::AudioFile::write_wav(out.join("result.wav"), &r.result)?;
            charon_audio::AudioFile::write_wav(out.join("removed.wav"), &r.removed)?;
            charon_audio::AudioFile::write_wav(out.join("input.wav"), &audio)?;
            let rate = audio.sample_rate as f64;
            let lines: Vec<String> = d.regions.iter().map(|r| format!("{:.3} {:.3}", r.start as f64 / rate, r.end as f64 / rate)).collect();
            std::fs::write(out.join("regions.txt"), lines.join("\n") + "\n")?;
            println!("{} regions, detect {:.1} s, total {:.1} s for {:.1} s of audio", d.regions.len(), t_detect, t_all, audio.duration());
        }
        Some("frames") if args.len() == 2 => frame_costs(Path::new(&args[1]))?,
        Some("bench-par") if args.len() == 5 => bench_parallel(
            Path::new(&args[1]),
            args[2].parse()?,
            args[3].parse()?,
            args[4].parse()?,
        )?,
        Some("preview-sync") if args.len() == 2 => preview_sync(Path::new(&args[1]))?,
        Some("process") if args.len() >= 3 => {
            let model = args.get(3).map(PathBuf::from).unwrap_or_else(default_model);
            process(Path::new(&args[1]), Path::new(&args[2]), &model)?;
        }
        Some(path) if Path::new(path).exists() => {
            run_window(Some(PathBuf::from(path)), args.iter().any(|a| a == "--analyze"))?
        }
        Some(_) => bail!(
            "usage: charon-music-removal [VIDEO | probe FILE | levels FILE [MODEL] | process IN OUT [MODEL] | run-audio IN OUT_DIR [MODEL]]"
        ),
        None => run_window(None, false)?,
    }
    Ok(())
}
