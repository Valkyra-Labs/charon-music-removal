//! Media through AVFoundation (macOS): probe, audio decode, preview
//! frames, and export of the same container with the video stream copied
//! and the audio replaced. No FFmpeg and no bundled codecs: every codec is
//! the operating system's.

use anyhow::{anyhow, bail, Context, Result};
use charon_audio::{AudioBuffer, AudioFile};
use ndarray::Array2;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_av_foundation::{
    AVAsset, AVAssetExportPresetAppleM4A, AVAssetExportPresetPassthrough, AVAssetExportSession,
    AVAssetExportSessionStatus, AVAssetImageGenerator, AVAssetReader, AVAssetReaderStatus,
    AVAssetReaderTrackOutput, AVAssetTrack, AVFileType, AVFileTypeAppleM4A, AVFileTypeMPEG4,
    AVFileTypeQuickTimeMovie, AVMediaTypeAudio, AVMediaTypeVideo, AVMutableComposition, AVURLAsset,
};
use objc2_avf_audio::{
    AVFormatIDKey, AVLinearPCMBitDepthKey, AVLinearPCMIsBigEndianKey, AVLinearPCMIsFloatKey,
    AVLinearPCMIsNonInterleaved, AVNumberOfChannelsKey, AVSampleRateKey,
};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_core_graphics::{
    CGBitmapContextCreate, CGColorSpace, CGContext, CGImage, CGImageAlphaInfo,
};
use objc2_core_media::{CMTime, CMTimeRange};
use objc2_foundation::{NSArray, NSDictionary, NSNumber, NSString, NSURL};
use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use std::sync::mpsc;

/// Sample rate the audio is decoded at: the model's.
pub const RATE: u32 = 44_100;
/// Channels the audio is decoded at (AVFoundation downmixes or upmixes).
pub const CHANNELS: usize = 2;
/// `kAudioFormatLinearPCM` ('lpcm').
const LINEAR_PCM: u32 = 0x6C70_636D;

/// What the file contains.
#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub duration: f64,
    /// Display size of the first video track, if any.
    pub video: Option<(u32, u32)>,
    pub audio_tracks: usize,
}

fn url(path: &Path) -> Retained<NSURL> {
    NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()))
}

fn asset(path: &Path) -> Retained<AVURLAsset> {
    unsafe { AVURLAsset::URLAssetWithURL_options(&url(path), None) }
}

#[allow(deprecated)] // synchronous track access is fine off the UI thread
fn tracks(asset: &AVAsset, audio: bool) -> Retained<NSArray<AVAssetTrack>> {
    let kind = if audio {
        unsafe { AVMediaTypeAudio }
    } else {
        unsafe { AVMediaTypeVideo }
    }
    .expect("AVFoundation media type constant");
    unsafe { asset.tracksWithMediaType(kind) }
}

/// Duration, video size and audio track count.
pub fn probe(path: &Path) -> Result<MediaInfo> {
    if !path.exists() {
        bail!("{} does not exist", path.display());
    }
    let asset = asset(path);
    let duration = unsafe { asset.duration().seconds() };
    let video = tracks(&asset, false).firstObject().map(|t| {
        let size = unsafe { t.naturalSize() };
        (
            size.width.abs().round() as u32,
            size.height.abs().round() as u32,
        )
    });
    let audio_tracks = tracks(&asset, true).count();
    if !duration.is_finite() || duration <= 0.0 {
        bail!(
            "{} is not a media file AVFoundation can read",
            path.display()
        );
    }
    Ok(MediaInfo {
        duration,
        video,
        audio_tracks,
    })
}

fn number_u32(v: u32) -> Retained<NSNumber> {
    NSNumber::new_u32(v)
}

/// Decode the first audio track to 32-bit float at [`RATE`] Hz, [`CHANNELS`]
/// channels. `progress` receives the decoded fraction.
pub fn decode_audio(path: &Path, progress: &dyn Fn(f32)) -> Result<AudioBuffer> {
    let asset = asset(path);
    let duration = unsafe { asset.duration().seconds() };
    let track = tracks(&asset, true)
        .firstObject()
        .ok_or_else(|| anyhow!("{} has no audio track", path.display()))?;

    let keys: [&NSString; 7] = unsafe {
        [
            AVFormatIDKey.unwrap(),
            AVSampleRateKey.unwrap(),
            AVNumberOfChannelsKey.unwrap(),
            AVLinearPCMBitDepthKey.unwrap(),
            AVLinearPCMIsFloatKey.unwrap(),
            AVLinearPCMIsNonInterleaved.unwrap(),
            AVLinearPCMIsBigEndianKey.unwrap(),
        ]
    };
    let values = [
        number_u32(LINEAR_PCM),
        NSNumber::new_f64(RATE as f64),
        number_u32(CHANNELS as u32),
        number_u32(32),
        NSNumber::new_bool(true),
        NSNumber::new_bool(false),
        NSNumber::new_bool(false),
    ];
    let objects: Vec<&AnyObject> = values.iter().map(|v| v.as_ref()).collect();
    let settings = NSDictionary::from_slices(&keys, &objects);

    let reader = unsafe { AVAssetReader::assetReaderWithAsset_error(&asset) }
        .map_err(|e| anyhow!("cannot read {}: {e}", path.display()))?;
    let output = unsafe {
        AVAssetReaderTrackOutput::assetReaderTrackOutputWithTrack_outputSettings(
            &track,
            Some(&settings),
        )
    };
    unsafe {
        output.setAlwaysCopiesSampleData(false);
        reader.addOutput(&output);
        if !reader.startReading() {
            bail!("AVAssetReader could not start: {:?}", reader.error());
        }
    }

    let expected = (duration * RATE as f64).ceil() as usize;
    let mut interleaved: Vec<f32> = Vec::with_capacity(expected * CHANNELS + 4096);
    let mut last_report = 0usize;
    while let Some(sample) = unsafe { output.copyNextSampleBuffer() } {
        let Some(block) = (unsafe { sample.data_buffer() }) else {
            continue;
        };
        let bytes = unsafe { block.data_length() };
        let start = interleaved.len();
        interleaved.resize(start + bytes / 4, 0.0);
        let dst = NonNull::new(interleaved[start..].as_mut_ptr().cast()).expect("non-null");
        let status = unsafe { block.copy_data_bytes(0, bytes, dst) };
        if status != 0 {
            bail!("CMBlockBufferCopyDataBytes failed: {status}");
        }
        let frames = interleaved.len() / CHANNELS;
        if frames - last_report > RATE as usize * 5 {
            last_report = frames;
            progress((frames as f32 / expected.max(1) as f32).min(1.0));
        }
    }
    if unsafe { reader.status() } == AVAssetReaderStatus::Failed {
        bail!("decoding failed: {:?}", unsafe { reader.error() });
    }
    let frames = interleaved.len() / CHANNELS;
    let mut data = Array2::zeros((CHANNELS, frames));
    for (i, frame) in interleaved.as_chunks::<CHANNELS>().0.iter().enumerate() {
        for (ch, &x) in frame.iter().enumerate() {
            data[[ch, i]] = x;
        }
    }
    progress(1.0);
    Ok(AudioBuffer::new(data, RATE))
}

/// Preview frames at a bounded size.
pub struct FrameSource {
    generator: Retained<AVAssetImageGenerator>,
}

// AVAssetImageGenerator is used from one worker thread at a time.
unsafe impl Send for FrameSource {}

/// An RGBA frame.
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

impl FrameSource {
    /// Frames scaled to fit `max_width` x `max_height`.
    pub fn new(path: &Path, max_width: f64, max_height: f64) -> Self {
        let asset = asset(path);
        let generator = unsafe { AVAssetImageGenerator::assetImageGeneratorWithAsset(&asset) };
        unsafe {
            generator.setAppliesPreferredTrackTransform(true);
            generator.setMaximumSize(CGSize::new(max_width, max_height));
            // A frame near the time is enough for preview and much faster
            // than an exact one.
            let tolerance = CMTime::with_seconds(0.1, 600);
            generator.setRequestedTimeToleranceBefore(tolerance);
            generator.setRequestedTimeToleranceAfter(tolerance);
        }
        Self { generator }
    }

    /// The frame at `seconds`.
    #[allow(deprecated)] // the synchronous call is what a worker thread wants
    pub fn frame(&self, seconds: f64) -> Result<Frame> {
        let time = unsafe { CMTime::with_seconds(seconds.max(0.0), 600) };
        let image = unsafe {
            self.generator
                .copyCGImageAtTime_actualTime_error(time, std::ptr::null_mut())
        }
        .map_err(|e| anyhow!("no frame at {seconds:.2} s: {e}"))?;
        to_rgba(&image)
    }
}

/// Preview size for a video of `size`: at most 1280 x 720, aspect kept,
/// even dimensions.
pub fn preview_size(size: (u32, u32)) -> (usize, usize) {
    let (w, h) = (size.0.max(2) as f64, size.1.max(2) as f64);
    let scale = (1280.0 / w).min(720.0 / h).min(1.0);
    let even = |x: f64| ((x * scale / 2.0).round() as usize * 2).max(2);
    (even(w), even(h))
}

/// Frames in order from a start time, decoded by the system (hardware
/// where available) and scaled to a bounded size by the decoder.
///
/// Playback reads frames one after another instead of seeking for each:
/// a seek costs a keyframe-to-target decode, a sequential frame costs one
/// frame (the lesson of an earlier player, where a per-frame seek
/// could not keep up with 60 fps masters).
pub struct FrameReader {
    reader: Retained<AVAssetReader>,
    output: Retained<AVAssetReaderTrackOutput>,
}

// Used from one worker thread at a time.
unsafe impl Send for FrameReader {}

/// A decoded frame and its presentation time.
pub struct TimedFrame {
    pub seconds: f64,
    pub frame: Frame,
}

impl FrameReader {
    /// Frames from `start` seconds on, scaled to `width` x `height` (the
    /// caller keeps the aspect ratio).
    pub fn new(path: &Path, start: f64, width: usize, height: usize) -> Result<Self> {
        let asset = asset(path);
        let track = tracks(&asset, false)
            .firstObject()
            .ok_or_else(|| anyhow!("{} has no video track", path.display()))?;
        let keys: [&NSString; 3] = unsafe {
            [
                cf_key(objc2_core_video::kCVPixelBufferPixelFormatTypeKey),
                cf_key(objc2_core_video::kCVPixelBufferWidthKey),
                cf_key(objc2_core_video::kCVPixelBufferHeightKey),
            ]
        };
        let values = [
            number_u32(objc2_core_video::kCVPixelFormatType_32BGRA),
            number_u32(width as u32),
            number_u32(height as u32),
        ];
        let objects: Vec<&AnyObject> = values.iter().map(|v| v.as_ref()).collect();
        let settings = NSDictionary::from_slices(&keys, &objects);
        let reader = unsafe { AVAssetReader::assetReaderWithAsset_error(&asset) }
            .map_err(|e| anyhow!("cannot read {}: {e}", path.display()))?;
        let output = unsafe {
            AVAssetReaderTrackOutput::assetReaderTrackOutputWithTrack_outputSettings(
                &track,
                Some(&settings),
            )
        };
        unsafe {
            output.setAlwaysCopiesSampleData(false);
            reader.addOutput(&output);
            reader.setTimeRange(CMTimeRange {
                start: CMTime::with_seconds(start.max(0.0), 600),
                duration: asset.duration(),
            });
            if !reader.startReading() {
                bail!("AVAssetReader could not start: {:?}", reader.error());
            }
        }
        Ok(Self { reader, output })
    }

    /// The next frame, or `None` at the end.
    pub fn next(&mut self) -> Option<Result<TimedFrame>> {
        loop {
            let sample = unsafe { self.output.copyNextSampleBuffer() }?;
            let seconds = unsafe { sample.presentation_time_stamp().seconds() };
            // Buffers without an image (format changes, markers) are
            // skipped.
            let Some(image) = (unsafe { sample.image_buffer() }) else {
                continue;
            };
            return Some(bgra_to_rgba(&image).map(|frame| TimedFrame { seconds, frame }));
        }
    }
}

impl Drop for FrameReader {
    fn drop(&mut self) {
        unsafe { self.reader.cancelReading() };
    }
}

/// A CoreFoundation string key as the Foundation string it is
/// toll-free bridged to.
unsafe fn cf_key(key: &'static objc2_core_foundation::CFString) -> &'static NSString {
    unsafe { &*(key as *const objc2_core_foundation::CFString as *const NSString) }
}

fn bgra_to_rgba(image: &objc2_core_video::CVPixelBuffer) -> Result<Frame> {
    use objc2_core_video::{
        CVPixelBufferGetBaseAddress, CVPixelBufferGetBytesPerRow, CVPixelBufferGetHeight,
        CVPixelBufferGetWidth, CVPixelBufferLockBaseAddress, CVPixelBufferLockFlags,
        CVPixelBufferUnlockBaseAddress,
    };
    let flags = CVPixelBufferLockFlags::ReadOnly;
    if unsafe { CVPixelBufferLockBaseAddress(image, flags) } != 0 {
        bail!("cannot lock a decoded frame");
    }
    let width = CVPixelBufferGetWidth(image);
    let height = CVPixelBufferGetHeight(image);
    let stride = CVPixelBufferGetBytesPerRow(image);
    let base = CVPixelBufferGetBaseAddress(image) as *const u8;
    let mut rgba = vec![0u8; width * height * 4];
    if !base.is_null() {
        let src = unsafe { std::slice::from_raw_parts(base, stride * height) };
        for (y, dst_row) in rgba.chunks_exact_mut(width * 4).enumerate() {
            let src_row = &src[y * stride..y * stride + width * 4];
            for (d, s) in dst_row
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(src_row.as_chunks::<4>().0)
            {
                *d = [s[2], s[1], s[0], 255];
            }
        }
    }
    unsafe { CVPixelBufferUnlockBaseAddress(image, flags) };
    Ok(Frame {
        width,
        height,
        rgba,
    })
}

fn to_rgba(image: &CGImage) -> Result<Frame> {
    let width = CGImage::width(Some(image));
    let height = CGImage::height(Some(image));
    let mut rgba = vec![0u8; width * height * 4];
    let space = CGColorSpace::new_device_rgb().ok_or_else(|| anyhow!("no RGB colour space"))?;
    let context = unsafe {
        CGBitmapContextCreate(
            rgba.as_mut_ptr().cast(),
            width,
            height,
            8,
            width * 4,
            Some(&space),
            CGImageAlphaInfo::PremultipliedLast.0,
        )
    }
    .ok_or_else(|| anyhow!("cannot create a bitmap context"))?;
    let rect = CGRect::new(
        CGPoint::new(0.0, 0.0),
        CGSize::new(width as f64, height as f64),
    );
    CGContext::draw_image(Some(&context), rect, Some(image));
    drop(context);
    Ok(Frame {
        width,
        height,
        rgba,
    })
}

fn wait_for(session: &AVAssetExportSession, what: &str, progress: &dyn Fn(f32)) -> Result<()> {
    let (tx, rx) = mpsc::channel::<()>();
    let block = block2::RcBlock::new(move || {
        let _ = tx.send(());
    });
    unsafe { session.exportAsynchronouslyWithCompletionHandler(&block) };
    loop {
        match rx.recv_timeout(std::time::Duration::from_millis(200)) {
            Ok(()) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => progress(unsafe { session.progress() }),
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    match unsafe { session.status() } {
        AVAssetExportSessionStatus::Completed => Ok(()),
        status => bail!("{what} failed ({status:?}): {:?}", unsafe {
            session.error()
        }),
    }
}

fn file_type_for(path: &Path) -> Result<&'static AVFileType> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let t = unsafe {
        match ext.as_str() {
            "mp4" | "m4v" => AVFileTypeMPEG4,
            "mov" => AVFileTypeQuickTimeMovie,
            other => bail!("cannot export to .{other}; use .mp4 or .mov"),
        }
    };
    t.ok_or_else(|| anyhow!("AVFoundation file type constant missing"))
}

/// Write `source` with its audio replaced by `audio`: the video track is
/// copied without re-encoding, the new audio is encoded to AAC by the
/// operating system. `progress` goes from 0 to 1 over both steps.
pub fn export(
    source: &Path,
    audio: &AudioBuffer,
    destination: &Path,
    progress: &dyn Fn(f32),
) -> Result<()> {
    let file_type = file_type_for(destination)?;
    let work = tempdir()?;
    let wav = work.join("audio.wav");
    let m4a = work.join("audio.m4a");
    AudioFile::write_wav(&wav, audio).context("writing the processed audio")?;

    // 1. PCM to AAC through the system encoder.
    let wav_asset = asset(&wav);
    let aac = unsafe {
        AVAssetExportSession::exportSessionWithAsset_presetName(
            &wav_asset,
            AVAssetExportPresetAppleM4A,
        )
    }
    .ok_or_else(|| anyhow!("no AAC export session"))?;
    unsafe {
        aac.setOutputURL(Some(&url(&m4a)));
        aac.setOutputFileType(AVFileTypeAppleM4A);
    }
    wait_for(&aac, "AAC encoding", &|p| progress(0.5 * p))?;

    // 2. Video from the source, audio from the new file, passed through.
    let src = asset(source);
    let new_audio = asset(&m4a);
    let composition = unsafe { AVMutableComposition::composition() };
    let video_range = CMTimeRange {
        start: unsafe { CMTime::with_seconds(0.0, 600) },
        duration: unsafe { src.duration() },
    };
    if let Some(video) = tracks(&src, false).firstObject() {
        let track = unsafe {
            composition.addMutableTrackWithMediaType_preferredTrackID(
                AVMediaTypeVideo.expect("video media type"),
                0,
            )
        }
        .ok_or_else(|| anyhow!("cannot add a video track"))?;
        unsafe {
            track
                .insertTimeRange_ofTrack_atTime_error(video_range, &video, video_range.start)
                .map_err(|e| anyhow!("copying the video track: {e}"))?;
            track.setPreferredTransform(video.preferredTransform());
        }
    }
    let audio_track = tracks(&new_audio, true)
        .firstObject()
        .ok_or_else(|| anyhow!("the encoded audio has no track"))?;
    let audio_range = CMTimeRange {
        start: video_range.start,
        duration: unsafe { new_audio.duration() },
    };
    let track = unsafe {
        composition.addMutableTrackWithMediaType_preferredTrackID(
            AVMediaTypeAudio.expect("audio media type"),
            0,
        )
    }
    .ok_or_else(|| anyhow!("cannot add an audio track"))?;
    unsafe {
        track
            .insertTimeRange_ofTrack_atTime_error(audio_range, &audio_track, audio_range.start)
            .map_err(|e| anyhow!("adding the new audio: {e}"))?;
    }

    if destination.exists() {
        std::fs::remove_file(destination).context("replacing the output file")?;
    }
    let mux = unsafe {
        AVAssetExportSession::exportSessionWithAsset_presetName(
            &composition,
            AVAssetExportPresetPassthrough,
        )
    }
    .ok_or_else(|| anyhow!("no passthrough export session"))?;
    unsafe {
        mux.setOutputURL(Some(&url(destination)));
        mux.setOutputFileType(Some(file_type));
    }
    wait_for(&mux, "writing the video", &|p| progress(0.5 + 0.5 * p))?;
    let _ = std::fs::remove_dir_all(&work);
    progress(1.0);
    Ok(())
}

fn tempdir() -> Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!(
        "charon-music-removal-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
