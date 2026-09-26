# Measurements

Every number the README or the devlog quotes about the app comes from
here. Host: Apple M4 Pro, 24 GB, macOS 26.6. Toolchain: Rust 1.98.0.
App: working tree of 2026-09-25 (demo, before its first code commit),
release build; sections dated 2026-09-26 are from the working tree of
that day, which became the first code commit. Engine: charon-audio branch `release/0.1.2` on top of
`9fd8eae` (charon-audio `main` after the history cleanup of 2026-09-26). Model: `tiger_music.onnx` SHA-256 `bb52541d...` (see
charon-audio `docs/MODELS.md`). CPU only.

## Synthetic set, 2026-09-25

`tools/eval/make_synthetic.py`: eight 40 s clips of continuous
synthesized speech (macOS voices) with music added from 12 s to 30 s at
-3, -9 or -15 dB relative to the speech; four clips use MUSDB18 preview
songs with vocals (singing must be removed as music), four use
instrumental cinematic tracks. No sound effects in this set.
`tools/eval/evaluate.py` runs `charon-music-removal run-audio` on each
clip (detection on the mono downmix, removal on both channels, removal
windows with 0.25 overlap) and scores it:

- detection precision and recall over 0.5 s frames against the true
  music interval;
- music residual: 20 log10 |alpha|, alpha the least-squares projection
  of (result - clean speech) on the placed music, i.e. how much of the
  original music is still linearly present (lower is better); it does
  not count separation artefacts;
- speech SI-SDR inside the music interval, of the input and of the
  result, against the clean speech;
- the maximum difference between result and input outside the detected
  regions.

| clip | music | level | precision | recall | music residual | speech SI-SDR in -> out | wall |
|---|---|---|---|---|---|---|---|
| 00 | songs (MUSDB) | -3 dB | 1.00 | 1.00 | -10.2 dB | 3.0 -> 9.3 dB | 43.5 s |
| 01 | cinematic | -3 dB | 1.00 | 1.00 | -18.9 dB | 3.0 -> 12.7 dB | 42.0 s |
| 02 | songs | -9 dB | 1.00 | 1.00 | -19.0 dB | 9.0 -> 19.1 dB | 42.0 s |
| 03 | cinematic | -9 dB | 1.00 | 1.00 | -12.3 dB | 9.0 -> 16.3 dB | 42.0 s |
| 04 | songs | -15 dB | 1.00 | 0.50 | -3.8 dB | 15.0 -> 17.1 dB | 49.9 s |
| 05 | cinematic | -15 dB | 1.00 | 1.00 | -21.4 dB | 15.0 -> 26.0 dB | 41.9 s |
| 06 | songs | -9 dB | 1.00 | 1.00 | -14.5 dB | 9.0 -> 17.0 dB | 41.8 s |
| 07 | cinematic | -9 dB | 1.00 | 1.00 | -20.4 dB | 9.0 -> 18.4 dB | 41.9 s |

Median music residual -16.7 dB; median speech SI-SDR gain +8.7 dB.
Outside the detected regions the result equals the input exactly in
every clip (maximum difference 0.0).

The same set with 0.5 removal overlap gave the same quality within
1 dB either way and took 50-55 s per clip instead of 42 s; 0.25 is the
default.

### 2026-09-26: recalibrated detector, two sessions

Detector: medians over 5.5 s (11 frames) of the music level (above
-65 dBFS) and of its share of the mix (above -35 dB), instead of
per-frame thresholds of -45 dBFS and -18 dB (see "Real recordings").
Two model sessions of 6 threads. Every clip: precision 1.00, recall
1.00, outside max difference 0.0. Only clip 04 changed: recall 0.50 ->
1.00, music residual -3.8 -> -9.1 dB, speech SI-SDR 15.0 -> 19.9 dB.
Medians: residual -16.7 dB, SI-SDR gain +8.7 dB (unchanged: clip 04 is
not at the median). Wall 26-27 s per clip instead of 42 s.

Rejected on the same set: removal without overlap. The scores stayed
within 1.4 dB and removal was 18-24% faster, but at the window seams
the residual jumps by 4-20 times the largest jump in the surrounding
100 ms in four of seven clips (`seams.py`, not kept): clicks.

What this does not support: real gameplay or vlog recordings (none yet;
the owner is recording test clips), sound effects preservation (no
effects in this set), anything about content-matching systems.

## Negative control

A 90 s gameplay recording with the game's music switched off: no music
region detected; the music stem stayed around -100 dBFS in all 181
frames. With the 2026-09-26 detector: still no region; the smoothed
share peaks at -40.8 dB (threshold -35) and the smoothed level at
-79.4 dBFS (threshold -65).

## Real recordings, 2026-09-26

Three gameplay recordings by the owner, no commentary: H.264 1080p60,
AAC 48 kHz stereo; Hades (166 s), RoboCop: Rogue City (129 s), Hogwarts
Legacy (281 s, with a quiet score under dialogue). No ground truth:
region boundaries still need a listening check.

What the frame levels showed on Hogwarts Legacy, 20-84 s: the music
stem sits at -50 dBFS (median; p10 -57, p90 -44) both in the pauses,
where it is the whole mix (share about 0 dB), and under the dialogue,
where its share falls to -20 to -28 dB. The per-frame rule (-45 dBFS,
-18 dB) rejected it. Speech alone gives far lower shares: median -52 dB
and p90 -42 dB on the RE8 recording, median -68 dB on the synthetic
set's speech-only frames. A median over 5.5 s separates the two with a
margin, as the table shows.

| recording | detector | regions | music found | whole analysis | x length | peak RSS |
|---|---|---|---|---|---|---|
| Hades | per-frame, 1 session (baseline) | 7 | 129.5 s | 245 s | 1.47 | 3.8 GB |
| Hades | per-frame, 2 sessions | 7 (same) | 129.5 s | 146 s | 0.88 | 7.2 GB |
| Hades | median, 2 sessions | 4 | 152 s | 135 s | 0.81 | 7.2 GB |
| RoboCop | per-frame, 1 session (baseline) | 7 | 106 s | 217 s | 1.68 | 3.7 GB |
| RoboCop | per-frame, 2 sessions | 7 (same) | 106 s | 130 s | 1.01 | 7.1 GB |
| RoboCop | median, 2 sessions | 2 | 125.5 s | 109 s | 0.85 | 7.2 GB |
| Hogwarts | per-frame, 1 session (baseline) | 7 | 74 s | 240 s | 0.85 | 4.0 GB |
| Hogwarts | per-frame, 2 sessions | 7 (same) | 74 s | 141 s | 0.50 | 7.5 GB |
| Hogwarts | median, 2 sessions | 5 | 255 s | 227 s | 0.81 | 7.6 GB |

"Whole analysis": `run-audio` wall time (decode, model load, detection,
removal), `/usr/bin/time -l`. The baseline rows are the build before
this work (CoreML tried and failed at each load, then CPU on 12
threads); a rerun with the CPU provider set explicitly gave the same
times within 3%. Two sessions against one, same regions: the outputs
differ by at most 1.1e-3, 72-81 dB below the removed part, because a
session on 6 threads sums in a different order than one on 12.

Where the time goes (median detector, 2 sessions): detection 29-59 s
(0.21-0.23 x length, the whole file once on a mono downmix), removal
79-167 s (both channels, 0.25 overlap, only inside regions plus 3 s of
context). With music almost throughout, removal covers most of the file.

## Preview, 2026-09-26

`charon-music-removal frames` and `preview-sync` on the Hades and
Hogwarts recordings (1080p60 H.264, preview at 1280 x 720):

| path | cost |
|---|---|
| image generator, one seek per frame (the old preview) | p50 24-27 ms, p95 34-39 ms per frame |
| asset reader opened at a time, to the first frame | p50 27-32 ms, p95 37-41 ms |
| asset reader, next frame in order (decode, scale, BGRA to RGBA) | p50 0.90 ms, p95 1.6 ms |
| preview thread following a clock at wall speed | 60.0 frames/s, gap p50 16.5 ms, p99 18.9 ms, max 19.2 ms |
| same, after a 40 s jump | first new frame after 20-27 ms |
| paused seek | new frame after p50 34-37 ms, max 50 ms |

The old preview asked the image generator for a frame at most every
1/30 s of playback, each a seek of about 25 ms, so it could not show
more than about 30 frames a second and lagged the sound by a seek. The new one decodes in order while playing
and seeks only on a jump or while paused.

## Media path

The same 90 s recording (H.264 1080p, AAC 48 kHz) exported through the
app: the video stream is bit-identical to the source (SHA-256 of the
video packets equal), the audio is AAC-LC 44.1 kHz stereo at about
125 kbit/s. Decode 0.4 s, export 0.5 s.

## Speed

Superseded for real recordings by the table above (0.81-0.85 x length
with two sessions). The 2026-09-25 figure, one session:
about 42 s of processing per 40 s clip with 18 s of music on the CPU:
detection runs over the whole file on a mono downmix, removal only over
the music regions (plus 3 s of context each side) on both channels.
The model runs at about 0.33 x real time per channel per pass on this
CPU; CoreML is not usable for it (charon-audio `docs/MEASUREMENTS.md`).

## Bundle

`tools/bundle.sh`: the app 64 MB (binary 42 MB with ONNX Runtime
statically linked, model 25 MB), the DMG 22 MB, ad-hoc signed.
