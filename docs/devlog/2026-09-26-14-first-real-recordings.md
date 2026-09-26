# 14. First real recordings: preview, speed, a quiet score

2026-09-26. Numbers and stamps in `../measurements.md` (sections dated
2026-09-26) and charon-audio's `docs/MEASUREMENTS.md`.

## Context

The owner recorded three gameplay clips with music (Hades, RoboCop:
Rogue City, Hogwarts Legacy; 1080p60, 2-5 minutes, no commentary) and
reported three problems from using the demo: the video preview stutters
and lags, analysis takes too long, and Hogwarts Legacy has a quiet song
under dialogue that the detector misses.

## What was done

- **Preview.** The old preview asked AVFoundation's image generator for
  a frame at the play position, at most every 1/30 s. Each request is a
  seek: about 25 ms to decode from the previous keyframe. The fix is the
  shape of the owner's earlier video app's player, which had the same problem with
  60 fps masters: while playing, an `AVAssetReader` decodes frames in
  order (0.9 ms each, scaled to 1280 x 720 by the decoder) and each frame
  is shown when the audio clock reaches its time; the reader is reopened
  only when the clock jumps. While paused, seeks go through the image
  generator and pile-ups collapse to the latest request. Measured
  against a simulated clock: 60 frames a second, worst gap 19 ms.
- **Speed.** Processor time was only 3.5x wall time on a 12-core
  machine: one ONNX Runtime session does not use more than about 6-7
  cores on this graph of 10,000 small nodes. Two sessions of 6 threads,
  each on its own part of the work, are 1.58x faster in isolation and
  1.6-1.7x faster end to end. Detection splits the mono downmix at
  window boundaries (no overlap, so the windows are the same); removal
  gives each session one channel (the model runs per channel). A new
  `Separator::with_process_config` in charon-audio lets the detection
  and removal passes share one loaded session instead of loading the
  model twice per session.
- **Detector.** The frame levels showed the Hogwarts score steady at
  -50 dBFS, whole in the pauses and 20-28 dB under the dialogue. The
  per-frame thresholds (-45 dBFS, share -18 dB) rejected it. Medians
  over 5.5 s of level (-65 dBFS) and share (-35 dB) find it, find the
  whole of the quiet song in synthetic clip 04 that the old rule half
  missed, and still find nothing in the recording with music off.
- **Perceived speed.** Regions appear on the timeline as soon as
  detection finishes (about a quarter of the time), while separation
  continues.

## Evidence

| recording | before | after |
|---|---|---|
| Hades, 166 s | 245 s, 129.5 s of music found | 135 s, 152 s found |
| RoboCop, 129 s | 217 s, 106 s found | 109 s, 125.5 s found |
| Hogwarts, 281 s | 240 s, 74 s found | 227 s, 255 s found |

Synthetic set: all eight clips at precision and recall 1.00 (clip 04
was 0.50 recall), 27 s per clip instead of 42 s. Preview: sequential
frame 0.9 ms against 25 ms per seek.

## What went wrong

- **An automatic choice hid a failure.** The app is built with
  charon-audio's `coreml` feature and the TIGER preset left the provider
  on `Auto`, so every model load compiled the graph for CoreML, failed
  on the PReLU and fell back to the CPU with a warning nobody saw. The
  preset now says CPU.
- **Three ideas measured and dropped.** More than two sessions: three
  are no faster, four run out of memory bandwidth (4 x 3 threads took
  2x longer than 2 x 6). Graph simplification with onnxslim: the graph
  came out the same. Removal without window overlap: 18-24% faster with
  the same scores, but clicks at the window seams, 4-20 times the
  largest jump nearby, in four of seven clips. The scores could not see
  them; a seam-local check could.
- **A wrong host description.** The measurement records called this
  machine a 14-core M4 Pro; it has 12 cores (8 performance, 4
  efficiency). The first thread benchmarks asked for 14 threads and
  oversubscribed it. The conclusions stand (rerun at 12 and 2 x 6), and
  the host line is corrected.
- **Memory doubles.** Two sessions peak at 7.2-7.6 GB against 3.7-4.0
  GB. The app uses two only on machines with 16 GB or more.

## Decision

Two sessions where the machine allows, the median detector, the
sequential preview. Not taken: skipping the separate detection pass by
separating the whole file in stereo and detecting from that estimate.
It would save the detection time (about a quarter) when music runs
throughout, as in all three recordings, and cost more when music is
sparse; worth measuring on more recordings first.

## Next

The owner listens to the new regions (Hogwarts 201-273 s especially,
which the old rule did not mark). A GitHub release of the demo and
charon-audio 0.1.2.
