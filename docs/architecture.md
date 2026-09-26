# Architecture

Status: the full product, 2026-09-25. The current deliverable is the
smaller macOS demo in `demo-plan.md` (decision 0011): in-process
inference, AVFoundation only, no network. Decision records 0003-0007 hold the
reasoning; this file is the current shape.

## Goals the shape serves

- One Rust codebase on macOS, Windows and Linux.
- The UI never waits: nothing longer than a frame on the UI thread,
  `try_lock` only there (the rule that the owner's earlier video app paid for).
- Multi-hour files with bounded memory.
- A green licence for every shipped component (decision 0002).
- A GPU path on every OS, chosen by measurement, with CPU always
  available.

## Processes

```
+-------------------------- app process (egui) --------------------------+
| UI thread        : egui/eframe on wgpu; reads slots with try_lock      |
| video thread     : OS decoder -> preview-size NV12 -> latest-frame slot|
| audio thread     : cpal callback owns an atomic cursor; SPSC rings fed |
|                    from PCM files: original / result / removed music   |
| engine client    : JSON lines to the engine process, progress slot     |
+-------------------------------------------------------------------------+
                 | stdin/stdout JSON lines, PCM via files
+----------------------- engine process (same binary, --engine) ---------+
| charon-audio: region detection, separation, stem arithmetic;           |
| ONNX Runtime sessions resident (CoreML compile paid once);             |
| exits when idle or with the app, returning all memory                  |
+-------------------------------------------------------------------------+
```

Why a separate engine process: ONNX Runtime peaks at 1.5-3.5 GB
during a job (measured in charon-audio), a crash in a GPU driver must
not take the window down, and the owner's earlier video app measured that freed
model memory is not returned to the OS inside a long-lived UI
process. Same binary started with `--engine`, as the owner's earlier video app's
`--hear` child. Audio travels as files (memory-mapped f32), never
through the pipe.

## Crates (workspace)

| crate | role | depends on |
|---|---|---|
| `cmr-app` | egui UI, windows, menus, playback, preview | cmr-core, cmr-media, cmr-engine-client |
| `cmr-core` | project model: file, tracks, regions, settings, job states; serialisable; no UI, no OS | serde |
| `cmr-media` | trait `Media` + backends: `macos` (AVFoundation, VideoToolbox, AudioToolbox via objc2), `windows` (Media Foundation via windows crate), `linux` (runtime FFmpeg or GStreamer from the Flatpak runtime) | cmr-core |
| `cmr-engine` | the `--engine` side: wraps charon-audio, streams, cancels, reports progress | charon-audio |
| `cmr-license` | offline Ed25519 licence file verification | ed25519-dalek |

`cmr-media` operations:
- `probe(path)`: container, tracks, codecs, channel layouts, duration.
- `extract_audio(path, track) -> PCM file`: decode through the OS.
- `video_frames(path, size) -> stream of NV12 frames` for preview.
- `remux(path, replacements, out)`: copy every stream except the
  replaced audio tracks, encode those through the OS (AAC) or as
  FLAC/Opus/ALAC where the container allows; timestamps preserved.

## Data on disk (per opened file, in the cache directory)

```
<cache>/<file-hash>/
  manifest.json        probe result, versions, model hashes
  track-<n>.f32        decoded audio per track (48 kHz or source rate)
  track-<n>.peaks      100 peaks per second (same format as the owner's earlier video app)
  regions.json         detected + edited music regions
  result-<n>.f32       processed audio, only regions differ
  music-<n>.f32        removed music, for solo preview
```

The original file is only read. Export always writes a new file.

## Processing pipeline (engine)

1. Detect regions: TVSM speech/music activity at 16 kHz over the
   whole track (small, fast), hysteresis and minimum durations,
   merged; the user edits them.
2. Separate inside regions only, with context padding and
   crossfades at the edges, through the DME model (TIGER-DnR family,
   retrained clean; decision 0004). Mono models run per channel for
   any layout, including 5.1.
3. Combine: `result = mix - g * music_estimate` inside regions
   (g from the per-region depth), the original sample-for-sample
   outside. Optional voice clean-up (DPDFNet) on the dialogue path.
4. Write result and music files; the UI swaps A/B by switching which
   ring the audio callback reads.

## GPU matrix (decision 0005)

| OS | order | shipped runtime pieces |
|---|---|---|
| macOS arm64 | CoreML (MLProgram) -> WebGPU (Metal) -> CPU | ORT built by us; CoreML is a system framework |
| macOS x86_64 | WebGPU -> CPU | universal ORT built by us |
| Windows | WebGPU (D3D12) -> CPU; DirectML only if measured clearly faster and accepted under its licence | ORT + Dawn built by us |
| Linux | WebGPU (Vulkan) -> CPU | ORT + Dawn built by us |
| any, NVIDIA opt-in | CUDA via a user-installed ORT (`load-dynamic`) | nothing redistributed |

Selection at first run by a measured micro-benchmark on the user's
machine, cached, overridable in settings.

## Threads and invariants carried from the owner's earlier video app

- UI reads shared state with `try_lock`; a miss keeps the previous
  value, never a default (a default once stamped frame zero into a
  document).
- The preview texture is updated in place with a generation counter;
  never created and dropped per frame.
- The audio callback owns the play position (atomic); seeks are
  atomics the callback consumes; a dead output stream is rebuilt.
- Work longer than a frame goes to a worker or the engine process.

## What is deliberately not here

FFmpeg bundled by us (Linux uses the Flatpak runtime's), any video
encoder, any generative model, any network access except the optional
update check, any account.
