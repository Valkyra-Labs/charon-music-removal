# 0011. First deliverable: a free macOS demo

Date: 2026-09-25. Status: accepted by the owner.

## Context

The immediate goal is a working demo that can be shown and measured,
not a commercial launch. Training a model, Windows, payments and a
server cost money and weeks. The owner asked for the minimum that is
free and fast, accepting macOS only and a different model.

## Decision

Build a **free, local-only macOS demo** of the music-removal app:

- **Platform**: macOS on Apple Silicon only.
- **Model**: the published TIGER-DnR checkpoint (weights Apache-2.0,
  1.4M parameters, dialogue/music/effects), exported to ONNX by our own
  script with a parity check against PyTorch. No training. Its
  training data (DnR) contains some non-commercial clips; acceptable
  for a free demo and disclosed in the model card. Runs on the CPU
  (small enough); CoreML is tried and measured, not required.
- **Music regions** come from the model's own music stem (short-term
  loudness), not a second model.
- **Media**: AVFoundation only (decode, preview frames, AAC encode,
  remux with the video stream passed through). No FFmpeg, no bundled
  codecs.
- **UI**: egui, with the preview above the timeline, regions, removal
  depth per region, A/B/C playback, export. Patterns from
  an earlier egui app of the owner's.
- **No network at all**: no server, no update check, no telemetry, no
  accounts, no licence. Downloads are counted by GitHub Releases.
- **Distribution**: GitHub Releases, ad-hoc signed; the README explains
  how to open an unnotarized app. Notarization when an Apple Developer
  account exists.
- **The record**: the devlog and decision records, a measured
  quality and speed record for the demo, and the charon-audio
  engineering record.

- **Inference in-process** on a worker thread: TIGER is small enough
  that the memory and crash reasons for a separate engine process
  (0007) do not bite yet; the child process returns with bigger models.
- **Licences**: every component stays green except the TIGER-DnR
  training-data note above, which decision 0002 would not accept for a
  paid product; the demo is free and says so.

## What is postponed (not cancelled)

Our own model and training data (0004, 0010), Windows and Linux (0008),
WebGPU and ORT built from source (0005), voice clean-up and our own
AC-3 decoder (0008), trial, payments and any server (0009), the
fingerprint experiment beyond a small optional check.

## Fallback

If TIGER-DnR does not export or does not reach usable quality on real
clips, the demo uses HTDemucs through charon-audio's existing path,
labelled as a research demo (its weights are for scientific purposes
only), and says so in the README.

## Update, same day

CoreML was measured and dropped for this model (1034 s and 28 GB for
one minute against 41 s on the CPU); the demo is CPU only.
