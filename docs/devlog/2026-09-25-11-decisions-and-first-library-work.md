# 11. Decisions taken, and the first library work for the app

2026-09-25.

## Decisions by the owner

egui on wgpu (0003); the GPU matrix (0005); the engine as a child
process (0007); Windows in the first release, voice clean-up in the
first release, our own AC-3 decoder, the localization order, and all
app-facing library work in charon-audio 0.1.2 (0008); the library
dual-licensed MIT OR Apache-2.0 and the app source-available (0006); a
free public beta followed by a time-limited trial, with installs and
trials counted (0009).

## Research done for the open parts

- Training data (`research/2026-09-25-training-data.md`): the product
  only needs the music boundary right, so whole songs with vocals are
  valid music examples and vocal stems are not needed. Clean pools
  exist for music (FMA CC BY/CC0/PD subset), speech (VCTK, HiFiTTS,
  Emilia-YODAS, ICSI, ECHO) and effects (FSD50K CC0/CC BY, Kenney,
  Freedoom). Proposed policy in 0010.
- Compute (research notes kept outside this repository): TIGER is about
  15 times cheaper per second of audio than Bandit v1; a full run is a
  hypothesis of 1-3 days on one RTX 4090, to be replaced by a measured
  pilot on the owner's machine first.
- Beta and trial (0009): three separable network channels with no
  identifiers, an opt-in beta token that later unlocks a discount, a
  14-days-of-use trial with an offline Ed25519 token.

## Library work so far (charon-audio, branch release/0.1.2)

- Dual licence files and notes.
- Symphonia optional (`decode`) and its AAC decoder separate (`aac`),
  so the app can decode through the operating system.
- Progress and cancellation (`Control`, `CancelToken`): progress in
  model windows, cancel within one window. The CLI progress bar, which
  had never moved, now follows real progress.
- Streaming separation (`AudioSource`, `StemSink`,
  `Separator::separate_stream`): bounded memory for multi-hour files.
  Output is **bit-identical** to whole-buffer separation, checked with
  identity models and with HTDemucs on CPU and CoreML (max difference
  0.0).
- Region-limited removal (`Separator::remove_in_regions`): outside the
  regions the output is the input bit for bit; inside, the target stem
  is subtracted from the mix with a crossfade at the edges; progress
  counts all regions as one job.
- CI: new feature combinations and a Windows job.

## What went wrong

- A test loop over feature sets passed a multi-word variable unquoted
  in zsh, which does not word-split: cargo received one argument. The
  same class of bug had invalidated a profiling sweep a day earlier.
- The first version of the region writer let one region's right-hand
  context overwrite the start of the next region with unprocessed
  audio when the two were close. Found while designing the test; each
  region now writes only up to the next region's start.
